#!/usr/bin/env -S deno run --allow-read --allow-run=git,cargo --allow-env=DENOEXEC_STRING_LITERAL_DEBUG

import * as toml from "jsr:@std/toml"
import { $, _ } from "https://deno.land/x/denoexec@v1.1.5/mod.ts";

interface CargoToml {
  package: {
    name: string
  }
}

const dirname = import.meta.dirname
const decoder = new TextDecoder();
const theCargoTomlText = await Deno.readTextFile(`${dirname}/Cargo.toml`)
const theCargoToml = toml.parse(theCargoTomlText) as unknown as CargoToml
const packageName = theCargoToml.package.name
const bin = packageName
const help = await new Deno.Command("cargo", {
  args: ['run', '--', '--help']
}).output();
if (!help.success) throw help
const repo = await getGitHubRepo()

async function getGitHubRepo() {
  const url = (await $(_`git remote get-url origin`)).trim();
  const match = url.match(/github\.com[:\/]([^.]+)/);
  if (match) {
    return match[1]
  } else {
    throw new Error(`Could not extract org/repo from ${url}`)
  }
}

const autogenerated = `
<!-- DO NOT EDIT -->
<!-- This file is automatically generated by README.ts. -->
<!-- Edit README.ts if you want to make changes. -->
`.trim()

console.info(`
${autogenerated}

# Create Rust GitHub repo

[![Build](https://github.com/${repo}/actions/workflows/ci.yml/badge.svg)](https://github.com/${repo})
[![Documentation](https://docs.rs/${packageName}/badge.svg)](https://docs.rs/${packageName})

## Overview

\`${bin}\` is a CLI program that creates a new repository on GitHub, clones it locally, initializes a Rust project, copies the configs from a pre-existing directory.

## Examples

\`\`\`shell
# Create a GitHub repo & init a Rust project
${bin} --name my-new-project
 
# Copy configs from existing project
${bin} --name my-new-project --copy-configs-from ~/workspace/my-existing-project

# Clone to a specific directory
${bin} --name my-new-project --dir ~/workspace/my-new-project

# Create a public repo
${bin} --name my-new-project --public

# Create a lib instead of bin
${bin} --name my-new-project --cargo-init-args '--lib'
\`\`\`

## Features

* Uses existing \`gh\`, \`git\`, \`cargo\` commands
* Forwards the flags to commands
* Can be used as a library

## Installation

\`\`\`shell
cargo install ${packageName}
\`\`\`

## Usage

\`\`\`shell
${decoder.decode(help.stdout).trim()}
\`\`\`

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
`.trim())