use std::io::{stderr, stdout};

use clap::Parser;

use create_rust_github_repo::CreateRustGithubRepo;

fn main() -> anyhow::Result<()> {
    CreateRustGithubRepo::parse()
        .repo_create_cmd("gh repo create --private {{name}}")
        .project_init_cmd("cargo init --lib")
        .run(&mut stdout(), &mut stderr(), None)
}
