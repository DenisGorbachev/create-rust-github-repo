use clap::Parser;

use create_rust_github_repo::CreateRustGithubRepo;

fn main() -> anyhow::Result<()> {
    CreateRustGithubRepo::parse()
        .repo_create_cmd("gh repo create --public {{name}}")
        .project_init_cmd("cargo init --bin")
        .run()
}
