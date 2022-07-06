import { Account, Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js"
import * as fs from "fs"
import * as os from "os"

import { Replicant, sleep } from "./replicant"

import CONFIG from "../../libraries/ts/src/margin/config.json"

import TEST_CONFIG from "./scenarios/deposit.json"

describe("Deposits", () => {

  const config = CONFIG.devnet;

  const connection = new Connection(config.url, "processed");

  const replicants: Replicant[] = [];

  it("Create users", async () => {
    for (const userConfig of TEST_CONFIG.users) {
      const file = os.homedir() + '/.config/solana/' + userConfig.keypair;
      if (!fs.existsSync(file)) {
        const keypair = Keypair.generate();
        fs.writeFileSync(file, JSON.stringify(Array.from(keypair.secretKey)))
        const airdropSignature = await connection.requestAirdrop(keypair.publicKey, 2 * LAMPORTS_PER_SOL)
        await connection.confirmTransaction(airdropSignature)
        await sleep(8 * 1000)
      }
      replicants.push(new Replicant(TEST_CONFIG, new Account(Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(file).toString()))).secretKey), 'devnet', connection));
    }
  })

  it("Load pools", async () => {
    for (const replicant of replicants) {
      await replicant.loadPools()
    }
  })

  it("Create margin accounts", async () => {
    for (const replicant of replicants) {
      await replicant.createAccounts()
    }
  })

  it("Process deposits", async () => {
    for (const replicant of replicants) {
      await replicant.processDeposits()
    }
  })

  it("Close margin accounts", async () => {
    for (const replicant of replicants) {
      await replicant.closeAccounts()
    }
  })

})
