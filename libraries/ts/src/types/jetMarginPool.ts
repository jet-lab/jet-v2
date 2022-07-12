export type JetMarginPool = {
  version: "0.1.0"
  name: "jet_margin_pool"
  instructions: [
    {
      name: "createPool"
      docs: ["Create a new pool for borrowing and lending"]
      accounts: [
        {
          name: "marginPool"
          isMut: true
          isSigner: false
          docs: ["The pool to be created"]
        },
        {
          name: "vault"
          isMut: true
          isSigner: false
          docs: ["The token account holding the pool's deposited funds"]
        },
        {
          name: "depositNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for deposit notes"]
        },
        {
          name: "loanNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for loan notes"]
        },
        {
          name: "tokenMint"
          isMut: false
          isSigner: false
          docs: ["The mint for the token being custodied by the pool"]
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority to create pools, which must sign"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["The payer of rent for new accounts"]
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
      args: [
        {
          name: "feeDestination"
          type: "publicKey"
        }
      ]
    },
    {
      name: "configure"
      docs: ["Configure an existing pool"]
      accounts: [
        {
          name: "marginPool"
          isMut: true
          isSigner: false
          docs: ["The pool to be configured"]
        },
        {
          name: "authority"
          isMut: false
          isSigner: false
          docs: ["The authority allowed to modify the pool, which must sign"]
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
      docs: ["Accrue interest on the pool, and collect any fees."]
      accounts: [
        {
          name: "marginPool"
          isMut: true
          isSigner: false
          docs: ["The pool to be refreshed"]
        },
        {
          name: "vault"
          isMut: true
          isSigner: false
          docs: ["The vault for the pool, where tokens are held"]
        },
        {
          name: "feeDestination"
          isMut: true
          isSigner: false
          docs: ["The account to deposit the collected fees"]
        },
        {
          name: "depositNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for the deposit notes"]
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
      docs: ["Deposit tokens into the pool in exchange for notes"]
      accounts: [
        {
          name: "marginPool"
          isMut: true
          isSigner: false
          docs: ["The pool to deposit into"]
        },
        {
          name: "vault"
          isMut: true
          isSigner: false
          docs: ["The vault for the pool, where tokens are held"]
        },
        {
          name: "depositNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for the deposit notes"]
        },
        {
          name: "depositor"
          isMut: false
          isSigner: true
          docs: ["The address with authority to deposit the tokens"]
        },
        {
          name: "source"
          isMut: true
          isSigner: false
          docs: ["The source of the tokens to be deposited"]
        },
        {
          name: "destination"
          isMut: true
          isSigner: false
          docs: ["The destination of the deposit notes"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount"
          type: "u64"
        }
      ]
    },
    {
      name: "withdraw"
      docs: ["Withdraw tokens from the pool, exchanging in previously received", "deposit notes."]
      accounts: [
        {
          name: "depositor"
          isMut: false
          isSigner: true
          docs: ["The address with authority to withdraw the deposit"]
        },
        {
          name: "marginPool"
          isMut: true
          isSigner: false
          docs: ["The pool to withdraw from"]
        },
        {
          name: "vault"
          isMut: true
          isSigner: false
          docs: ["The vault for the pool, where tokens are held"]
        },
        {
          name: "depositNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for the deposit notes"]
        },
        {
          name: "source"
          isMut: true
          isSigner: false
          docs: ["The source of the deposit notes to be redeemed"]
        },
        {
          name: "destination"
          isMut: true
          isSigner: false
          docs: ["The destination of the tokens withdrawn"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount"
          type: "u64"
        }
      ]
    },
    {
      name: "marginBorrow"
      docs: ["Borrow tokens using a margin account"]
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The margin account being executed on"]
        },
        {
          name: "marginPool"
          isMut: true
          isSigner: false
          docs: ["The pool to borrow from"]
        },
        {
          name: "loanNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "depositNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for the notes representing deposit into the pool"]
        },
        {
          name: "loanAccount"
          isMut: true
          isSigner: false
          docs: ["The account to receive the loan notes"]
        },
        {
          name: "depositAccount"
          isMut: true
          isSigner: false
          docs: ["The account to receive the borrowed tokens (as deposit notes)"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount"
          type: "u64"
        }
      ]
    },
    {
      name: "marginRepay"
      docs: [
        "Repay a loan with a maximum amount.",
        "If the loan balance is lower than the amount, the excess is left in the",
        "deposit account."
      ]
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The margin account being executed on"]
        },
        {
          name: "marginPool"
          isMut: true
          isSigner: false
          docs: ["The pool with the outstanding loan"]
        },
        {
          name: "loanNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "depositNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for the notes representing deposit into the pool"]
        },
        {
          name: "loanAccount"
          isMut: true
          isSigner: false
          docs: ["The account with the loan notes"]
        },
        {
          name: "depositAccount"
          isMut: true
          isSigner: false
          docs: ["The account with the deposit to pay off the loan with"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount"
          type: "u64"
        }
      ]
    },
    {
      name: "marginRepayFromWallet"
      docs: ["Repay a margin account debt from an outside token account"]
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The margin account being executed on"]
        },
        {
          name: "marginPool"
          isMut: true
          isSigner: false
          docs: ["The pool with the outstanding loan"]
        },
        {
          name: "loanNoteMint"
          isMut: true
          isSigner: false
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "poolVault"
          isMut: true
          isSigner: false
          docs: ["The vault responsible for storing the pool's tokens"]
        },
        {
          name: "loanAccount"
          isMut: true
          isSigner: false
          docs: ["The account with the loan notes"]
        },
        {
          name: "repaymentTokenAccount"
          isMut: true
          isSigner: false
          docs: ["The token account repaying the debt"]
        },
        {
          name: "repaymentAccountAuthority"
          isMut: false
          isSigner: true
          docs: ["Signing authority for the repaying token account"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount"
          type: "u64"
        }
      ]
    },
    {
      name: "marginRefreshPosition"
      docs: ["Update the pool position on a margin account"]
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: false
          docs: ["The margin account being executed on"]
        },
        {
          name: "marginPool"
          isMut: false
          isSigner: false
          docs: ["The pool to be refreshed"]
        },
        {
          name: "tokenPriceOracle"
          isMut: false
          isSigner: false
          docs: ["The pyth price account for the pool's token"]
        }
      ]
      args: []
    },
    {
      name: "registerLoan"
      docs: ["Creates the token account to track the loan notes,", "then requests margin to register the position"]
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
        },
        {
          name: "positionTokenMetadata"
          isMut: false
          isSigner: false
          docs: [
            "This will be required for margin to register the position,",
            "so requiring it here makes it easier for clients to ensure",
            "that it will be sent."
          ]
        },
        {
          name: "loanNoteAccount"
          isMut: true
          isSigner: false
          docs: ["The token account to store the loan notes representing the claim", "against the margin account"]
        },
        {
          name: "loanNoteMint"
          isMut: false
          isSigner: false
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "marginPool"
          isMut: false
          isSigner: false
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
      name: "closeLoan"
      docs: ["Closes a previously opened loan token account"]
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
        },
        {
          name: "loanNoteAccount"
          isMut: true
          isSigner: false
          docs: ["The token account to store the loan notes representing the claim", "against the margin account"]
        },
        {
          name: "loanNoteMint"
          isMut: false
          isSigner: false
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "marginPool"
          isMut: false
          isSigner: false
        },
        {
          name: "beneficiary"
          isMut: true
          isSigner: true
        },
        {
          name: "tokenProgram"
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
      docs: ["Account containing information about a margin pool, which", "services lending/borrowing operations."]
      type: {
        kind: "struct"
        fields: [
          {
            name: "version"
            type: "u8"
          },
          {
            name: "poolBump"
            docs: ["The bump seed used to create the pool address"]
            type: {
              array: ["u8", 1]
            }
          },
          {
            name: "vault"
            docs: ["The address of the vault account, which has custody of the", "pool's tokens"]
            type: "publicKey"
          },
          {
            name: "feeDestination"
            docs: ["The address of the account to deposit collected fees, represented as", "deposit notes"]
            type: "publicKey"
          },
          {
            name: "depositNoteMint"
            docs: ["The address of the mint for deposit notes"]
            type: "publicKey"
          },
          {
            name: "loanNoteMint"
            docs: ["The address of the mint for the loan notes"]
            type: "publicKey"
          },
          {
            name: "tokenMint"
            docs: ["The token the pool allows lending and borrowing on"]
            type: "publicKey"
          },
          {
            name: "tokenPriceOracle"
            docs: ["The address of the pyth oracle with price information for the token"]
            type: "publicKey"
          },
          {
            name: "address"
            docs: ["The address of this pool"]
            type: "publicKey"
          },
          {
            name: "config"
            docs: ["The configuration of the pool"]
            type: {
              defined: "MarginPoolConfig"
            }
          },
          {
            name: "borrowedTokens"
            docs: ["The total amount of tokens borrowed, that need to be repaid to", "the pool."]
            type: {
              array: ["u8", 24]
            }
          },
          {
            name: "uncollectedFees"
            docs: ["The total amount of tokens in the pool that's reserved for collection", "as fees."]
            type: {
              array: ["u8", 24]
            }
          },
          {
            name: "depositTokens"
            docs: ["The total amount of tokens available in the pool's vault"]
            type: "u64"
          },
          {
            name: "depositNotes"
            docs: ["The total amount of notes issued to depositors of tokens."]
            type: "u64"
          },
          {
            name: "loanNotes"
            docs: ["The total amount of notes issued to borrowers of tokens"]
            type: "u64"
          },
          {
            name: "accruedUntil"
            docs: ["The time the interest was last accrued up to"]
            type: "i64"
          }
        ]
      }
    }
  ]
  types: [
    {
      name: "MarginPoolSummary"
      docs: ["Common fields from MarginPool for event logging."]
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
      docs: ["Configuration for a margin pool"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "flags"
            docs: ["Space for binary settings"]
            type: "u64"
          },
          {
            name: "utilizationRate1"
            docs: ["The utilization rate at which first regime transitions to second"]
            type: "u16"
          },
          {
            name: "utilizationRate2"
            docs: ["The utilization rate at which second regime transitions to third"]
            type: "u16"
          },
          {
            name: "borrowRate0"
            docs: ["The lowest borrow rate"]
            type: "u16"
          },
          {
            name: "borrowRate1"
            docs: ["The borrow rate at the transition point from first to second regime"]
            type: "u16"
          },
          {
            name: "borrowRate2"
            docs: ["The borrow rate at the transition point from second to third regime"]
            type: "u16"
          },
          {
            name: "borrowRate3"
            docs: ["The highest possible borrow rate."]
            type: "u16"
          },
          {
            name: "managementFeeRate"
            docs: ["The fee rate applied to interest payments collected"]
            type: "u16"
          },
          {
            name: "managementFeeCollectThreshold"
            docs: ["The threshold for fee collection"]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "TokenChange"
      docs: ["Interface for changing the token value of an account through pool instructions"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "kind"
            type: {
              defined: "ChangeKind"
            }
          },
          {
            name: "tokens"
            type: "u64"
          }
        ]
      }
    },
    {
      name: "Amount"
      docs: ["Represent an amount of some value (like tokens, or notes)"]
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
      docs: [
        "Represents the primary pool actions, used in determining the",
        "rounding direction between tokens and notes."
      ]
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
      docs: ["Represents the direction in which we should round when converting", "between tokens and notes."]
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
          name: "feeDestination"
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
      name: "Repay"
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
          name: "repaymentTokenAccount"
          type: "publicKey"
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
      name: "InvalidSetTo"
      msg: "An invalid `SetTo` value was given for a `TokenChange`"
    },
    {
      code: 141108
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
      docs: ["Create a new pool for borrowing and lending"],
      accounts: [
        {
          name: "marginPool",
          isMut: true,
          isSigner: false,
          docs: ["The pool to be created"]
        },
        {
          name: "vault",
          isMut: true,
          isSigner: false,
          docs: ["The token account holding the pool's deposited funds"]
        },
        {
          name: "depositNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for deposit notes"]
        },
        {
          name: "loanNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for loan notes"]
        },
        {
          name: "tokenMint",
          isMut: false,
          isSigner: false,
          docs: ["The mint for the token being custodied by the pool"]
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: ["The authority to create pools, which must sign"]
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The payer of rent for new accounts"]
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
      args: [
        {
          name: "feeDestination",
          type: "publicKey"
        }
      ]
    },
    {
      name: "configure",
      docs: ["Configure an existing pool"],
      accounts: [
        {
          name: "marginPool",
          isMut: true,
          isSigner: false,
          docs: ["The pool to be configured"]
        },
        {
          name: "authority",
          isMut: false,
          isSigner: false,
          docs: ["The authority allowed to modify the pool, which must sign"]
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
      docs: ["Accrue interest on the pool, and collect any fees."],
      accounts: [
        {
          name: "marginPool",
          isMut: true,
          isSigner: false,
          docs: ["The pool to be refreshed"]
        },
        {
          name: "vault",
          isMut: true,
          isSigner: false,
          docs: ["The vault for the pool, where tokens are held"]
        },
        {
          name: "feeDestination",
          isMut: true,
          isSigner: false,
          docs: ["The account to deposit the collected fees"]
        },
        {
          name: "depositNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for the deposit notes"]
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
      docs: ["Deposit tokens into the pool in exchange for notes"],
      accounts: [
        {
          name: "marginPool",
          isMut: true,
          isSigner: false,
          docs: ["The pool to deposit into"]
        },
        {
          name: "vault",
          isMut: true,
          isSigner: false,
          docs: ["The vault for the pool, where tokens are held"]
        },
        {
          name: "depositNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for the deposit notes"]
        },
        {
          name: "depositor",
          isMut: false,
          isSigner: true,
          docs: ["The address with authority to deposit the tokens"]
        },
        {
          name: "source",
          isMut: true,
          isSigner: false,
          docs: ["The source of the tokens to be deposited"]
        },
        {
          name: "destination",
          isMut: true,
          isSigner: false,
          docs: ["The destination of the deposit notes"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount",
          type: "u64"
        }
      ]
    },
    {
      name: "withdraw",
      docs: ["Withdraw tokens from the pool, exchanging in previously received", "deposit notes."],
      accounts: [
        {
          name: "depositor",
          isMut: false,
          isSigner: true,
          docs: ["The address with authority to withdraw the deposit"]
        },
        {
          name: "marginPool",
          isMut: true,
          isSigner: false,
          docs: ["The pool to withdraw from"]
        },
        {
          name: "vault",
          isMut: true,
          isSigner: false,
          docs: ["The vault for the pool, where tokens are held"]
        },
        {
          name: "depositNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for the deposit notes"]
        },
        {
          name: "source",
          isMut: true,
          isSigner: false,
          docs: ["The source of the deposit notes to be redeemed"]
        },
        {
          name: "destination",
          isMut: true,
          isSigner: false,
          docs: ["The destination of the tokens withdrawn"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount",
          type: "u64"
        }
      ]
    },
    {
      name: "marginBorrow",
      docs: ["Borrow tokens using a margin account"],
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: true,
          docs: ["The margin account being executed on"]
        },
        {
          name: "marginPool",
          isMut: true,
          isSigner: false,
          docs: ["The pool to borrow from"]
        },
        {
          name: "loanNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "depositNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for the notes representing deposit into the pool"]
        },
        {
          name: "loanAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account to receive the loan notes"]
        },
        {
          name: "depositAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account to receive the borrowed tokens (as deposit notes)"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount",
          type: "u64"
        }
      ]
    },
    {
      name: "marginRepay",
      docs: [
        "Repay a loan with a maximum amount.",
        "If the loan balance is lower than the amount, the excess is left in the",
        "deposit account."
      ],
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: true,
          docs: ["The margin account being executed on"]
        },
        {
          name: "marginPool",
          isMut: true,
          isSigner: false,
          docs: ["The pool with the outstanding loan"]
        },
        {
          name: "loanNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "depositNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for the notes representing deposit into the pool"]
        },
        {
          name: "loanAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account with the loan notes"]
        },
        {
          name: "depositAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account with the deposit to pay off the loan with"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount",
          type: "u64"
        }
      ]
    },
    {
      name: "marginRepayFromWallet",
      docs: ["Repay a margin account debt from an outside token account"],
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: true,
          docs: ["The margin account being executed on"]
        },
        {
          name: "marginPool",
          isMut: true,
          isSigner: false,
          docs: ["The pool with the outstanding loan"]
        },
        {
          name: "loanNoteMint",
          isMut: true,
          isSigner: false,
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "poolVault",
          isMut: true,
          isSigner: false,
          docs: ["The vault responsible for storing the pool's tokens"]
        },
        {
          name: "loanAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account with the loan notes"]
        },
        {
          name: "repaymentTokenAccount",
          isMut: true,
          isSigner: false,
          docs: ["The token account repaying the debt"]
        },
        {
          name: "repaymentAccountAuthority",
          isMut: false,
          isSigner: true,
          docs: ["Signing authority for the repaying token account"]
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
          type: { defined: "ChangeKind" }
        },
        {
          name: "amount",
          type: "u64"
        }
      ]
    },
    {
      name: "marginRefreshPosition",
      docs: ["Update the pool position on a margin account"],
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: false,
          docs: ["The margin account being executed on"]
        },
        {
          name: "marginPool",
          isMut: false,
          isSigner: false,
          docs: ["The pool to be refreshed"]
        },
        {
          name: "tokenPriceOracle",
          isMut: false,
          isSigner: false,
          docs: ["The pyth price account for the pool's token"]
        }
      ],
      args: []
    },
    {
      name: "registerLoan",
      docs: ["Creates the token account to track the loan notes,", "then requests margin to register the position"],
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: true
        },
        {
          name: "positionTokenMetadata",
          isMut: false,
          isSigner: false,
          docs: [
            "This will be required for margin to register the position,",
            "so requiring it here makes it easier for clients to ensure",
            "that it will be sent."
          ]
        },
        {
          name: "loanNoteAccount",
          isMut: true,
          isSigner: false,
          docs: ["The token account to store the loan notes representing the claim", "against the margin account"]
        },
        {
          name: "loanNoteMint",
          isMut: false,
          isSigner: false,
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "marginPool",
          isMut: false,
          isSigner: false
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
      name: "closeLoan",
      docs: ["Closes a previously opened loan token account"],
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: true
        },
        {
          name: "loanNoteAccount",
          isMut: true,
          isSigner: false,
          docs: ["The token account to store the loan notes representing the claim", "against the margin account"]
        },
        {
          name: "loanNoteMint",
          isMut: false,
          isSigner: false,
          docs: ["The mint for the notes representing loans from the pool"]
        },
        {
          name: "marginPool",
          isMut: false,
          isSigner: false
        },
        {
          name: "beneficiary",
          isMut: true,
          isSigner: true
        },
        {
          name: "tokenProgram",
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
      docs: ["Account containing information about a margin pool, which", "services lending/borrowing operations."],
      type: {
        kind: "struct",
        fields: [
          {
            name: "version",
            type: "u8"
          },
          {
            name: "poolBump",
            docs: ["The bump seed used to create the pool address"],
            type: {
              array: ["u8", 1]
            }
          },
          {
            name: "vault",
            docs: ["The address of the vault account, which has custody of the", "pool's tokens"],
            type: "publicKey"
          },
          {
            name: "feeDestination",
            docs: ["The address of the account to deposit collected fees, represented as", "deposit notes"],
            type: "publicKey"
          },
          {
            name: "depositNoteMint",
            docs: ["The address of the mint for deposit notes"],
            type: "publicKey"
          },
          {
            name: "loanNoteMint",
            docs: ["The address of the mint for the loan notes"],
            type: "publicKey"
          },
          {
            name: "tokenMint",
            docs: ["The token the pool allows lending and borrowing on"],
            type: "publicKey"
          },
          {
            name: "tokenPriceOracle",
            docs: ["The address of the pyth oracle with price information for the token"],
            type: "publicKey"
          },
          {
            name: "address",
            docs: ["The address of this pool"],
            type: "publicKey"
          },
          {
            name: "config",
            docs: ["The configuration of the pool"],
            type: {
              defined: "MarginPoolConfig"
            }
          },
          {
            name: "borrowedTokens",
            docs: ["The total amount of tokens borrowed, that need to be repaid to", "the pool."],
            type: {
              array: ["u8", 24]
            }
          },
          {
            name: "uncollectedFees",
            docs: ["The total amount of tokens in the pool that's reserved for collection", "as fees."],
            type: {
              array: ["u8", 24]
            }
          },
          {
            name: "depositTokens",
            docs: ["The total amount of tokens available in the pool's vault"],
            type: "u64"
          },
          {
            name: "depositNotes",
            docs: ["The total amount of notes issued to depositors of tokens."],
            type: "u64"
          },
          {
            name: "loanNotes",
            docs: ["The total amount of notes issued to borrowers of tokens"],
            type: "u64"
          },
          {
            name: "accruedUntil",
            docs: ["The time the interest was last accrued up to"],
            type: "i64"
          }
        ]
      }
    }
  ],
  types: [
    {
      name: "MarginPoolSummary",
      docs: ["Common fields from MarginPool for event logging."],
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
      docs: ["Configuration for a margin pool"],
      type: {
        kind: "struct",
        fields: [
          {
            name: "flags",
            docs: ["Space for binary settings"],
            type: "u64"
          },
          {
            name: "utilizationRate1",
            docs: ["The utilization rate at which first regime transitions to second"],
            type: "u16"
          },
          {
            name: "utilizationRate2",
            docs: ["The utilization rate at which second regime transitions to third"],
            type: "u16"
          },
          {
            name: "borrowRate0",
            docs: ["The lowest borrow rate"],
            type: "u16"
          },
          {
            name: "borrowRate1",
            docs: ["The borrow rate at the transition point from first to second regime"],
            type: "u16"
          },
          {
            name: "borrowRate2",
            docs: ["The borrow rate at the transition point from second to third regime"],
            type: "u16"
          },
          {
            name: "borrowRate3",
            docs: ["The highest possible borrow rate."],
            type: "u16"
          },
          {
            name: "managementFeeRate",
            docs: ["The fee rate applied to interest payments collected"],
            type: "u16"
          },
          {
            name: "managementFeeCollectThreshold",
            docs: ["The threshold for fee collection"],
            type: "u64"
          }
        ]
      }
    },
    {
      name: "TokenChange",
      docs: ["Interface for changing the token value of an account through pool instructions"],
      type: {
        kind: "struct",
        fields: [
          {
            name: "kind",
            type: {
              defined: "ChangeKind"
            }
          },
          {
            name: "tokens",
            type: "u64"
          }
        ]
      }
    },
    {
      name: "Amount",
      docs: ["Represent an amount of some value (like tokens, or notes)"],
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
      docs: [
        "Represents the primary pool actions, used in determining the",
        "rounding direction between tokens and notes."
      ],
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
      docs: ["Represents the direction in which we should round when converting", "between tokens and notes."],
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
          name: "feeDestination",
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
      name: "Repay",
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
          name: "repaymentTokenAccount",
          type: "publicKey",
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
      name: "InvalidSetTo",
      msg: "An invalid `SetTo` value was given for a `TokenChange`"
    },
    {
      code: 141108,
      name: "RepaymentExceedsTotalOutstanding"
    }
  ]
}
