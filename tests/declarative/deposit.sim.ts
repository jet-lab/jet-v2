import { Connection } from "@solana/web3.js"
import * as os from "os"

import { Replicant } from "./replicant"

import CONFIG from "../../libraries/ts/src/margin/config.json"

import TEST_CONFIG from "./scenarios/deposit.json"

describe("Deposits", () => {
  const config = CONFIG.devnet
  const connection = new Connection(config.url, "processed")

  const replicants: Replicant[] = []

  it("Create users", async () => {
    for (const userConfig of TEST_CONFIG.users) {
      replicants.push(
        await Replicant.create(TEST_CONFIG, os.homedir() + "/.config/solana/" + userConfig.keypair, "devnet", connection)
      )
    }
  })

  it("Fund users", async () => {
    for (const replicant of replicants) {
      await replicant.fundUser()
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
  /*
  */

  it("Close margin accounts", async () => {
    for (const replicant of replicants) {
      await replicant.printAccounts()
      await replicant.closeAccounts()
    }
  })
})
