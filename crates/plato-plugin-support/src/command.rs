use anyhow::{Context, Result, bail};
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Runs a command in `workdir`, forwarding stdout and stderr to stderr.
///
/// # Errors
/// Returns an error if the command cannot be spawned, exits unsuccessfully,
/// or its output cannot be forwarded.
pub fn run_command<I, S>(program: &str, args: I, workdir: &Path) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_command_with_timeout(program, args, workdir, None)
}

/// Runs a command in `workdir`, forwarding stdout and stderr to stderr.
///
/// # Errors
/// Returns an error if the command cannot be spawned, times out, exits unsuccessfully,
/// or its output cannot be forwarded.
pub fn run_command_with_timeout<I, S>(
    program: &str,
    args: I,
    workdir: &Path,
    timeout: Option<Duration>,
) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_os_string())
        .collect::<Vec<OsString>>();
    let mut child = Command::new(program)
        .args(&args)
        .current_dir(workdir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to execute {program}"))?;

    let stdout = child
        .stdout
        .take()
        .context("Child stdout pipe was unavailable")?;
    let stderr = child
        .stderr
        .take()
        .context("Child stderr pipe was unavailable")?;
    let stdout_forwarder = forward_to_stderr(stdout);
    let stderr_forwarder = forward_to_stderr(stderr);

    let status = wait_for_child(&mut child, timeout, program)?;
    join_forwarder(stdout_forwarder)?;
    join_forwarder(stderr_forwarder)?;

    if !status.success() {
        let rendered_args = args
            .iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        bail!("Command failed: {program} {rendered_args}");
    }
    Ok(())
}

fn forward_to_stderr<R>(mut reader: R) -> thread::JoinHandle<std::io::Result<u64>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let stderr = std::io::stderr();
        let mut stderr = stderr.lock();
        std::io::copy(&mut reader, &mut stderr)
    })
}

fn join_forwarder(handle: thread::JoinHandle<std::io::Result<u64>>) -> Result<()> {
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("Output forwarder panicked"))?
        .context("Failed to forward command output")?;
    Ok(())
}

fn wait_for_child(
    child: &mut Child,
    timeout: Option<Duration>,
    program: &str,
) -> Result<ExitStatus> {
    let Some(timeout) = timeout else {
        return child
            .wait()
            .with_context(|| format!("Failed waiting for {program}"));
    };

    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }

        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            bail!(
                "Command {program} timed out after {} seconds",
                timeout.as_secs()
            );
        }

        thread::sleep(Duration::from_millis(50));
    }
}
