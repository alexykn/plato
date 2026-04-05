use plato::cli::dispatch;

fn main() {
    if let Err(error) = dispatch::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
