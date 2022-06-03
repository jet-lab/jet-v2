<div align="center">
  <h1>@jet-lab/margin</h1>

[![Version](https://img.shields.io/npm/v/@jet-lab/margin?color=red)](https://www.npmjs.com/package/@jet-lab/margin/)
[![Docs](https://img.shields.io/badge/doc-typedocs-success)](https://jet-lab.github.io/margin/)
[![Discord](https://img.shields.io/discord/833805114602291200?color=blueviolet)](https://discord.gg/RW2hsqwfej)
[![License](https://img.shields.io/github/license/jet-lab/jet-v2?color=blue)](./LICENSE)

</div>

## Install

Add the package as a dependency to your project:

```bash
$ npm i @jet-lab/margin
```

...or with `yarn`

```bash
$ yarn add @jet-lab/margin
```

## Usage

> View the [typedocs](https://jet-lab.github.io/margin/) for the full package documentation and available API.

### Instantiate the Client

```ts
import { clusterApiUrl, Connection, Keypair } from "@solana/web3.js"
import { MarginAccount, MarginClient } from "@jet-lab/margin"
import { AnchorProvider, Wallet } from "@project-serum/anchor"

const provider = new AnchorProvider(
  new Connection(clusterApiUrl("devnet")),
  new Wallet(Keypair.generate()),
  AnchorProvider.defaultOptions()
)
const programs = MarginClient.getPrograms(provider, "devnet")
const marginAccount = MarginAccount.load(programs, provider, "A4aVtbwfHDX2RsGyoLMe7jQqeTuFmMhGTqHYNUZ9cpB5", 0)
```
