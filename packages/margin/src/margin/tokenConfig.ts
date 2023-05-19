import { Address, BN, translateAddress } from "@project-serum/anchor"
import { AccountInfo, PublicKey } from "@solana/web3.js"
import { findDerivedAccount } from "../utils"
import { Airspace } from "./airspace"
import { MarginPrograms } from "./marginClient"
import { PositionKind } from "./state"

/**
 * On-chain representation of the [[TokenConfig]]
 */
export interface TokenConfigInfo {
  mint: PublicKey
  underlyingMint: PublicKey
  airspace: PublicKey
  admin: number[]
  tokenKind: number
  valueModifier: BN
  maxStaleness: BN
}

export class TokenConfig {
  private programs: MarginPrograms
  tokenMint: PublicKey
  address: PublicKey
  info: TokenConfigInfo | undefined

  valueModifier: number = 0
  tokenKind: PositionKind = PositionKind.NoValue

  static derive(programs: MarginPrograms, airspace: Address | undefined, tokenMint: PublicKey) {
    airspace = airspace
      ? translateAddress(airspace)
      : Airspace.deriveAddress(programs.airspace.programId, programs.config.airspaces[0].name)
    return findDerivedAccount(programs.config.marginProgramId, "token-config", airspace, tokenMint)
  }

  constructor({
    programs,
    airspace,
    tokenMint
  }: {
    programs: MarginPrograms
    airspace: Address | undefined
    tokenMint: PublicKey
  }) {
    this.programs = programs
    this.tokenMint = tokenMint
    this.address = TokenConfig.derive(programs, airspace, tokenMint)
  }

  static async load(programs: MarginPrograms, airspace: Address | undefined, tokenMint: PublicKey) {
    const config = new TokenConfig({ programs, airspace, tokenMint })
    await config.refresh()
    return config
  }

  async refresh() {
    const info = await this.programs.connection.getAccountInfo(this.address)
    this.decode(info)
  }

  decode(info: AccountInfo<Buffer> | null) {
    if (!info) {
      this.info = undefined
      this.valueModifier = 0
      return
    }
    this.info = this.programs.margin.coder.accounts.decode("TokenConfig", info.data)
    this.valueModifier = this.info!.valueModifier.toNumber() / 100
    this.tokenKind = TokenConfig.decodeTokenKind(this.info!.tokenKind)
  }

  static decodeTokenKind(kind: number) {
    if (kind == 0) {
      return PositionKind.NoValue
    } else if (kind == 1 || kind == 3) {
      return PositionKind.Deposit
    } else if (kind == 2) {
      return PositionKind.Claim
    } else {
      throw new Error("Unrecognized TokenKind: " + kind.toString())
    }
  }
}
