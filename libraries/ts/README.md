<div align="center">
  <h1>@jet-lab/margin</h1>

[![Version](https://img.shields.io/npm/v/@jet-lab/margin?color=red)](https://www.npmjs.com/package/@jet-lab/margin/)
[![Docs](https://img.shields.io/badge/doc-typedocs-success)](https://jet-lab.github.io/margin/)
[![Discord](https://img.shields.io/discord/833805114602291200?color=blueviolet)](https://discord.gg/RW2hsqwfej)
[![License](https://img.shields.io/github/license/jet-lab/jet-v2?color=blue)](./LICENSE)

</div>

## Install

Add your preferred library to your project.

with `npm`

```bash
$ npm i @jet-lab/margin
```

...or with `yarn`

```bash
$ yarn add @jet-lab/margin
```

## Usage

> View the [typedocs](https://jet-lab.github.io/margin/) for the full package documentation and available API.
> 
> View more [examples](https://github.com/jet-lab/jet-v2/tree/master/tests/integration/examples) for usage reference.


### Instantiate the Client 
Loading first margin account if local wallet created the first margin account

```ts
import { MarginAccount, MarginClient } from "@jet-lab/margin"
import { AnchorProvider, Wallet } from "@project-serum/anchor"
import { Connection, Keypair } from "@solana/web3.js"

const connection = new Connection("https://api.devnet.solana.com", "recent")
const options = AnchorProvider.defaultOptions()
const wallet = Wallet.local()
const provider = new AnchorProvider(connection, wallet, options)

const programs = MarginClient.getPrograms(provider, "devnet")
const marginAccount = await MarginAccount.load(programs, provider, wallet.publicKey, 0)

```

###  Monitoring Margin Account Health
Loading margin accounts and getting a margin account's risk indicator

```ts
import { MarginAccount, MarginClient, PoolManager } from "@jet-lab/margin"
import { Connection } from "@solana/web3.js"
import { AnchorProvider, Wallet } from "@project-serum/anchor"

const config = await MarginClient.getConfig("devnet")
const connection = new Connection("https://api.devnet.solana.com", "recent")
const options = AnchorProvider.defaultOptions()
const wallet = Wallet.local()
const localWalletPubkey = wallet.publicKey
const provider = new AnchorProvider(connection, wallet, options)

const programs = MarginClient.getPrograms(provider, config)

const poolManager = new PoolManager(programs, provider)
//Load margin pools
const pools = await poolManager.loadAll()

//Load wallet tokens
const walletTokens = await MarginAccount.loadTokens(poolManager.programs, localWalletPubkey)

//Load all margin accounts - users can have multiple margin accounts eventually
const marginAccounts = await MarginAccount.loadAllByOwner({
  programs: poolManager.programs,
  provider: poolManager.provider,
  pools,
  walletTokens,
  owner: localWalletPubkey
})

//Print risk level of a margin account
if (marginAccounts) {
  console.log(
    `Public key ${localWalletPubkey} risk indicator is ${marginAccounts[0].riskIndicator}`
  )
} else {
  console.log("We have trouble getting margin accounts")
}
