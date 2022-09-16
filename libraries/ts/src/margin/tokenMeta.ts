import { AccountInfo, PublicKey } from "@solana/web3.js"
import BN from "bn.js"
import { findDerivedAccount, Number128 } from "../utils"
import { MarginPrograms } from "./marginClient"
import { TokenKind } from "./metadata"
import { TokenMetaInfo, PositionKind } from "./state"

export class TokenMeta {
  private programs: MarginPrograms
  tokenMint: PublicKey
  address: PublicKey
  info: TokenMetaInfo | undefined

  valueModifier: Number128 = Number128.ZERO
  positionKind: PositionKind = PositionKind.NoValue

  static derive(programs: MarginPrograms, tokenMint: PublicKey) {
    return findDerivedAccount(programs.config.marginProgramId, tokenMint)
  }

  constructor({ programs, tokenMint }: { programs: MarginPrograms; tokenMint: PublicKey }) {
    this.programs = programs
    this.tokenMint = tokenMint
    this.address = TokenMeta.derive(programs, tokenMint)
  }

  static async load(programs: MarginPrograms, tokenMint: PublicKey) {
    const metadata = new TokenMeta({ programs, tokenMint: tokenMint })
    await metadata.refresh()
    return metadata
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
    this.info = this.programs.margin.coder.accounts.decode<TokenMetaInfo>(
      "tokenMeta",
      info.data
    )
    this.valueModifier = Number128.fromDecimal(new BN(this.info.valueModifier), -2)
    this.positionKind = TokenMeta.decodeTokenKind(this.info.positionKind)
  }

  static decodeTokenKind(kind: TokenKind) {
    if ("noValue" in kind) {
      return PositionKind.NoValue
    } else if ("deposit" in kind) {
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
