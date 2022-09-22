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

Make sure you update Solana to the latest version

```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.11.10/install)"
```

Install anchor. Please see the [Anchor Documentation](https://project-serum.github.io/anchor/getting-started/installation.html)

```bash
cargo install --git https://github.com/project-serum/anchor avm --locked --force

avm install latest
avm use latest
anchor --version # anchor-cli 0.25.0
```

Install the project's node_modules

```bash
yarn
```

# Test

Run

```bash
yarn test
```

to run the test suite

# App

Run

```bash
cd app
yarn start
```

to run the app.

You may have to run the app in legacy mode if you get the following error
`error:0308010C:digital envelope routines::unsupported`

```bash
cd app
yarn start:legacy
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
