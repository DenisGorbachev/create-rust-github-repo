use clap::Parser;
use create_rust_github_repo::{CreateRustGithubRepo, RepoVisibility};

fn main() -> anyhow::Result<()> {
    CreateRustGithubRepo::parse()
        .visibility(RepoVisibility::Private)
        .cargo_init_args(["--lib".to_string()])
        .run()
}
