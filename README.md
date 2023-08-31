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

# Jet Protocol v2

This repository contains the source code for the implementation of Jet Protocol v2 to run on the Solana network,
and associated tools for using the protocol (like a web frontend). The protocol allows for users to participate in 
non-custodial borrowing and lending marketplaces.

## Status

The protocol is currently under active development, and all APIs are subject to change.

## Documentation

Auto-generated API docs are available [here](https://jet-lab.github.io/jet-v2/)

## Getting Started

Install yarn, anchor and the Solana CLI ([instructions](https://www.anchor-lang.com/docs/installation))

### Wasm Pack

To run the frontend web application also requires wasm-pack, which can be installed with `cargo`:

```bash
cargo install wasm-pack --locked
```

### Test

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

### Web App

Ensure you have a `/app/.env` file with the required variables:
```
REACT_APP_RPC_TOKEN = <YOUR_RPC_TOKEN>
REACT_APP_RPC_DEV_TOKEN = <YOUR_DEV_RPC_TOKEN>
REACT_APP_IP_REGISTRY = <YOUR_IP_REGISTRY_TOKEN>
REACT_APP_LOGROCKET_PROJECT = ""
```

To run the app:

```bash
yarn
yarn --cwd packages build
yarn dev
```