export type JetMarginSwap = {
  version: "1.0.0"
  name: "jet_margin_swap"
  instructions: [
    {
      name: "marginSwap"
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The margin account being executed on"]
        },
        {
          name: "sourceAccount"
          isMut: true
          isSigner: false
          docs: ["The account with the source deposit to be exchanged from"]
        },
        {
          name: "destinationAccount"
          isMut: true
          isSigner: false
          docs: ["The destination account to send the deposit that is exchanged into"]
        },
        {
          name: "transitSourceAccount"
          isMut: true
          isSigner: false
          docs: ["Temporary account for moving tokens"]
        },
        {
          name: "transitDestinationAccount"
          isMut: true
          isSigner: false
          docs: ["Temporary account for moving tokens"]
        },
        {
          name: "swapInfo"
          accounts: [
            {
              name: "swapPool"
              isMut: false
              isSigner: false
            },
            {
              name: "authority"
              isMut: false
              isSigner: false
            },
            {
              name: "vaultInto"
              isMut: true
              isSigner: false
            },
            {
              name: "vaultFrom"
              isMut: true
              isSigner: false
            },
            {
              name: "tokenMint"
              isMut: true
              isSigner: false
            },
            {
              name: "feeAccount"
              isMut: true
              isSigner: false
            },
            {
              name: "swapProgram"
              isMut: false
              isSigner: false
              docs: ["The address of the swap program"]
            }
          ]
        },
        {
          name: "sourceMarginPool"
          accounts: [
            {
              name: "marginPool"
              isMut: true
              isSigner: false
            },
            {
              name: "vault"
              isMut: true
              isSigner: false
            },
            {
              name: "depositNoteMint"
              isMut: true
              isSigner: false
            }
          ]
        },
        {
          name: "destinationMarginPool"
          accounts: [
            {
              name: "marginPool"
              isMut: true
              isSigner: false
            },
            {
              name: "vault"
              isMut: true
              isSigner: false
            },
            {
              name: "depositNoteMint"
              isMut: true
              isSigner: false
            }
          ]
        },
        {
          name: "marginPoolProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "changeKind"
          type: {
            defined: "ChangeKind"
          }
        },
        {
          name: "amountIn"
          type: "u64"
        },
        {
          name: "minimumAmountOut"
          type: "u64"
        }
      ]
    }
  ]
  types: [
    {
      name: "ChangeKind"
      type: {
        kind: "enum"
        variants: [
          {
            name: "SetTo"
          },
          {
            name: "ShiftBy"
          }
        ]
      }
    }
  ]
  errors: [
    {
      code: 6000
      name: "NoSwapTokensWithdrawn"
      msg: "Zero tokens have been withdrawn from a pool for the swap"
    }
  ]
}

export const IDL: JetMarginSwap = {
  version: "1.0.0",
  name: "jet_margin_swap",
  instructions: [
    {
      name: "marginSwap",
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: true,
          docs: ["The margin account being executed on"]
        },
        {
          name: "sourceAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account with the source deposit to be exchanged from"]
        },
        {
          name: "destinationAccount",
          isMut: true,
          isSigner: false,
          docs: ["The destination account to send the deposit that is exchanged into"]
        },
        {
          name: "transitSourceAccount",
          isMut: true,
          isSigner: false,
          docs: ["Temporary account for moving tokens"]
        },
        {
          name: "transitDestinationAccount",
          isMut: true,
          isSigner: false,
          docs: ["Temporary account for moving tokens"]
        },
        {
          name: "swapInfo",
          accounts: [
            {
              name: "swapPool",
              isMut: false,
              isSigner: false
            },
            {
              name: "authority",
              isMut: false,
              isSigner: false
            },
            {
              name: "vaultInto",
              isMut: true,
              isSigner: false
            },
            {
              name: "vaultFrom",
              isMut: true,
              isSigner: false
            },
            {
              name: "tokenMint",
              isMut: true,
              isSigner: false
            },
            {
              name: "feeAccount",
              isMut: true,
              isSigner: false
            },
            {
              name: "swapProgram",
              isMut: false,
              isSigner: false,
              docs: ["The address of the swap program"]
            }
          ]
        },
        {
          name: "sourceMarginPool",
          accounts: [
            {
              name: "marginPool",
              isMut: true,
              isSigner: false
            },
            {
              name: "vault",
              isMut: true,
              isSigner: false
            },
            {
              name: "depositNoteMint",
              isMut: true,
              isSigner: false
            }
          ]
        },
        {
          name: "destinationMarginPool",
          accounts: [
            {
              name: "marginPool",
              isMut: true,
              isSigner: false
            },
            {
              name: "vault",
              isMut: true,
              isSigner: false
            },
            {
              name: "depositNoteMint",
              isMut: true,
              isSigner: false
            }
          ]
        },
        {
          name: "marginPoolProgram",
          isMut: false,
          isSigner: false
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: [
        {
          name: "changeKind",
          type: {
            defined: "ChangeKind"
          }
        },
        {
          name: "amountIn",
          type: "u64"
        },
        {
          name: "minimumAmountOut",
          type: "u64"
        }
      ]
    }
  ],
  types: [
    {
      name: "ChangeKind",
      type: {
        kind: "enum",
        variants: [
          {
            name: "SetTo"
          },
          {
            name: "ShiftBy"
          }
        ]
      }
    }
  ],
  errors: [
    {
      code: 6000,
      name: "NoSwapTokensWithdrawn",
      msg: "Zero tokens have been withdrawn from a pool for the swap"
    }
  ]
}
