export type JetControl = {
  version: "0.1.0"
  name: "jet_control"
  instructions: [
    {
      name: "createAuthority"
      accounts: [
        {
          name: "authority"
          isMut: true
          isSigner: false
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
      args: []
    },
    {
      name: "registerToken"
      accounts: [
        {
          name: "requester"
          isMut: true
          isSigner: true
        },
        {
          name: "authority"
          isMut: false
          isSigner: false
        },
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
        },
        {
          name: "loanNoteMint"
          isMut: true
          isSigner: false
        },
        {
          name: "tokenMint"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenMetadata"
          isMut: true
          isSigner: false
        },
        {
          name: "depositNoteMetadata"
          isMut: true
          isSigner: false
        },
        {
          name: "loanNoteMetadata"
          isMut: true
          isSigner: false
        },
        {
          name: "marginPoolProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "metadataProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "rent"
          isMut: false
          isSigner: false
        }
      ]
      args: []
    },
    {
      name: "registerAdapter"
      accounts: [
        {
          name: "requester"
          isMut: false
          isSigner: true
        },
        {
          name: "authority"
          isMut: false
          isSigner: false
        },
        {
          name: "adapter"
          isMut: false
          isSigner: false
        },
        {
          name: "metadataAccount"
          isMut: true
          isSigner: false
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
        },
        {
          name: "metadataProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: []
    },
    {
      name: "configureToken"
      accounts: [
        {
          name: "requester"
          isMut: false
          isSigner: true
        },
        {
          name: "authority"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenMint"
          isMut: false
          isSigner: false
        },
        {
          name: "marginPool"
          isMut: true
          isSigner: false
        },
        {
          name: "tokenMetadata"
          isMut: true
          isSigner: false
        },
        {
          name: "depositMetadata"
          isMut: true
          isSigner: false
        },
        {
          name: "pythProduct"
          isMut: false
          isSigner: false
        },
        {
          name: "pythPrice"
          isMut: false
          isSigner: false
        },
        {
          name: "marginPoolProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "metadataProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "metadata"
          type: {
            option: {
              defined: "TokenMetadataParams"
            }
          }
        },
        {
          name: "poolParam"
          type: {
            option: {
              defined: "MarginPoolParams"
            }
          }
        },
        {
          name: "poolConfig"
          type: {
            option: {
              defined: "MarginPoolConfig"
            }
          }
        }
      ]
    }
  ]
  accounts: [
    {
      name: "authority"
      type: {
        kind: "struct"
        fields: [
          {
            name: "seed"
            type: {
              array: ["u8", 1]
            }
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
    },
    {
      name: "TokenMetadataParams"
      type: {
        kind: "struct"
        fields: [
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
      name: "MarginPoolConfig"
      type: {
        kind: "struct"
        fields: [
          {
            name: "flags"
            type: "u64"
          },
          {
            name: "utilizationRate1"
            type: "u16"
          },
          {
            name: "utilizationRate2"
            type: "u16"
          },
          {
            name: "borrowRate0"
            type: "u16"
          },
          {
            name: "borrowRate1"
            type: "u16"
          },
          {
            name: "borrowRate2"
            type: "u16"
          },
          {
            name: "borrowRate3"
            type: "u16"
          },
          {
            name: "managementFeeRate"
            type: "u16"
          },
          {
            name: "managementFeeCollectThreshold"
            type: "u64"
          }
        ]
      }
    },
    {
      name: "MarginPoolParams"
      type: {
        kind: "struct"
        fields: [
          {
            name: "feeDestination"
            type: "publicKey"
          }
        ]
      }
    }
  ]
}

export const IDL: JetControl = {
  version: "0.1.0",
  name: "jet_control",
  instructions: [
    {
      name: "createAuthority",
      accounts: [
        {
          name: "authority",
          isMut: true,
          isSigner: false
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
      args: []
    },
    {
      name: "registerToken",
      accounts: [
        {
          name: "requester",
          isMut: true,
          isSigner: true
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false
        },
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
        },
        {
          name: "loanNoteMint",
          isMut: true,
          isSigner: false
        },
        {
          name: "tokenMint",
          isMut: false,
          isSigner: false
        },
        {
          name: "tokenMetadata",
          isMut: true,
          isSigner: false
        },
        {
          name: "depositNoteMetadata",
          isMut: true,
          isSigner: false
        },
        {
          name: "loanNoteMetadata",
          isMut: true,
          isSigner: false
        },
        {
          name: "marginPoolProgram",
          isMut: false,
          isSigner: false
        },
        {
          name: "metadataProgram",
          isMut: false,
          isSigner: false
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false
        },
        {
          name: "rent",
          isMut: false,
          isSigner: false
        }
      ],
      args: []
    },
    {
      name: "registerAdapter",
      accounts: [
        {
          name: "requester",
          isMut: false,
          isSigner: true
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false
        },
        {
          name: "adapter",
          isMut: false,
          isSigner: false
        },
        {
          name: "metadataAccount",
          isMut: true,
          isSigner: false
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true
        },
        {
          name: "metadataProgram",
          isMut: false,
          isSigner: false
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: []
    },
    {
      name: "configureToken",
      accounts: [
        {
          name: "requester",
          isMut: false,
          isSigner: true
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false
        },
        {
          name: "tokenMint",
          isMut: false,
          isSigner: false
        },
        {
          name: "marginPool",
          isMut: true,
          isSigner: false
        },
        {
          name: "tokenMetadata",
          isMut: true,
          isSigner: false
        },
        {
          name: "depositMetadata",
          isMut: true,
          isSigner: false
        },
        {
          name: "pythProduct",
          isMut: false,
          isSigner: false
        },
        {
          name: "pythPrice",
          isMut: false,
          isSigner: false
        },
        {
          name: "marginPoolProgram",
          isMut: false,
          isSigner: false
        },
        {
          name: "metadataProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: [
        {
          name: "metadata",
          type: {
            option: {
              defined: "TokenMetadataParams"
            }
          }
        },
        {
          name: "poolParam",
          type: {
            option: {
              defined: "MarginPoolParams"
            }
          }
        },
        {
          name: "poolConfig",
          type: {
            option: {
              defined: "MarginPoolConfig"
            }
          }
        }
      ]
    }
  ],
  accounts: [
    {
      name: "authority",
      type: {
        kind: "struct",
        fields: [
          {
            name: "seed",
            type: {
              array: ["u8", 1]
            }
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
    },
    {
      name: "TokenMetadataParams",
      type: {
        kind: "struct",
        fields: [
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
      name: "MarginPoolConfig",
      type: {
        kind: "struct",
        fields: [
          {
            name: "flags",
            type: "u64"
          },
          {
            name: "utilizationRate1",
            type: "u16"
          },
          {
            name: "utilizationRate2",
            type: "u16"
          },
          {
            name: "borrowRate0",
            type: "u16"
          },
          {
            name: "borrowRate1",
            type: "u16"
          },
          {
            name: "borrowRate2",
            type: "u16"
          },
          {
            name: "borrowRate3",
            type: "u16"
          },
          {
            name: "managementFeeRate",
            type: "u16"
          },
          {
            name: "managementFeeCollectThreshold",
            type: "u64"
          }
        ]
      }
    },
    {
      name: "MarginPoolParams",
      type: {
        kind: "struct",
        fields: [
          {
            name: "feeDestination",
            type: "publicKey"
          }
        ]
      }
    }
  ]
}
