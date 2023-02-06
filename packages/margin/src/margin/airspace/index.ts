import { Address, Program, translateAddress } from "@project-serum/anchor"
import { PublicKey } from "@solana/web3.js"
import { JetAirspace } from "types"
import { findDerivedAccount } from "../../utils"
import { AirspaceData } from "./state"

export * from "./state"

/**
 * The Jet Airspace program
 */
export class Airspace {
  /**
   * Derives the address for the issuerId required to create an airspace permit
   *
   * @param authority The authority requesting the permit
   * @returns
   */
  derivePermitIssuerId(authority: Address): PublicKey {
    return findDerivedAccount(this.program.programId, "airspace-permit-issuer", this.address, authority)
  }

  /**
   *
   * @param program The JetAirspace program
   * @param address The address of the onchain airspace
   * @param info The airspace metadata
   */
  constructor(
    readonly program: Program<JetAirspace>,
    readonly address: PublicKey,
    readonly info: AirspaceData | undefined
  ) {}

  /**
   *
   * @param program The JetAirspace program
   * @param airspace The address of this particular airspace
   * @returns
   */
  static async load(program: Program<JetAirspace>, airspace: Address): Promise<Airspace> {
    const address = translateAddress(airspace)
    const data = (await program.provider.connection.getAccountInfo(address))!.data
    const info: AirspaceData = program.coder.accounts.decode("Airspace", data)
    return new Airspace(program, address, info)
  }
}
