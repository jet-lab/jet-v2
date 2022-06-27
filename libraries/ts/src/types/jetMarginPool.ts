export type JetMarginPool = {
  version: "0.1.0"
  name: "jet_margin_pool"
  instructions: [
    {
      name: "createPool"
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
      name: "configure"
      accounts: [
        {
          name: "marginPool"
          isMut: true
          isSigner: false
        },
        {
          name: "authority"
          isMut: false
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
        }
      ]
      args: [
        {
          name: "feeDestination"
          type: {
            option: "publicKey"
          }
        },
        {
          name: "config"
          type: {
            option: {
              defined: "MarginPoolConfig"
            }
          }
        }
      ]
    },
    {
      name: "collect"
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
          name: "feeDestination"
          isMut: true
          isSigner: false
        },
        {
          name: "depositNoteMint"
          isMut: true
          isSigner: false
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: []
    },
    {
      name: "deposit"
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
        },
        {
          name: "depositor"
          isMut: false
          isSigner: true
        },
        {
          name: "source"
          isMut: true
          isSigner: false
        },
        {
          name: "destination"
          isMut: true
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
          name: "amount"
          type: "u64"
        }
      ]
    },
    {
      name: "withdraw"
      accounts: [
        {
          name: "depositor"
          isMut: false
          isSigner: true
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
          name: "source"
          isMut: true
          isSigner: false
        },
        {
          name: "destination"
          isMut: true
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
          name: "amount"
          type: {
            defined: "Amount"
          }
        }
      ]
    },
    {
      name: "marginBorrow"
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
        },
        {
          name: "marginPool"
          isMut: true
          isSigner: false
        },
        {
          name: "loanNoteMint"
          isMut: true
          isSigner: false
        },
        {
          name: "depositNoteMint"
          isMut: true
          isSigner: false
        },
        {
          name: "loanAccount"
          isMut: true
          isSigner: false
        },
        {
          name: "depositAccount"
          isMut: true
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
          name: "amount"
          type: "u64"
        }
      ]
    },
    {
      name: "marginRepay"
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
        },
        {
          name: "marginPool"
          isMut: true
          isSigner: false
        },
        {
          name: "loanNoteMint"
          isMut: true
          isSigner: false
        },
        {
          name: "depositNoteMint"
          isMut: true
          isSigner: false
        },
        {
          name: "loanAccount"
          isMut: true
          isSigner: false
        },
        {
          name: "depositAccount"
          isMut: true
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
          name: "maxAmount"
          type: {
            defined: "Amount"
          }
        }
      ]
    },
    {
      name: "marginRefreshPosition"
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: false
        },
        {
          name: "marginPool"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenPriceOracle"
          isMut: false
          isSigner: false
        }
      ]
      args: []
    }
  ]
  accounts: [
    {
      name: "marginPool"
      type: {
        kind: "struct"
        fields: [
          {
            name: "version"
            type: "u8"
          },
          {
            name: "poolBump"
            type: {
              array: ["u8", 1]
            }
          },
          {
            name: "vault"
            type: "publicKey"
          },
          {
            name: "feeDestination"
            type: "publicKey"
          },
          {
            name: "depositNoteMint"
            type: "publicKey"
          },
          {
            name: "loanNoteMint"
            type: "publicKey"
          },
          {
            name: "tokenMint"
            type: "publicKey"
          },
          {
            name: "tokenPriceOracle"
            type: "publicKey"
          },
          {
            name: "address"
            type: "publicKey"
          },
          {
            name: "config"
            type: {
              defined: "MarginPoolConfig"
            }
          },
          {
            name: "borrowedTokens"
            type: {
              array: ["u8", 24]
            }
          },
          {
            name: "uncollectedFees"
            type: {
              array: ["u8", 24]
            }
          },
          {
            name: "depositTokens"
            type: "u64"
          },
          {
            name: "depositNotes"
            type: "u64"
          },
          {
            name: "loanNotes"
            type: "u64"
          },
          {
            name: "accruedUntil"
            type: "i64"
          }
        ]
      }
    }
  ]
  types: [
    {
      name: "MarginPoolSummary"
      type: {
        kind: "struct"
        fields: [
          {
            name: "borrowedTokens"
            type: "u64"
          },
          {
            name: "uncollectedFees"
            type: "u64"
          },
          {
            name: "depositTokens"
            type: "u64"
          },
          {
            name: "depositNotes"
            type: "u64"
          },
          {
            name: "loanNotes"
            type: "u64"
          },
          {
            name: "accruedUntil"
            type: "i64"
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
      name: "Amount"
      type: {
        kind: "struct"
        fields: [
          {
            name: "kind"
            type: {
              defined: "AmountKind"
            }
          },
          {
            name: "value"
            type: "u64"
          }
        ]
      }
    },
    {
      name: "PoolAction"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Borrow"
          },
          {
            name: "Deposit"
          },
          {
            name: "Repay"
          },
          {
            name: "Withdraw"
          }
        ]
      }
    },
    {
      name: "RoundingDirection"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Down"
          },
          {
            name: "Up"
          }
        ]
      }
    },
    {
      name: "AmountKind"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Tokens"
          },
          {
            name: "Notes"
          }
        ]
      }
    },
    {
      name: "PoolAction"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Borrow"
          },
          {
            name: "Deposit"
          },
          {
            name: "Repay"
          },
          {
            name: "Withdraw"
          }
        ]
      }
    },
    {
      name: "RoundingDirection"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Down"
          },
          {
            name: "Up"
          }
        ]
      }
    }
  ]
  events: [
    {
      name: "PoolCreated"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "vault"
          type: "publicKey"
          index: false
        },
        {
          name: "depositNoteMint"
          type: "publicKey"
          index: false
        },
        {
          name: "loanNoteMint"
          type: "publicKey"
          index: false
        },
        {
          name: "tokenMint"
          type: "publicKey"
          index: false
        },
        {
          name: "authority"
          type: "publicKey"
          index: false
        },
        {
          name: "payer"
          type: "publicKey"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "PoolConfigured"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "feeDestination"
          type: "publicKey"
          index: false
        },
        {
          name: "pythProduct"
          type: "publicKey"
          index: false
        },
        {
          name: "pythPrice"
          type: "publicKey"
          index: false
        },
        {
          name: "config"
          type: {
            defined: "MarginPoolConfig"
          }
          index: false
        }
      ]
    },
    {
      name: "Deposit"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "user"
          type: "publicKey"
          index: false
        },
        {
          name: "source"
          type: "publicKey"
          index: false
        },
        {
          name: "destination"
          type: "publicKey"
          index: false
        },
        {
          name: "depositTokens"
          type: "u64"
          index: false
        },
        {
          name: "depositNotes"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "Withdraw"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "user"
          type: "publicKey"
          index: false
        },
        {
          name: "source"
          type: "publicKey"
          index: false
        },
        {
          name: "destination"
          type: "publicKey"
          index: false
        },
        {
          name: "withdrawTokens"
          type: "u64"
          index: false
        },
        {
          name: "withdrawNotes"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "MarginBorrow"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "user"
          type: "publicKey"
          index: false
        },
        {
          name: "loanAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "depositAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "tokens"
          type: "u64"
          index: false
        },
        {
          name: "loanNotes"
          type: "u64"
          index: false
        },
        {
          name: "depositNotes"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "MarginRepay"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "user"
          type: "publicKey"
          index: false
        },
        {
          name: "loanAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "depositAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "maxRepayTokens"
          type: "u64"
          index: false
        },
        {
          name: "maxRepayNotes"
          type: "u64"
          index: false
        },
        {
          name: "repaidTokens"
          type: "u64"
          index: false
        },
        {
          name: "repaidLoanNotes"
          type: "u64"
          index: false
        },
        {
          name: "repaidDepositNotes"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "Collect"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "feeNotesMinted"
          type: "u64"
          index: false
        },
        {
          name: "feeTokensClaimed"
          type: "u64"
          index: false
        },
        {
          name: "feeNotesBalance"
          type: "u64"
          index: false
        },
        {
          name: "feeTokensBalance"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    }
  ]
  events: [
    {
      name: "PoolCreated"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "vault"
          type: "publicKey"
          index: false
        },
        {
          name: "depositNoteMint"
          type: "publicKey"
          index: false
        },
        {
          name: "loanNoteMint"
          type: "publicKey"
          index: false
        },
        {
          name: "tokenMint"
          type: "publicKey"
          index: false
        },
        {
          name: "authority"
          type: "publicKey"
          index: false
        },
        {
          name: "payer"
          type: "publicKey"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "PoolConfigured"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "feeDestination"
          type: "publicKey"
          index: false
        },
        {
          name: "pythProduct"
          type: "publicKey"
          index: false
        },
        {
          name: "pythPrice"
          type: "publicKey"
          index: false
        },
        {
          name: "config"
          type: {
            defined: "MarginPoolConfig"
          }
          index: false
        }
      ]
    },
    {
      name: "Deposit"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "user"
          type: "publicKey"
          index: false
        },
        {
          name: "source"
          type: "publicKey"
          index: false
        },
        {
          name: "destination"
          type: "publicKey"
          index: false
        },
        {
          name: "depositTokens"
          type: "u64"
          index: false
        },
        {
          name: "depositNotes"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "Withdraw"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "user"
          type: "publicKey"
          index: false
        },
        {
          name: "source"
          type: "publicKey"
          index: false
        },
        {
          name: "destination"
          type: "publicKey"
          index: false
        },
        {
          name: "withdrawTokens"
          type: "u64"
          index: false
        },
        {
          name: "withdrawNotes"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "MarginBorrow"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "user"
          type: "publicKey"
          index: false
        },
        {
          name: "loanAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "depositAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "tokens"
          type: "u64"
          index: false
        },
        {
          name: "loanNotes"
          type: "u64"
          index: false
        },
        {
          name: "depositNotes"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "MarginRepay"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "user"
          type: "publicKey"
          index: false
        },
        {
          name: "loanAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "depositAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "maxRepayTokens"
          type: "u64"
          index: false
        },
        {
          name: "maxRepayNotes"
          type: "u64"
          index: false
        },
        {
          name: "repaidTokens"
          type: "u64"
          index: false
        },
        {
          name: "repaidLoanNotes"
          type: "u64"
          index: false
        },
        {
          name: "repaidDepositNotes"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "Collect"
      fields: [
        {
          name: "marginPool"
          type: "publicKey"
          index: false
        },
        {
          name: "feeNotesMinted"
          type: "u64"
          index: false
        },
        {
          name: "feeTokensClaimed"
          type: "u64"
          index: false
        },
        {
          name: "feeNotesBalance"
          type: "u64"
          index: false
        },
        {
          name: "feeTokensBalance"
          type: "u64"
          index: false
        },
        {
          name: "summary"
          type: {
            defined: "MarginPoolSummary"
          }
          index: false
        }
      ]
    }
  ]
  errors: [
    {
      code: 141100
      name: "Disabled"
      msg: "The pool is currently disabled"
    },
    {
      code: 141101
      name: "InterestAccrualBehind"
      msg: "Interest accrual is too far behind"
    },
    {
      code: 141102
      name: "DepositsOnly"
      msg: "The pool currently only allows deposits"
    },
    {
      code: 141103
      name: "InsufficientLiquidity"
      msg: "The pool does not have sufficient liquidity for the transaction"
    },
    {
      code: 141104
      name: "InvalidAmount"
      msg: "An invalid amount has been supplied"
    },
    {
      code: 141105
      name: "InvalidPrice"
    },
    {
      code: 141106
      name: "InvalidOracle"
    },
    {
      code: 141107
      name: "RepaymentExceedsTotalOutstanding"
    }
  ]
}

export const IDL: JetMarginPool = {
  version: "0.1.0",
  name: "jet_margin_pool",
  instructions: [
    {
      name: "createPool",
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
      name: "configure",
      accounts: [
        {
          name: "marginPool",
          isMut: true,
          isSigner: false
        },
        {
          name: "authority",
          isMut: false,
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
        }
      ],
      args: [
        {
          name: "feeDestination",
          type: {
            option: "publicKey"
          }
        },
        {
          name: "config",
          type: {
            option: {
              defined: "MarginPoolConfig"
            }
          }
        }
      ]
    },
    {
      name: "collect",
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
          name: "feeDestination",
          isMut: true,
          isSigner: false
        },
        {
          name: "depositNoteMint",
          isMut: true,
          isSigner: false
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: []
    },
    {
      name: "deposit",
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
        },
        {
          name: "depositor",
          isMut: false,
          isSigner: true
        },
        {
          name: "source",
          isMut: true,
          isSigner: false
        },
        {
          name: "destination",
          isMut: true,
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
          name: "amount",
          type: "u64"
        }
      ]
    },
    {
      name: "withdraw",
      accounts: [
        {
          name: "depositor",
          isMut: false,
          isSigner: true
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
          name: "source",
          isMut: true,
          isSigner: false
        },
        {
          name: "destination",
          isMut: true,
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
          name: "amount",
          type: {
            defined: "Amount"
          }
        }
      ]
    },
    {
      name: "marginBorrow",
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: true
        },
        {
          name: "marginPool",
          isMut: true,
          isSigner: false
        },
        {
          name: "loanNoteMint",
          isMut: true,
          isSigner: false
        },
        {
          name: "depositNoteMint",
          isMut: true,
          isSigner: false
        },
        {
          name: "loanAccount",
          isMut: true,
          isSigner: false
        },
        {
          name: "depositAccount",
          isMut: true,
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
          name: "amount",
          type: "u64"
        }
      ]
    },
    {
      name: "marginRepay",
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: true
        },
        {
          name: "marginPool",
          isMut: true,
          isSigner: false
        },
        {
          name: "loanNoteMint",
          isMut: true,
          isSigner: false
        },
        {
          name: "depositNoteMint",
          isMut: true,
          isSigner: false
        },
        {
          name: "loanAccount",
          isMut: true,
          isSigner: false
        },
        {
          name: "depositAccount",
          isMut: true,
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
          name: "maxAmount",
          type: {
            defined: "Amount"
          }
        }
      ]
    },
    {
      name: "marginRefreshPosition",
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: false
        },
        {
          name: "marginPool",
          isMut: false,
          isSigner: false
        },
        {
          name: "tokenPriceOracle",
          isMut: false,
          isSigner: false
        }
      ],
      args: []
    }
  ],
  accounts: [
    {
      name: "marginPool",
      type: {
        kind: "struct",
        fields: [
          {
            name: "version",
            type: "u8"
          },
          {
            name: "poolBump",
            type: {
              array: ["u8", 1]
            }
          },
          {
            name: "vault",
            type: "publicKey"
          },
          {
            name: "feeDestination",
            type: "publicKey"
          },
          {
            name: "depositNoteMint",
            type: "publicKey"
          },
          {
            name: "loanNoteMint",
            type: "publicKey"
          },
          {
            name: "tokenMint",
            type: "publicKey"
          },
          {
            name: "tokenPriceOracle",
            type: "publicKey"
          },
          {
            name: "address",
            type: "publicKey"
          },
          {
            name: "config",
            type: {
              defined: "MarginPoolConfig"
            }
          },
          {
            name: "borrowedTokens",
            type: {
              array: ["u8", 24]
            }
          },
          {
            name: "uncollectedFees",
            type: {
              array: ["u8", 24]
            }
          },
          {
            name: "depositTokens",
            type: "u64"
          },
          {
            name: "depositNotes",
            type: "u64"
          },
          {
            name: "loanNotes",
            type: "u64"
          },
          {
            name: "accruedUntil",
            type: "i64"
          }
        ]
      }
    }
  ],
  types: [
    {
      name: "MarginPoolSummary",
      type: {
        kind: "struct",
        fields: [
          {
            name: "borrowedTokens",
            type: "u64"
          },
          {
            name: "uncollectedFees",
            type: "u64"
          },
          {
            name: "depositTokens",
            type: "u64"
          },
          {
            name: "depositNotes",
            type: "u64"
          },
          {
            name: "loanNotes",
            type: "u64"
          },
          {
            name: "accruedUntil",
            type: "i64"
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
      name: "Amount",
      type: {
        kind: "struct",
        fields: [
          {
            name: "kind",
            type: {
              defined: "AmountKind"
            }
          },
          {
            name: "value",
            type: "u64"
          }
        ]
      }
    },
    {
      name: "PoolAction",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Borrow"
          },
          {
            name: "Deposit"
          },
          {
            name: "Repay"
          },
          {
            name: "Withdraw"
          }
        ]
      }
    },
    {
      name: "RoundingDirection",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Down"
          },
          {
            name: "Up"
          }
        ]
      }
    },
    {
      name: "AmountKind",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Tokens"
          },
          {
            name: "Notes"
          }
        ]
      }
    }
  ],
  events: [
    {
      name: "PoolCreated",
      fields: [
        {
          name: "marginPool",
          type: "publicKey",
          index: false
        },
        {
          name: "vault",
          type: "publicKey",
          index: false
        },
        {
          name: "depositNoteMint",
          type: "publicKey",
          index: false
        },
        {
          name: "loanNoteMint",
          type: "publicKey",
          index: false
        },
        {
          name: "tokenMint",
          type: "publicKey",
          index: false
        },
        {
          name: "authority",
          type: "publicKey",
          index: false
        },
        {
          name: "payer",
          type: "publicKey",
          index: false
        },
        {
          name: "summary",
          type: {
            defined: "MarginPoolSummary"
          },
          index: false
        }
      ]
    },
    {
      name: "PoolConfigured",
      fields: [
        {
          name: "marginPool",
          type: "publicKey",
          index: false
        },
        {
          name: "feeDestination",
          type: "publicKey",
          index: false
        },
        {
          name: "pythProduct",
          type: "publicKey",
          index: false
        },
        {
          name: "pythPrice",
          type: "publicKey",
          index: false
        },
        {
          name: "config",
          type: {
            defined: "MarginPoolConfig"
          },
          index: false
        }
      ]
    },
    {
      name: "Deposit",
      fields: [
        {
          name: "marginPool",
          type: "publicKey",
          index: false
        },
        {
          name: "user",
          type: "publicKey",
          index: false
        },
        {
          name: "source",
          type: "publicKey",
          index: false
        },
        {
          name: "destination",
          type: "publicKey",
          index: false
        },
        {
          name: "depositTokens",
          type: "u64",
          index: false
        },
        {
          name: "depositNotes",
          type: "u64",
          index: false
        },
        {
          name: "summary",
          type: {
            defined: "MarginPoolSummary"
          },
          index: false
        }
      ]
    },
    {
      name: "Withdraw",
      fields: [
        {
          name: "marginPool",
          type: "publicKey",
          index: false
        },
        {
          name: "user",
          type: "publicKey",
          index: false
        },
        {
          name: "source",
          type: "publicKey",
          index: false
        },
        {
          name: "destination",
          type: "publicKey",
          index: false
        },
        {
          name: "withdrawTokens",
          type: "u64",
          index: false
        },
        {
          name: "withdrawNotes",
          type: "u64",
          index: false
        },
        {
          name: "summary",
          type: {
            defined: "MarginPoolSummary"
          },
          index: false
        }
      ]
    },
    {
      name: "MarginBorrow",
      fields: [
        {
          name: "marginPool",
          type: "publicKey",
          index: false
        },
        {
          name: "user",
          type: "publicKey",
          index: false
        },
        {
          name: "loanAccount",
          type: "publicKey",
          index: false
        },
        {
          name: "depositAccount",
          type: "publicKey",
          index: false
        },
        {
          name: "tokens",
          type: "u64",
          index: false
        },
        {
          name: "loanNotes",
          type: "u64",
          index: false
        },
        {
          name: "depositNotes",
          type: "u64",
          index: false
        },
        {
          name: "summary",
          type: {
            defined: "MarginPoolSummary"
          },
          index: false
        }
      ]
    },
    {
      name: "MarginRepay",
      fields: [
        {
          name: "marginPool",
          type: "publicKey",
          index: false
        },
        {
          name: "user",
          type: "publicKey",
          index: false
        },
        {
          name: "loanAccount",
          type: "publicKey",
          index: false
        },
        {
          name: "depositAccount",
          type: "publicKey",
          index: false
        },
        {
          name: "maxRepayTokens",
          type: "u64",
          index: false
        },
        {
          name: "maxRepayNotes",
          type: "u64",
          index: false
        },
        {
          name: "repaidTokens",
          type: "u64",
          index: false
        },
        {
          name: "repaidLoanNotes",
          type: "u64",
          index: false
        },
        {
          name: "repaidDepositNotes",
          type: "u64",
          index: false
        },
        {
          name: "summary",
          type: {
            defined: "MarginPoolSummary"
          },
          index: false
        }
      ]
    },
    {
      name: "Collect",
      fields: [
        {
          name: "marginPool",
          type: "publicKey",
          index: false
        },
        {
          name: "feeNotesMinted",
          type: "u64",
          index: false
        },
        {
          name: "feeTokensClaimed",
          type: "u64",
          index: false
        },
        {
          name: "feeNotesBalance",
          type: "u64",
          index: false
        },
        {
          name: "feeTokensBalance",
          type: "u64",
          index: false
        },
        {
          name: "summary",
          type: {
            defined: "MarginPoolSummary"
          },
          index: false
        }
      ]
    }
  ],
  errors: [
    {
      code: 141100,
      name: "Disabled",
      msg: "The pool is currently disabled"
    },
    {
      code: 141101,
      name: "InterestAccrualBehind",
      msg: "Interest accrual is too far behind"
    },
    {
      code: 141102,
      name: "DepositsOnly",
      msg: "The pool currently only allows deposits"
    },
    {
      code: 141103,
      name: "InsufficientLiquidity",
      msg: "The pool does not have sufficient liquidity for the transaction"
    },
    {
      code: 141104,
      name: "InvalidAmount",
      msg: "An invalid amount has been supplied"
    },
    {
      code: 141105,
      name: "InvalidPrice"
    },
    {
      code: 141106,
      name: "InvalidOracle"
    },
    {
      code: 141107,
      name: "RepaymentExceedsTotalOutstanding"
    }
  ]
}
