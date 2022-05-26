export type JetMarginSwap = {
  "version": "0.1.0",
  "name": "jet_margin_swap",
  "instructions": [
    {
      "name": "marginSwap",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "sourceAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "destinationAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transitSourceAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transitDestinationAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "swapInfo",
          "accounts": [
            {
              "name": "swapPool",
              "isMut": false,
              "isSigner": false
            },
            {
              "name": "authority",
              "isMut": false,
              "isSigner": false
            },
            {
              "name": "vaultInto",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "vaultFrom",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "tokenMint",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "feeAccount",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "swapProgram",
              "isMut": false,
              "isSigner": false
            }
          ]
        },
        {
          "name": "sourceMarginPool",
          "accounts": [
            {
              "name": "marginPool",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "vault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "depositNoteMint",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "destinationMarginPool",
          "accounts": [
            {
              "name": "marginPool",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "vault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "depositNoteMint",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "marginPoolProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amountIn",
          "type": "u64"
        },
        {
          "name": "minimumAmountOut",
          "type": "u64"
        }
      ]
    }
  ]
};

export const IDL: JetMarginSwap = {
  "version": "0.1.0",
  "name": "jet_margin_swap",
  "instructions": [
    {
      "name": "marginSwap",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "sourceAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "destinationAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transitSourceAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "transitDestinationAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "swapInfo",
          "accounts": [
            {
              "name": "swapPool",
              "isMut": false,
              "isSigner": false
            },
            {
              "name": "authority",
              "isMut": false,
              "isSigner": false
            },
            {
              "name": "vaultInto",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "vaultFrom",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "tokenMint",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "feeAccount",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "swapProgram",
              "isMut": false,
              "isSigner": false
            }
          ]
        },
        {
          "name": "sourceMarginPool",
          "accounts": [
            {
              "name": "marginPool",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "vault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "depositNoteMint",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "destinationMarginPool",
          "accounts": [
            {
              "name": "marginPool",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "vault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "depositNoteMint",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "marginPoolProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amountIn",
          "type": "u64"
        },
        {
          "name": "minimumAmountOut",
          "type": "u64"
        }
      ]
    }
  ]
};
