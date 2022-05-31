import camelCase from "camelcase"
import { AccountsCoder, BorshAccountsCoder, BorshInstructionCoder, Idl, InstructionCoder } from "@project-serum/anchor"
import { Commitment, Connection, PublicKey, TransactionInstruction } from "@solana/web3.js"
import { IdlAccountDef, IdlInstruction } from "@project-serum/anchor/dist/cjs/idl"

export function buildAccounts(IDL: Idl) {
  const accountCoder: AccountsCoder = new BorshAccountsCoder(IDL)

  const accounts = {}

  IDL.accounts?.forEach(idlAccount => {
    accounts[camelCase(idlAccount.name)] = new AccountClient(idlAccount, accountCoder)
  })

  return accounts
}

class AccountClient {
  constructor(private _idlAccount: IdlAccountDef, private _coder: AccountsCoder) {}

  /**
   * Returns a deserialized account, returning null if it doesn't exist.
   *
   * @param address The address of the account to fetch.
   */
  async fetchNullable(connection: Connection, address: PublicKey, commitment?: Commitment) {
    const accountInfo = await this.getAccountInfo(connection, address, commitment)
    if (accountInfo === null) {
      return null
    }
    return this._coder.decode(this._idlAccount.name, accountInfo.data)
  }

  /**
   * Returns a deserialized account.
   *
   * @param address The address of the account to fetch.
   */
  async fetch(connection: Connection, address: PublicKey, commitment?: Commitment) {
    const data = await this.fetchNullable(connection, address, commitment)
    if (data === null) {
      throw new Error(`Account does not exist ${address.toString()}`)
    }
    return data
  }

  async getAccountInfo(connection: Connection, address: PublicKey, commitment?: Commitment) {
    return await connection.getAccountInfo(translateAddress(address), commitment)
  }
}

export function buildInstructions(IDL: Idl, programId: PublicKey) {
  const instructionCoder: InstructionCoder = new BorshInstructionCoder(IDL)
  const instructions = {}
  IDL.instructions.forEach(idlIx => {
    instructions[idlIx.name] = buildInstruction(
      idlIx,
      (ixName: string, ix: any) => instructionCoder.encode(ixName, ix),
      programId
    )
  })
  return instructions
}

export function buildInstruction(idlIx: IdlInstruction, encodeFn, programId: PublicKey) {
  const ix = (...args): TransactionInstruction => {
    const [ixArgs, ctx] = splitArgsAndCtx(idlIx, [...args])
    validateAccounts(idlIx.accounts, ctx.accounts)
    //validateInstruction(idlIx, ...args);

    const keys = ix.accounts(ctx.accounts)

    if (ctx.remainingAccounts !== undefined) {
      keys.push(...ctx.remainingAccounts)
    }

    return new TransactionInstruction({
      keys,
      programId,
      data: encodeFn(idlIx.name, toInstruction(idlIx, ...ixArgs))
    })
  }

  // Utility fn for ordering the accounts for this instruction.
  ix["accounts"] = accs => {
    return accountsArray(accs, idlIx.accounts, idlIx.name)
  }

  return ix
}

function accountsArray(ctx, accounts, ixName?: string) {
  if (!ctx) {
    return []
  }

  return accounts
    .map(acc => {
      // Nested accounts.
      const nestedAccounts = "accounts" in acc ? acc.accounts : undefined
      if (nestedAccounts !== undefined) {
        const rpcAccs = ctx[acc.name]
        return accountsArray(rpcAccs, acc.accounts, ixName).flat()
      } else {
        const account = acc
        let pubkey
        try {
          pubkey = translateAddress(ctx[acc.name])
        } catch (err) {
          throw new Error(
            `Wrong input type for account "${acc.name}" in the instruction accounts object${
              ixName !== undefined ? ' for instruction "' + ixName + '"' : ""
            }. Expected PublicKey or string.`
          )
        }
        return {
          pubkey,
          isWritable: account.isMut,
          isSigner: account.isSigner
        }
      }
    })
    .flat()
}

function splitArgsAndCtx(idlIx, args) {
  let options = {}

  const inputLen = idlIx.args ? idlIx.args.length : 0
  if (args.length > inputLen) {
    if (args.length !== inputLen + 1) {
      throw new Error(
        `provided too many arguments ${args} to instruction ${idlIx?.name} expecting: ${
          idlIx.args?.map(a => a.name) ?? []
        }`
      )
    }
    options = args.pop()
  }

  return [args, options]
}

// Allow either IdLInstruction or IdlStateMethod since the types share fields.
function toInstruction(idlIx, ...args: any[]) {
  if (idlIx.args.length != args.length) {
    throw new Error("Invalid argument length")
  }
  const ix: { [key: string]: any } = {}
  let idx = 0
  idlIx.args.forEach(ixArg => {
    ix[ixArg.name] = args[idx]
    idx += 1
  })

  return ix
}

// Translates an address to a Pubkey.
function translateAddress(address): PublicKey {
  return address instanceof PublicKey ? address : new PublicKey(address)
}

function validateAccounts(ixAccounts, accounts) {
  ixAccounts.forEach(acc => {
    if ("accounts" in acc) {
      validateAccounts(acc.accounts, accounts[acc.name])
    } else {
      if (accounts[acc.name] === undefined) {
        throw new Error(`Invalid arguments: ${acc.name} not provided.`)
      }
    }
  })
}
