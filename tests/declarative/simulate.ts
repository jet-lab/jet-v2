#!/usr/bin/env ts-node

import { Account } from "@solana/web3.js"
import * as fs from "fs"
import * as os from "os"

import DEPOSIT_CONFIG from "./scenarios/deposit.json"
import BORROW_CONFIG from "./scenarios/borrow.json"
import { Replicant } from "./replicant"

async function simulate() {
  let replicant = new Replicant(
    DEPOSIT_CONFIG,
    new Account(JSON.parse(fs.readFileSync(os.homedir() + `/.config/solana/id.json`, "utf-8")))
  )
  await replicant.load()
  await replicant.createAccounts()
  await replicant.processDeposits()
  await replicant.printAccounts()
  await replicant.closeAccounts()


  replicant = new Replicant(
    BORROW_CONFIG,
    new Account(JSON.parse(fs.readFileSync(os.homedir() + `/.config/solana/id.json`, "utf-8")))
  )
  await replicant.load()
  await replicant.createAccounts()
  await replicant.processDeposits()
  await replicant.processBorrows()
  await replicant.printAccounts()
  await replicant.closeAccounts()


  await replicant.printAccounts()
}

simulate()
