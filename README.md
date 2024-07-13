# Create Rust GitHub repo

## Installation

```shell
cargo install create-rust-github-repo
```

## Usage

```shell
Usage: create-rust-github-repo [OPTIONS] --name <NAME> --source <SOURCE> --target <TARGET>

Options:
  -n, --name <NAME>
          Repository name
  -v, --visibility <VISIBILITY>
          Repository visibility [default: private] [possible values: public, private, internal]
  -s, --source <SOURCE>
          Source directory for configuration files
  -t, --target <TARGET>
          Target directory for cloning the repository (must include the repo name)
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
