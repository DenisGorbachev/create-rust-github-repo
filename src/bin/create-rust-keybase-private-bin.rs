use clap::Parser;

use create_rust_github_repo::CreateRustGithubRepo;

fn main() -> anyhow::Result<()> {
    CreateRustGithubRepo::parse()
        .repo_exists_cmd("keybase git list | grep \" {{name}} \"")
        .repo_create_cmd("keybase git create {{name}}")
        .repo_clone_cmd("git clone $(keybase git list | grep \" {{name}} \" | awk '{print $2}') {{dir}}")
        .project_init_cmd("cargo init --bin")
        .run()
}
