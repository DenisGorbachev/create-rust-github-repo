use std::io::{stderr, stdout};

use clap::Parser;

use create_rust_github_repo::CreateRustGithubRepo;

fn main() -> anyhow::Result<()> {
    CreateRustGithubRepo::parse().run(&mut stdout(), &mut stderr(), None)
}
