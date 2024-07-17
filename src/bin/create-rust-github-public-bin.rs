use clap::Parser;
use create_rust_github_repo::{CreateRustGithubRepo, RepoVisibility};

fn main() -> anyhow::Result<()> {
    CreateRustGithubRepo::parse()
        .visibility(RepoVisibility::Public)
        .cargo_init_args(["--bin".to_string()])
        .run()
}
