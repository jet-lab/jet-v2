export type JetMetadata = {
  version: "0.1.0"
  name: "jet_metadata"
  instructions: [
    {
      name: "createEntry"
      accounts: [
        {
          name: "keyAccount"
          isMut: false
          isSigner: false
        },
        {
          name: "metadataAccount"
          isMut: true
          isSigner: false
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "seed"
          type: "string"
        },
        {
          name: "space"
          type: "u64"
        }
      ]
    },
    {
      name: "setEntry"
      accounts: [
        {
          name: "metadataAccount"
          isMut: true
          isSigner: false
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
        }
      ]
      args: [
        {
          name: "offset"
          type: "u64"
        },
        {
          name: "data"
          type: "bytes"
        }
      ]
    }
  ]
  accounts: [
    {
      name: "positionTokenMetadata"
      type: {
        kind: "struct"
        fields: [
          {
            name: "positionTokenMint"
            type: "publicKey"
          },
          {
            name: "underlyingTokenMint"
            type: "publicKey"
          },
          {
            name: "adapterProgram"
            type: "publicKey"
          },
          {
            name: "tokenKind"
            type: {
              defined: "TokenKind"
            }
          },
          {
            name: "collateralWeight"
            type: "u16"
          },
          {
            name: "collateralMaxStaleness"
            type: "u64"
          }
        ]
      }
    },
    {
      name: "tokenMetadata"
      type: {
        kind: "struct"
        fields: [
          {
            name: "tokenMint"
            type: "publicKey"
          },
          {
            name: "pythPrice"
            type: "publicKey"
          },
          {
            name: "pythProduct"
            type: "publicKey"
          }
        ]
      }
    },
    {
      name: "marginAdapterMetadata"
      type: {
        kind: "struct"
        fields: [
          {
            name: "adapterProgram"
            type: "publicKey"
          }
        ]
      }
    },
    {
      name: "liquidatorAdapterMetadata"
      type: {
        kind: "struct"
        fields: [
          {
            name: "adapterProgram"
            type: "publicKey"
          }
        ]
      }
    },
    {
      name: "liquidatorMetadata"
      type: {
        kind: "struct"
        fields: [
          {
            name: "liquidator"
            type: "publicKey"
          }
        ]
      }
    }
  ]
  types: [
    {
      name: "TokenKind"
      type: {
        kind: "enum"
        variants: [
          {
            name: "NonCollateral"
          },
          {
            name: "Collateral"
          },
          {
            name: "Claim"
          }
        ]
      }
    }
  ]
}

export const IDL: JetMetadata = {
  version: "0.1.0",
  name: "jet_metadata",
  instructions: [
    {
      name: "createEntry",
      accounts: [
        {
          name: "keyAccount",
          isMut: false,
          isSigner: false
        },
        {
          name: "metadataAccount",
          isMut: true,
          isSigner: false
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: [
        {
          name: "seed",
          type: "string"
        },
        {
          name: "space",
          type: "u64"
        }
      ]
    },
    {
      name: "setEntry",
      accounts: [
        {
          name: "metadataAccount",
          isMut: true,
          isSigner: false
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true
        }
      ],
      args: [
        {
          name: "offset",
          type: "u64"
        },
        {
          name: "data",
          type: "bytes"
        }
      ]
    }
  ],
  accounts: [
    {
      name: "positionTokenMetadata",
      type: {
        kind: "struct",
        fields: [
          {
            name: "positionTokenMint",
            type: "publicKey"
          },
          {
            name: "underlyingTokenMint",
            type: "publicKey"
          },
          {
            name: "adapterProgram",
            type: "publicKey"
          },
          {
            name: "tokenKind",
            type: {
              defined: "TokenKind"
            }
          },
          {
            name: "collateralWeight",
            type: "u16"
          },
          {
            name: "collateralMaxStaleness",
            type: "u64"
          }
        ]
      }
    },
    {
      name: "tokenMetadata",
      type: {
        kind: "struct",
        fields: [
          {
            name: "tokenMint",
            type: "publicKey"
          },
          {
            name: "pythPrice",
            type: "publicKey"
          },
          {
            name: "pythProduct",
            type: "publicKey"
          }
        ]
      }
    },
    {
      name: "marginAdapterMetadata",
      type: {
        kind: "struct",
        fields: [
          {
            name: "adapterProgram",
            type: "publicKey"
          }
        ]
      }
    },
    {
      name: "liquidatorAdapterMetadata",
      type: {
        kind: "struct",
        fields: [
          {
            name: "adapterProgram",
            type: "publicKey"
          }
        ]
      }
    },
    {
      name: "liquidatorMetadata",
      type: {
        kind: "struct",
        fields: [
          {
            name: "liquidator",
            type: "publicKey"
          }
        ]
      }
    }
  ],
  types: [
    {
      name: "TokenKind",
      type: {
        kind: "enum",
        variants: [
          {
            name: "NonCollateral"
          },
          {
            name: "Collateral"
          },
          {
            name: "Claim"
          }
        ]
      }
    }
  ]
}
