# Create Rust GitHub repo

## Overview

`create-rust-github-repo` is a CLI program that creates a new repository on GitHub, clones it locally, initializes a Rust project, copies the configs from a pre-existing directory.

## Examples

```shell
# Create a GitHub repo & init a Rust project
create-rust-github-repo --name my-new-project
 
# Copy configs from existing project
create-rust-github-repo --name my-new-project --copy-configs-from ~/workspace/my-existing-project

# Clone to a specific directory
create-rust-github-repo --name my-new-project --dir ~/workspace/my-new-project

# Create a public repo
create-rust-github-repo --name my-new-project --public

# Create a lib instead of bin
create-rust-github-repo --name my-new-project --cargo-init-args '--lib'
```

## Features

* Uses existing `gh`, `git`, `cargo` commands
* Forwards the flags to commands
* Can be used as a library

## Installation

```shell
cargo install create-rust-github-repo
```

## Usage

```shell
Usage: create-rust-github-repo [OPTIONS] --name <NAME>

Options:
  -n, --name <NAME>
          Repository name
  -d, --dir <DIR>
          Target directory for cloning the repository (must include the repo name) (defaults to "{current_dir}/{repo_name}")
  -v, --visibility <VISIBILITY>
          Repository visibility [default: private] [possible values: public, private, internal]
  -c, --copy-configs-from <COPY_CONFIGS_FROM>
          Source directory for configuration files
      --git-commit-message <GIT_COMMIT_MESSAGE>
          Message for git commit [default: "Add configs"]
      --extra-configs <EXTRA_CONFIGS>
          Extra config file paths (relative to `source` directory)
      --gh-repo-create-args <GH_REPO_CREATE_ARGS>
          Forwarded arguments for `gh repo create`
      --gh-repo-clone-args <GH_REPO_CLONE_ARGS>
          Forwarded arguments for `gh repo clone`
      --cargo-init-args <CARGO_INIT_ARGS>
          Forwarded arguments for `cargo init`
      --cargo-build-args <CARGO_BUILD_ARGS>
          Forwarded arguments for `cargo build`
      --git-commit-args <GIT_COMMIT_ARGS>
          Forwarded arguments for `git commit`
      --git-push-args <GIT_PUSH_ARGS>
          Forwarded arguments for `git push`
  -h, --help
          Print help
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
