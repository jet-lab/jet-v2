import { AllAccountsMap, IdlTypes, TypeDef } from "@project-serum/anchor/dist/cjs/program/namespace/types"

export type MarginPoolData = TypeDef<AllAccountsMap<JetMarginPoolIDL>["marginPool"], IdlTypes<JetMarginPoolIDL>>
export type MarginPoolConfigData = MarginPoolData["config"]
