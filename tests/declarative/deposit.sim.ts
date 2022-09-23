import { Connection } from "@solana/web3.js"
import * as os from "os"

import { MarginConfig, MarginClient } from "@jet-lab/margin"
import { Replicant } from "./replicant"

import TEST_CONFIG from "./scenarios/deposit.json"

describe("Deposits", () => {
  let marginConfig: MarginConfig
  let connection: Connection

  it("Load config", async () => {
    marginConfig = await MarginClient.getConfig("devnet")

    connection = new Connection(marginConfig.url, "processed")
  })

  const replicants: Replicant[] = []

  it("Create users", async () => {
    for (const userConfig of TEST_CONFIG.users) {
      replicants.push(
        await Replicant.create(
          TEST_CONFIG,
          marginConfig,
          os.homedir() + "/.config/solana/" + userConfig.keypair,
          "devnet",
          connection
        )
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

  it("Close margin accounts", async () => {
    for (const replicant of replicants) {
      await replicant.closeAccounts()
    }
  })
})
