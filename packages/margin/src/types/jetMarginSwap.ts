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
          name: "withdrawalChangeKind"
          type: {
            defined: "ChangeKind"
          }
        },
        {
          name: "withdrawalAmount"
          type: "u64"
        },
        {
          name: "minimumAmountOut"
          type: "u64"
        }
      ]
    },
    {
      name: "saberStableSwap"
      docs: ["Swap using Saber for stable pools"]
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
          name: "adminFeeDestination"
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
      args: []
    },
    {
      name: "routeSwap"
      docs: ["Route a swap to one or more venues"]
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
          docs: [
            "The destination account to send the deposit that is exchanged into",
            "The swap is also atomic, and no excess funds would be taken/left in the account."
          ]
        },
        {
          name: "transitSourceAccount"
          isMut: true
          isSigner: false
          docs: [
            "Temporary account for moving tokens",
            "The swap is also atomic, and no excess funds would be taken/left in the account."
          ]
        },
        {
          name: "transitDestinationAccount"
          isMut: true
          isSigner: false
          docs: [
            "Temporary account for moving tokens",
            "The swap is also atomic, and no excess funds would be taken/left in the account."
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
          name: "withdrawalChangeKind"
          type: {
            defined: "ChangeKind"
          }
        },
        {
          name: "withdrawalAmount"
          type: "u64"
        },
        {
          name: "minimumAmountOut"
          type: "u64"
        },
        {
          name: "swapRoutes"
          type: {
            array: [
              {
                defined: "SwapRouteDetail"
              },
              3
            ]
          }
        }
      ]
    }
  ]
  types: [
    {
      name: "SwapRouteDetail"
      type: {
        kind: "struct"
        fields: [
          {
            name: "routeA"
            type: {
              defined: "SwapRouteIdentifier"
            }
          },
          {
            name: "routeB"
            type: {
              defined: "SwapRouteIdentifier"
            }
          },
          {
            name: "destinationMint"
            type: "publicKey"
          },
          {
            name: "split"
            type: "u8"
          }
        ]
      }
    },
    {
      name: "SwapRouteIdentifier"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Empty"
          },
          {
            name: "Spl"
          },
          {
            name: "Whirlpool"
          },
          {
            name: "SaberStable"
          }
        ]
      }
    },
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
    },
    {
      code: 6001
      name: "InvalidSwapRoute"
      msg: "An invalid swap route has been provided"
    },
    {
      code: 6002
      name: "InvalidSwapRouteParam"
      msg: "An invalid swap route parameter has been provided"
    },
    {
      code: 6003
      name: "SlippageExceeded"
      msg: "The swap exceeds the maximum slippage tolerance"
    },
    {
      code: 6004
      name: "DisallowedDirectInstruction"
      msg: "The instruction should not be called directly, use route_swap"
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
          name: "withdrawalChangeKind",
          type: {
            defined: "ChangeKind"
          }
        },
        {
          name: "withdrawalAmount",
          type: "u64"
        },
        {
          name: "minimumAmountOut",
          type: "u64"
        }
      ]
    },
    {
      name: "saberStableSwap",
      docs: ["Swap using Saber for stable pools"],
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
          name: "adminFeeDestination",
          isMut: true,
          isSigner: false
        },
        {
          name: "swapProgram",
          isMut: false,
          isSigner: false,
          docs: ["The address of the swap program"]
        }
      ],
      args: []
    },
    {
      name: "routeSwap",
      docs: ["Route a swap to one or more venues"],
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
          docs: [
            "The destination account to send the deposit that is exchanged into",
            "The swap is also atomic, and no excess funds would be taken/left in the account."
          ]
        },
        {
          name: "transitSourceAccount",
          isMut: true,
          isSigner: false,
          docs: [
            "Temporary account for moving tokens",
            "The swap is also atomic, and no excess funds would be taken/left in the account."
          ]
        },
        {
          name: "transitDestinationAccount",
          isMut: true,
          isSigner: false,
          docs: [
            "Temporary account for moving tokens",
            "The swap is also atomic, and no excess funds would be taken/left in the account."
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
          name: "withdrawalChangeKind",
          type: {
            defined: "ChangeKind"
          }
        },
        {
          name: "withdrawalAmount",
          type: "u64"
        },
        {
          name: "minimumAmountOut",
          type: "u64"
        },
        {
          name: "swapRoutes",
          type: {
            array: [
              {
                defined: "SwapRouteDetail"
              },
              3
            ]
          }
        }
      ]
    }
  ],
  types: [
    {
      name: "SwapRouteDetail",
      type: {
        kind: "struct",
        fields: [
          {
            name: "routeA",
            type: {
              defined: "SwapRouteIdentifier"
            }
          },
          {
            name: "routeB",
            type: {
              defined: "SwapRouteIdentifier"
            }
          },
          {
            name: "destinationMint",
            type: "publicKey"
          },
          {
            name: "split",
            type: "u8"
          }
        ]
      }
    },
    {
      name: "SwapRouteIdentifier",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Empty"
          },
          {
            name: "Spl"
          },
          {
            name: "Whirlpool"
          },
          {
            name: "SaberStable"
          }
        ]
      }
    },
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
    },
    {
      code: 6001,
      name: "InvalidSwapRoute",
      msg: "An invalid swap route has been provided"
    },
    {
      code: 6002,
      name: "InvalidSwapRouteParam",
      msg: "An invalid swap route parameter has been provided"
    },
    {
      code: 6003,
      name: "SlippageExceeded",
      msg: "The swap exceeds the maximum slippage tolerance"
    },
    {
      code: 6004,
      name: "DisallowedDirectInstruction",
      msg: "The instruction should not be called directly, use route_swap"
    }
  ]
}
