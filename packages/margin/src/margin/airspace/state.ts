import { AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types"
import { JetAirspace } from "../../types"

export type AirspaceData = TypeDef<AllAccountsMap<JetAirspace>["Airspace"], IdlTypes<JetAirspace>>
