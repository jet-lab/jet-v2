import { AccountInfo, PublicKey } from "@solana/web3.js"
import BN from "bn.js"
import { findDerivedAccount, Number128 } from "../utils"
import { MarginPrograms } from "./marginClient"
import { PositionTokenMetadataInfo, TokenKind } from "./metadata"
import { PositionKind } from "./state"

export class PositionTokenMetadata {
  private programs: MarginPrograms
  tokenMint: PublicKey
  address: PublicKey
  info: PositionTokenMetadataInfo | undefined

  valueModifier: Number128 = Number128.ZERO
  tokenKind: PositionKind = PositionKind.NoValue

  static derive(programs: MarginPrograms, tokenMint: PublicKey) {
    return findDerivedAccount(programs.config.metadataProgramId, tokenMint)
  }

  constructor({ programs, tokenMint }: { programs: MarginPrograms; tokenMint: PublicKey }) {
    this.programs = programs
    this.tokenMint = tokenMint
    this.address = PositionTokenMetadata.derive(programs, tokenMint)
  }

  static async load(programs: MarginPrograms, tokenMint: PublicKey) {
    const metadata = new PositionTokenMetadata({ programs, tokenMint: tokenMint })
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
    this.info = this.programs.metadata.coder.accounts.decode<PositionTokenMetadataInfo>(
      "positionTokenMetadata",
      info.data
    )
    this.valueModifier = Number128.fromDecimal(new BN(this.info.valueModifier), -2)
    this.tokenKind = PositionTokenMetadata.decodeTokenKind(this.info.tokenKind)
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

  getExposure(value: Number128) {
    return value
  }

  getCollateralValue(value: Number128) {
    return this.valueModifier.mul(value).div(Number128.ONE)
  }

  getRequiredCollateralValue(value: Number128) {
    if (this.valueModifier.eq(Number128.ZERO)) {
      // No leverage configured for claim
      return Number128.MAX
    } else {
      return value.mul(Number128.ONE).div(this.valueModifier)
    }
  }
}
