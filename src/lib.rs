use std::collections::HashMap;
use std::env::current_dir;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::Context;
use clap::{value_parser, Parser, ValueEnum};
use derive_setters::Setters;

#[derive(ValueEnum, Default, Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum RepoVisibility {
    Public,
    #[default]
    Private,
    Internal,
}

impl RepoVisibility {
    pub fn to_gh_create_repo_flag(&self) -> &'static str {
        match self {
            RepoVisibility::Public => "--public",
            RepoVisibility::Private => "--private",
            RepoVisibility::Internal => "--internal",
        }
    }
}

#[derive(Parser, Setters, Debug)]
#[command(version, about, author, after_help = "All command arg options support the following substitutions:\n* {{name}} - substituted with --name arg\n* {{dir}} - substituted with resolved directory for repo (the resolved value of --dir)\n")]
#[setters(into)]
pub struct CreateRustGithubRepo {
    #[arg(long, short = 'n', help = "Repository name")]
    name: String,

    #[arg(long, short, help = "Target directory for cloning the repository (must include the repo name) (defaults to \"{current_dir}/{repo_name}\") (see also: --workspace)", value_parser = value_parser!(PathBuf))]
    dir: Option<PathBuf>,

    #[arg(long, short, help = "Parent of the target directory for cloning the repository (must NOT include the repo name). If this option is specified, then the repo is cloned to \"{workspace}/{repo_name}\". The --dir option overrides this option", value_parser = value_parser!(PathBuf))]
    workspace: Option<PathBuf>,

    #[arg(long, help = "Shell to use for executing commands", default_value = "/bin/sh")]
    shell_cmd: String,

    #[arg(long, short, help = "Source directory for configuration files", value_parser = value_parser!(PathBuf))]
    copy_configs_from: Option<PathBuf>,

    #[arg(long, help = "Extra config file paths (relative to `source` directory)", value_delimiter = ',')]
    extra_configs: Vec<String>,

    #[arg(long, help = "Shell command to check if repo exists (supports substitutions - see help below)", default_value = "gh repo view --json nameWithOwner {{name}} 2>/dev/null")]
    repo_exists_cmd: String,

    #[arg(long, help = "Shell command to create a repo (supports substitutions - see help below)", default_value = "gh repo create --private {{name}}")]
    repo_create_cmd: String,

    #[arg(long, help = "Shell command to clone a repo (supports substitutions - see help below)", default_value = "gh repo clone {{name}} {{dir}}")]
    repo_clone_cmd: String,

    #[arg(long, help = "Shell command to initialize a project (supports substitutions - see help below)", default_value = "cargo init")]
    project_init_cmd: String,

    #[arg(long, help = "Shell command to test a project (supports substitutions - see help below)", default_value = "cargo test")]
    project_test_cmd: String,

    #[arg(long, help = "Shell command to add new files (supports substitutions - see help below)", default_value = "git add .")]
    repo_add_args: String,

    #[arg(long, help = "Shell command to make a commit (supports substitutions - see help below)", default_value = "git commit -m \"Add configs\"")]
    repo_commit_args: String,

    #[arg(long, help = "Shell command to push the commit (supports substitutions - see help below)", default_value = "git push")]
    repo_push_args: String,
}

impl CreateRustGithubRepo {
    pub fn run(self) -> anyhow::Result<()> {
        let current_dir = current_dir()?;
        let dir = self
            .dir
            .or_else(|| self.workspace.map(|workspace| workspace.join(&self.name)))
            .unwrap_or(current_dir.join(&self.name));
        let dir_string = dir.display().to_string();

        let substitutions = HashMap::<&'static str, &str>::from([
            ("{{name}}", self.name.as_str()),
            ("{{dir}}", dir_string.as_str()),
        ]);

        let repo_exists = success(&self.shell_cmd, ["-c"], [self.repo_exists_cmd], &current_dir, &substitutions)?;

        if !repo_exists {
            // Create a GitHub repo
            exec(&self.shell_cmd, ["-c"], [self.repo_create_cmd], &current_dir, &substitutions).context("Failed to create repository")?;
        }

        if !dir.exists() {
            // Clone the repo
            exec(&self.shell_cmd, ["-c"], [self.repo_clone_cmd], &current_dir, &substitutions).context("Failed to clone repository")?;
        } else {
            println!("Directory \"{}\" exists, skipping clone command", dir.display())
        }

        let cargo_toml = dir.join("Cargo.toml");

        if !cargo_toml.exists() {
            // Run cargo init
            exec(&self.shell_cmd, ["-c"], [self.project_init_cmd], &dir, &substitutions).context("Failed to initialize the project")?;
        } else {
            println!("Cargo.toml exists in \"{}\", skipping `cargo init` command", dir.display())
        }

        if let Some(copy_configs_from) = self.copy_configs_from {
            let mut configs: Vec<String> = vec![];
            configs.extend(CONFIGS.iter().copied().map(ToOwned::to_owned));
            configs.extend(self.extra_configs);
            // Copy config files
            copy_configs_if_not_exists(&copy_configs_from, &dir, configs).context("Failed to copy configuration files")?;
        }

        // test
        exec(&self.shell_cmd, ["-c"], [self.project_test_cmd], &dir, &substitutions).context("Failed to test the project")?;

        // add
        exec(&self.shell_cmd, ["-c"], [self.repo_add_args], &dir, &substitutions).context("Failed to add files for commit")?;

        // commit
        exec(&self.shell_cmd, ["-c"], [self.repo_commit_args], &dir, &substitutions).context("Failed to commit changes")?;

        // push
        exec(&self.shell_cmd, ["-c"], [self.repo_push_args], &dir, &substitutions).context("Failed to push changes")?;

        Ok(())
    }
}

pub fn replace_args(args: impl IntoIterator<Item = String>, substitutions: &HashMap<&str, &str>) -> Vec<String> {
    args.into_iter()
        .map(|arg| replace_all(arg, substitutions))
        .collect()
}

pub fn replace_all(mut input: String, substitutions: &HashMap<&str, &str>) -> String {
    for (key, value) in substitutions {
        input = input.replace(key, value);
    }
    input
}

pub fn exec(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item = impl AsRef<OsStr>>, extra_args: impl IntoIterator<Item = String>, current_dir: impl AsRef<Path>, substitutions: &HashMap<&str, &str>) -> io::Result<ExitStatus> {
    let replacements = replace_args(extra_args, substitutions);
    let extra_args = replacements.iter().map(AsRef::<OsStr>::as_ref);
    exec_raw(cmd, args, extra_args, current_dir)
}

pub fn success(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item = impl AsRef<OsStr>>, extra_args: impl IntoIterator<Item = String>, current_dir: impl AsRef<Path>, substitutions: &HashMap<&str, &str>) -> io::Result<bool> {
    let replacements = replace_args(extra_args, substitutions);
    let extra_args = replacements.iter().map(AsRef::<OsStr>::as_ref);
    success_raw(cmd, args, extra_args, current_dir)
}

pub fn exec_raw(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item = impl AsRef<OsStr>>, extra_args: impl IntoIterator<Item = impl AsRef<OsStr>>, current_dir: impl AsRef<Path>) -> io::Result<ExitStatus> {
    get_status_raw(cmd, args, extra_args, current_dir).and_then(check_status)
}

pub fn success_raw(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item = impl AsRef<OsStr>>, extra_args: impl IntoIterator<Item = impl AsRef<OsStr>>, current_dir: impl AsRef<Path>) -> io::Result<bool> {
    get_status_raw(cmd, args, extra_args, current_dir).map(|status| status.success())
}

pub fn get_status_raw(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item = impl AsRef<OsStr>>, extra_args: impl IntoIterator<Item = impl AsRef<OsStr>>, current_dir: impl AsRef<Path>) -> io::Result<ExitStatus> {
    Command::new(cmd)
        .args(args)
        .args(extra_args)
        .current_dir(current_dir)
        .spawn()?
        .wait()
}

pub fn check_status(status: ExitStatus) -> io::Result<ExitStatus> {
    if status.success() {
        Ok(status)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, format!("Process exited with with status {}", status)))
    }
}

pub fn copy_configs_if_not_exists<P: Clone + AsRef<Path>>(source: &Path, target: &Path, configs: impl IntoIterator<Item = P>) -> io::Result<()> {
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

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    CreateRustGithubRepo::command().debug_assert();
}
