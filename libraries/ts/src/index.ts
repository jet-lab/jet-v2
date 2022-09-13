/*
 * Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

export * from "./margin"
export * from "./token"
export * from "./types"
export * from "./utils"


// import { PublicKey } from "@solana/web3.js"
// import { MarginAccount, MarginClient,PoolManager} from "./margin"
// import {AnchorProvider, Wallet} from '@project-serum/anchor'
// import {Connection, Keypair} from '@solana/web3.js'


// let pubkey = new PublicKey('8oPT9UsUkW7zHqZzGnx1BuSKd4JHCEhWEXXbJkbknouh')


// let connection = new Connection('https://jetprot-main-0d7b.mainnet.rpcpool.com/cad6ce6e-2bbf-4a77-bea5-3a30d03ad0e9')
// const walletKepair = Keypair.generate()
// const walletPubkey = walletKepair.publicKey



// const options = AnchorProvider.defaultOptions()
// const wallet = new Wallet(walletKepair)
// const provider = new AnchorProvider(connection, wallet, options)


// async function main(){

//     // Load programs
// let config = await MarginClient.getConfig("mainnet-beta")
// let programs = MarginClient.getPrograms(provider, config)

// const poolManager = new PoolManager(programs, provider);
// const tokenPools = await poolManager.loadAll();

// const mints: any = {};
// for (const pool of Object.values(tokenPools)) {
//   mints[pool.symbol] = {
//     tokenMint: pool.addresses.tokenMint,
//     depositNoteMint: pool.addresses.depositNoteMint,
//     loanNoteMint: pool.addresses.loanNoteMint
//   };
// }

// const transactions = await MarginClient.getTransactionHistory(
//     provider,
//     new PublicKey('CoAFgUnxRRcMm6HFcaN1VEJtn3y2ACL92AynFKLWaDLX'),
//     mints,
//     'mainnet-beta'
//   );

// }


