import { Address, BN, translateAddress } from "@project-serum/anchor"
import { AccountInfo, PublicKey } from "@solana/web3.js"
import { findDerivedAccount, Number128 } from "../utils"
import { Airspace } from "./airspace"
import { MarginPrograms } from "./marginClient"
import { TokenKind } from "./metadata"
import { PositionKind } from "./state"

/**
 * On-chain representation of the [[TokenConfig]]
 */
export interface TokenConfigInfo {
  mint: PublicKey
  underlyingMint: PublicKey
  airspace: PublicKey
  admin: number[]
  tokenKind: TokenKind
  valueModifier: BN
  maxStaleness: BN
}

export class TokenConfig {
  private programs: MarginPrograms
  tokenMint: PublicKey
  address: PublicKey
  info: TokenConfigInfo | undefined

  valueModifier: Number128 = Number128.ZERO
  tokenKind: PositionKind = PositionKind.NoValue

  static derive(programs: MarginPrograms, airspace: Address | undefined, tokenMint: PublicKey) {
    airspace = airspace
      ? translateAddress(airspace)
      : Airspace.deriveAddress(programs.airspace.programId, programs.config.airspaces[0].name)
    return findDerivedAccount(programs.config.marginProgramId, "token-config", tokenMint)
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
      this.valueModifier = Number128.ZERO
      return
    }
    this.info = this.programs.margin.coder.accounts.decode<TokenConfigInfo>("TokenConfig", info.data)
    this.valueModifier = Number128.fromDecimal(new BN(this.info.valueModifier), -2)
    this.tokenKind = TokenConfig.decodeTokenKind(this.info.tokenKind)
  }

  static decodeTokenKind(kind: TokenKind) {
    if ("nonCollateral" in kind) {
      return PositionKind.NoValue
    } else if ("collateral" in kind) {
      return PositionKind.Deposit
    } else if ("claim" in kind) {
      return PositionKind.Claim
    } else {
      throw new Error("Unrecognized TokenKind.")
    }
  }

  getLiability(value: Number128) {
    return value
  }

  collateralValue(value: Number128) {
    return this.valueModifier.mul(value)
  }

  requiredCollateralValue(value: Number128) {
    if (this.valueModifier.eq(Number128.ZERO)) {
      // No leverage configured for claim
      return Number128.MAX
    } else {
      return value.div(this.valueModifier)
    }
  }
}
