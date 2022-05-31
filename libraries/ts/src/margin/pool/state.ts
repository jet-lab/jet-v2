import { AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types"
import { JetMarginPool } from "../../types"

export type MarginPoolData = TypeDef<AllAccountsMap<JetMarginPool>["marginPool"], IdlTypes<JetMarginPool>>
export type MarginPoolConfigData = MarginPoolData["config"]
export type MarginPoolOracleData = TypeDef<AllAccountsMap<JetMarginPool>["marginPoolOracle"], IdlTypes<JetMarginPool>>
