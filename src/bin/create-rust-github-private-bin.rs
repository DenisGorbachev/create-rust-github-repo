use clap::Parser;

use create_rust_github_repo::CreateRustGithubRepo;

fn main() -> anyhow::Result<()> {
    CreateRustGithubRepo::parse()
        .project_init_cmd("gh repo create --private {{name}}")
        .project_init_cmd("cargo init --bin")
        .run()
}
