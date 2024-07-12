use std::env::current_dir;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::Context;
use clap::{value_parser, Parser, ValueEnum};

#[derive(ValueEnum, Default, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum RepoVisibility {
    Public,
    #[default]
    Private,
    Internal,
}

#[derive(Parser, Debug)]
pub struct CreateRustGithubRepo {
    #[arg(long, short, help = "Account name (either user or organization name)")]
    account_name: String,

    #[arg(long, short = 'n', help = "Repository name")]
    repo_name: String,

    #[arg(long, short = 'v', help = "Repository visibility", value_enum, default_value_t)]
    repo_visibility: RepoVisibility,

    #[arg(long, short, help = "Source directory for configuration files", value_parser = value_parser!(PathBuf))]
    source: PathBuf,

    #[arg(long, short, help = "Target directory for cloning the repository (must include the repo name)", value_parser = value_parser!(PathBuf))]
    target: PathBuf,

    #[arg(long, help = "Message for git commit", default_value = "Add configs")]
    git_commit_message: String,

    #[arg(long, help = "Extra config file paths (relative to `source` directory)", value_delimiter = ',')]
    extra_configs: Vec<String>,

    #[arg(long, help = "Forwarded arguments for `gh repo create`", value_delimiter = ' ')]
    gh_repo_create_args: Vec<String>,

    #[arg(long, help = "Forwarded arguments for `git clone`", value_delimiter = ' ')]
    git_clone_args: Vec<String>,

    #[arg(long, help = "Forwarded arguments for `cargo init`", value_delimiter = ' ')]
    cargo_init_args: Vec<String>,

    #[arg(long, help = "Forwarded arguments for `cargo build`", value_delimiter = ' ')]
    cargo_build_args: Vec<String>,

    #[arg(long, help = "Forwarded arguments for `git commit`", value_delimiter = ' ')]
    git_commit_args: Vec<String>,

    #[arg(long, help = "Forwarded arguments for `git push`", value_delimiter = ' ')]
    git_push_args: Vec<String>,
}

impl CreateRustGithubRepo {
    pub fn run(self) -> anyhow::Result<()> {
        // Create a GitHub repo
        exec(
            "gh",
            [
                "repo",
                "create",
                &self.repo_name,
                into_gh_create_repo_flag(self.repo_visibility),
            ],
            self.gh_repo_create_args.into_iter(),
            current_dir()?,
        )
        .context("Failed to create GitHub repository")?;

        // Clone the repo
        exec(
            "git",
            [
                "clone",
                &format!("git@github.com:{}/{}.git", self.account_name, self.repo_name),
                self.target.to_str().unwrap(),
            ],
            self.git_clone_args.into_iter(),
            current_dir()?,
        )
        .context("Failed to clone repository")?;

        // Run cargo init
        exec("cargo", ["init"], self.cargo_init_args.into_iter(), &self.target).context("Failed to initialize Cargo project")?;

        let mut configs: Vec<String> = vec![];
        configs.extend(CONFIGS.iter().copied().map(ToOwned::to_owned));
        configs.extend(self.extra_configs);
        // Copy config files
        copy_configs(&self.source, &self.target, configs).context("Failed to copy configuration files")?;

        // Run cargo build
        exec("cargo", ["build"], self.cargo_build_args.into_iter(), &self.target).context("Failed to build Cargo project")?;

        // Git commit
        exec("git", ["add", "."], Vec::<String>::new().into_iter(), &self.target).context("Failed to stage files for commit")?;

        exec("git", ["commit", "-m", &self.git_commit_message], self.git_commit_args.into_iter(), &self.target).context("Failed to commit changes")?;

        // Git push
        exec("git", ["push"], self.git_push_args.into_iter(), &self.target).context("Failed to push changes")?;

        Ok(())
    }
}

pub fn into_gh_create_repo_flag(repo_visibility: RepoVisibility) -> &'static str {
    match repo_visibility {
        RepoVisibility::Public => "--public",
        RepoVisibility::Private => "--private",
        RepoVisibility::Internal => "--internal",
    }
}

pub fn exec(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item = impl AsRef<OsStr>>, extra_args: impl IntoIterator<Item = impl AsRef<OsStr>>, current_dir: impl AsRef<Path>) -> io::Result<ExitStatus> {
    Command::new(cmd)
        .args(args)
        .args(extra_args)
        .current_dir(current_dir)
        .spawn()?
        .wait()
        .and_then(|status| if status.success() { Ok(status) } else { Err(io::Error::new(io::ErrorKind::Other, format!("Process exited with with status {}", status))) })
}

pub fn copy_configs<P: Clone + AsRef<Path>>(source: &Path, target: &Path, configs: impl IntoIterator<Item = P>) -> io::Result<()> {
    for config in configs {
        let source_path = source.join(config.clone());
        let target_path = target.join(config);
        if source_path.exists() && !target_path.exists() {
            fs_err::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

pub const CONFIGS: &[&str] = &[
    "clippy.toml",
    "rustfmt.toml",
    "Justfile",
    "lefthook.yml",
    ".lefthook.yml",
    "lefthook.yaml",
    ".lefthook.yaml",
    "lefthook.toml",
    ".lefthook.toml",
    "lefthook.json",
    ".lefthook.json",
];
