use std::io::{stderr, stdout};

use clap::Parser;

use create_rust_github_repo::{set_keybase_defaults, CreateRustGithubRepo};

fn main() -> anyhow::Result<()> {
    set_keybase_defaults(CreateRustGithubRepo::parse())
        .project_init_cmd("cargo init --lib")
        .run(&mut stdout(), &mut stderr(), None)
}
