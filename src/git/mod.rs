pub(crate) mod cache;
pub(crate) mod fetcher;
mod setup;
pub(crate) mod spec;

pub(crate) use fetcher::{GitTemplateFetcher, TempCheckout};
pub(crate) use setup::{
    GitCommitLocalConfig, GitCoreConfig, GitInitialCommit, GitLocalConfig, GitSetupOptions,
    GitUserConfig, setup_git_repository,
};
pub(crate) use spec::merge_git_template_spec;
