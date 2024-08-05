//! # Overview
//!
//! `create-rust-github-repo` is a CLI program that creates a new repository on GitHub, clones it locally, initializes a Rust project, copies the configs from a pre-existing directory.
//!
//! # Examples
//!
//! ```shell,ignore
//! # Create a GitHub repo & init a Rust project
//! create-rust-github-repo --name my-new-project
//!
//! # Copy configs from existing project
//! create-rust-github-repo --name my-new-project --copy-configs-from ~/workspace/my-existing-project --configs .github,rustfmt.toml,clippy.toml
//!
//! # Clone to a specific directory
//! create-rust-github-repo --name my-new-project --dir ~/workspace/my-new-project
//!
//! # Create a public repo
//! create-rust-github-repo --name my-new-project --repo-create-cmd "gh repo create --public {{name}}"
//!
//! # Create a lib instead of bin
//! create-rust-github-repo --name my-new-project --project-init-cmd "cargo init --lib"
//! ```
//!
//! # Features
//!
//! * ✅ Uses existing `gh`, `git`, `cargo` commands
//! * ✅ Supports overrides for all commands
//! * ✅ Supports substitutions (see help below)
//! * ✅ Can be used as a library

use std::collections::HashMap;
use std::env::{current_dir, current_exe};
use std::ffi::{OsStr, OsString};
use std::fs::create_dir_all;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context};
use clap::{value_parser, Parser};
use derive_new::new;
use derive_setters::Setters;
use fs_extra::{dir, file};

#[derive(Parser, Setters, Default, Debug)]
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
    shell_cmd: OsString,

    #[arg(long, help = "Shell args to use for executing commands (note that '-c' is always passed as last arg)")]
    shell_args: Vec<OsString>,

    #[arg(long, short, help = "Source directory for config paths", value_parser = value_parser!(PathBuf))]
    copy_configs_from: Option<PathBuf>,

    /// Config paths separated by comma (relative to `copy_configs_from`) (only applies if `copy_configs_from` is specified) (supports files and directories)
    #[arg(long, value_delimiter = ',')]
    configs: Vec<String>,

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

    #[arg(long, help = "Shell command to make a commit (supports substitutions - see help below)", default_value = "git commit -m \"feat: setup project\"")]
    repo_commit_args: String,

    #[arg(long, help = "Shell command to push the commit (supports substitutions - see help below)", default_value = "git push")]
    repo_push_args: String,

    /// The probability of seeing a support link in a single execution of the command is `1 / {{this-field-value}}`.
    ///
    /// Set it to 0 to disable the support link.
    #[arg(long, short = 's', env, default_value_t = 1)]
    support_link_probability: u64,

    /// Don't actually execute commands that modify the data, only print them (note that read-only commands will still be executed)
    #[arg(long)]
    dry_run: bool,
}

impl CreateRustGithubRepo {
    pub fn run(self, stdout: &mut impl Write, stderr: &mut impl Write, now: Option<u64>) -> anyhow::Result<()> {
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

        let shell = Shell::new(self.shell_cmd, self.shell_args);
        let executor = Executor::new(shell, self.dry_run);

        let repo_exists = executor
            .is_success(replace_all(self.repo_exists_cmd, &substitutions), &current_dir, stderr)
            .context("Failed to find out if repository exists")?;

        if !repo_exists {
            // Create a GitHub repo
            executor
                .exec(replace_all(self.repo_create_cmd, &substitutions), &current_dir, stderr)
                .context("Failed to create repository")?;
        }

        if !dir.exists() {
            // Clone the repo
            executor
                .exec(replace_all(self.repo_clone_cmd, &substitutions), &current_dir, stderr)
                .context("Failed to clone repository")?;
        } else {
            writeln!(stdout, "Directory \"{}\" exists, skipping clone command", dir.display())?;
        }

        let cargo_toml = dir.join("Cargo.toml");

        if !cargo_toml.exists() {
            // Run cargo init
            executor
                .exec(replace_all(self.project_init_cmd, &substitutions), &dir, stderr)
                .context("Failed to initialize the project")?;
        } else {
            writeln!(stdout, "Cargo.toml exists in \"{}\", skipping `cargo init` command", dir.display())?;
        }

        if let Some(copy_configs_from) = self.copy_configs_from {
            let non_empty_configs = self.configs.iter().filter(|s| !s.is_empty());

            for config in non_empty_configs {
                let source = copy_configs_from.join(config);
                let target = dir.join(config);

                if !self.dry_run {
                    if source.exists() && !target.exists() {
                        writeln!(stderr, "[INFO] Copying {} to {}", source.display(), target.display())?;
                        let parent = target
                            .parent()
                            .ok_or(anyhow!("Could not find parent of {}", source.display()))?;
                        create_dir_all(parent)?;
                        if source.is_file() {
                            let options = file::CopyOptions::new()
                                .skip_exist(true)
                                .buffer_size(MEGABYTE);
                            file::copy(&source, &target, &options)?;
                        } else {
                            let options = dir::CopyOptions::new()
                                .skip_exist(true)
                                .copy_inside(true)
                                .buffer_size(MEGABYTE);
                            dir::copy(&source, &target, &options)?;
                        }
                    } else {
                        writeln!(stderr, "[INFO] Skipping {} because {} exists", source.display(), target.display())?;
                    }
                } else {
                    writeln!(stderr, "[INFO] Would copy {} to {}", source.display(), target.display())?;
                }
            }
        }

        // test
        executor
            .exec(replace_all(self.project_test_cmd, &substitutions), &dir, stderr)
            .context("Failed to test the project")?;

        // add
        executor
            .exec(replace_all(self.repo_add_args, &substitutions), &dir, stderr)
            .context("Failed to add files for commit")?;

        // commit
        executor
            .exec(replace_all(self.repo_commit_args, &substitutions), &dir, stderr)
            .context("Failed to commit changes")?;

        // push
        executor
            .exec(replace_all(self.repo_push_args, &substitutions), &dir, stderr)
            .context("Failed to push changes")?;

        let timestamp = now.unwrap_or_else(get_unix_timestamp_or_zero);

        if self.support_link_probability != 0 && timestamp % self.support_link_probability == 0 {
            if let Some(new_issue_url) = get_new_issue_url(CARGO_PKG_REPOSITORY) {
                let exe_name = get_current_exe_name()
                    .and_then(|name| name.into_string().ok())
                    .unwrap_or_else(|| String::from("this program"));
                let option_name = get_option_name_from_field_name(SUPPORT_LINK_FIELD_NAME);
                let thank_you = format!("Thank you for using {exe_name}!");
                let can_we_make_it_better = "Can we make it better for you?";
                let open_issue = format!("Open an issue at {new_issue_url}");
                let newline = "";
                display_message_box(
                    &[
                        newline,
                        &thank_you,
                        newline,
                        can_we_make_it_better,
                        &open_issue,
                        newline,
                    ],
                    stderr,
                )?;
                writeln!(stderr, "The message above can be disabled with {option_name} option")?;
            }
        }

        Ok(())
    }
}

fn display_message_box(lines: &[&str], writer: &mut impl Write) -> io::Result<()> {
    if lines.is_empty() {
        return Ok(());
    }

    let width = lines.iter().map(|s| s.len()).max().unwrap_or(0) + 4;
    let border = "+".repeat(width);

    writeln!(writer, "{}", border)?;

    for message in lines {
        let padding = width - message.len() - 4;
        writeln!(writer, "+ {}{} +", message, " ".repeat(padding))?;
    }

    writeln!(writer, "{}", border)?;
    Ok(())
}

/// This function may return 0 on error
fn get_unix_timestamp_or_zero() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(new, Eq, PartialEq, Clone, Debug)]
pub struct Shell {
    cmd: OsString,
    args: Vec<OsString>,
}

impl Shell {
    pub fn spawn_and_wait(&self, command: impl AsRef<OsStr>, current_dir: impl AsRef<Path>) -> io::Result<ExitStatus> {
        Command::new(&self.cmd)
            .args(&self.args)
            .arg("-c")
            .arg(command)
            .current_dir(current_dir)
            .spawn()?
            .wait()
    }

    pub fn exec(&self, command: impl AsRef<OsStr>, current_dir: impl AsRef<Path>) -> io::Result<ExitStatus> {
        self.spawn_and_wait(command, current_dir)
            .and_then(check_status)
    }

    pub fn is_success(&self, command: impl AsRef<OsStr>, current_dir: impl AsRef<Path>) -> io::Result<bool> {
        self.spawn_and_wait(command, current_dir)
            .map(|status| status.success())
    }
}

#[derive(new, Eq, PartialEq, Clone, Debug)]
pub struct Executor {
    shell: Shell,
    dry_run: bool,
}

impl Executor {
    pub fn exec(&self, command: impl AsRef<OsStr>, current_dir: impl AsRef<Path>, stderr: &mut impl Write) -> io::Result<Option<ExitStatus>> {
        writeln!(stderr, "$ {}", command.as_ref().to_string_lossy())?;
        if self.dry_run {
            Ok(None)
        } else {
            self.shell.exec(command, current_dir).map(Some)
        }
    }

    pub fn is_success(&self, command: impl AsRef<OsStr>, current_dir: impl AsRef<Path>, stderr: &mut impl Write) -> io::Result<bool> {
        writeln!(stderr, "$ {}", command.as_ref().to_string_lossy())?;
        self.shell.is_success(command, current_dir)
    }
}

fn get_new_issue_url(repo_url: &str) -> Option<String> {
    if repo_url.starts_with("https://github.com/") {
        Some(repo_url.to_string() + "/issues/new")
    } else {
        None
    }
}

fn get_option_name_from_field_name(field_name: &str) -> String {
    let field_name = field_name.replace('_', "-");
    format!("--{}", field_name)
}

fn get_current_exe_name() -> Option<OsString> {
    current_exe()
        .map(|exe| exe.file_name().map(OsStr::to_owned))
        .unwrap_or_default()
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

// fn cmd_to_string(cmd: impl AsRef<OsStr>, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> String {
//     let mut cmd_str = cmd.as_ref().to_string_lossy().to_string();
//     for arg in args {
//         cmd_str.push(' ');
//         cmd_str.push_str(arg.as_ref().to_string_lossy().as_ref());
//     }
//     cmd_str
// }

fn check_status(status: ExitStatus) -> io::Result<ExitStatus> {
    if status.success() {
        Ok(status)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, format!("Process exited with with status {}", status)))
    }
}

const CARGO_PKG_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const SUPPORT_LINK_FIELD_NAME: &str = "support_link_probability";
const MEGABYTE: usize = 1048576;

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        CreateRustGithubRepo::command().debug_assert();
    }

    #[cfg(test)]
    macro_rules! test_support_link_probability_name {
        ($field:ident) => {
            let cmd = CreateRustGithubRepo::default();
            cmd.$field(0u64);
            assert_eq!(stringify!($field), SUPPORT_LINK_FIELD_NAME);
        };
    }

    #[test]
    fn test_support_link_probability_name() {
        test_support_link_probability_name!(support_link_probability);
    }

    #[test]
    fn test_support_link() {
        let mut stdout = Cursor::new(Vec::new());
        let mut stderr = Cursor::new(Vec::new());
        let cmd = get_dry_cmd().support_link_probability(1u64);
        cmd.run(&mut stdout, &mut stderr, Some(0)).unwrap();
        let stderr_string = String::from_utf8(stderr.into_inner()).unwrap();
        assert!(stderr_string.contains("Open an issue"))
    }

    fn get_dry_cmd() -> CreateRustGithubRepo {
        CreateRustGithubRepo::default()
            .name("test")
            .shell_cmd("/bin/sh")
            .repo_exists_cmd("echo")
            .dry_run(true)
    }
}
