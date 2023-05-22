import { AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types"

export type AirspaceData = TypeDef<AllAccountsMap<JetAirspaceIDL>["Airspace"], IdlTypes<JetAirspaceIDL>>
