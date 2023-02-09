<div align="center">
  <img height="170" src="https://293354890-files.gitbook.io/~/files/v0/b/gitbook-legacy-files/o/assets%2F-M_72skN1dye71puMdjs%2F-Miqzl5oK1cXXAkARfER%2F-Mis-yeKp1Krh7JOFzQG%2Fjet_logomark_color.png?alt=media&token=0b8dfc84-37d7-455d-9dfd-7bb59cee5a1a" />

  <h1>Jet V2</h1>

  <p>
    <a target="_blank" href="https://github.com/jet-lab/jet-v2/actions/workflows/check.yml">
      <img alt="Build" src="https://github.com/jet-lab/jet-v2/actions/workflows/check.yml/badge.svg" />
    </a>
    <a target="_blank" href="https://discord.com/channels/880316176612343891">
      <img alt="Discord" src="https://img.shields.io/discord/833805114602291200?color=blueviolet" />
    </a>
    <a target="_blank" href="https://opensource.org/licenses/AGPL-3.0">
      <img alt="License" src="https://img.shields.io/badge/license-AGPL--3.0--or--later-blue" />
    </a>
  </p>

  <h4>
    <a target="_blank" href="https://jetprotocol.io">Webite</a>
    |
    <a target="_blank" href="https://docs.jetprotocol.io">Docs</a>
  </h4>
</div>

# Install

## Solana 

Make sure you update Solana to a newer version.

```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.11.10/install)"
```

Install anchor. Please see the [Anchor Documentation](https://project-serum.github.io/anchor/getting-started/installation.html)

```bash
cargo install --git https://github.com/project-serum/anchor avm --locked --force

avm install 0.24.2
avm use 0.24.2
anchor --version # anchor-cli 0.24.2
```

## Wasm Pack

Install the wasm-pack tool

```bash
cargo install wasm-pack
```

## Yarn

Install the project's node_modules

```bash
yarn
```

# Test

Run the full test suite used by the github CI workflow. This requires all dependencies to be installed:
```bash
./check
```

Run it in a docker container that already contains all the solana and anchor dependencies. This only requires docker:
```bash
./check in-docker
```

Run a single job from the workflow:
```bash
./check [in-docker] [job-name (e.g. e2e-test)]
```

# App

Ensure you have a `/app/.env` file with the required variables:
```
REACT_APP_RPC_TOKEN = <YOUR_RPC_TOKEN>
REACT_APP_RPC_DEV_TOKEN = <YOUR_DEV_RPC_TOKEN>
REACT_APP_IP_REGISTRY = <YOUR_IP_REGISTRY_TOKEN>
REACT_APP_LOGROCKET_PROJECT = ""
```

Run

```bash
yarn
yarn --cwd packages build
yarn dev
```

to run the app.

If `watch` or `wasm-pack` are missing (they should be installed automatically after running `yarn`) Install Cargo dependencies
```bash
cargo install cargo-watch
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

# Documentations
[![Docs](https://img.shields.io/badge/docs-TypeScript-success)](https://jet-lab.github.io/jet-v2/ts-client/)
[![Docs](https://img.shields.io/badge/docs-Rust-success)](https://jet-lab.github.io/jet-v2/margin-rust/jet_margin/)


Developer resources for integrating with Jet Margin Program.
## Margin Program 

> View the [rust docs](https://jet-lab.github.io/jet-v2/margin-rust/jet_margin/) for the full package documentation and available API.
> 

## Margin TypeScript Client

> View the [typedocs](https://jet-lab.github.io/jet-v2/ts-client) for the full package documentation and available API.
> 
> View more [examples](https://github.com/jet-lab/jet-v2/tree/master/tests/integration/examples) for usage reference.
