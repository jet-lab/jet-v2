export type JetBonds = {
  "version": "0.1.0",
  "name": "jet_bonds",
  "constants": [
    {
      "name": "BOND_MANAGER",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"bond_manager\""
    },
    {
      "name": "BOND_TICKET_ACCOUNT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"bond_ticket_account\""
    },
    {
      "name": "BOND_TICKET_MINT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"bond_ticket_mint\""
    },
    {
      "name": "CLAIM_TICKET",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"claim_ticket\""
    },
    {
      "name": "CRANK_AUTHORIZATION",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"crank_authorization\""
    },
    {
      "name": "CLAIM_NOTES",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"claim_notes\""
    },
    {
      "name": "COLLATERAL_NOTES",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"collateral_notes\""
    },
    {
      "name": "SPLIT_TICKET",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"split_ticket\""
    },
    {
      "name": "EVENT_ADAPTER",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"event_adapter\""
    },
    {
      "name": "OBLIGATION",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"obligation\""
    },
    {
      "name": "ORDERBOOK_MARKET_STATE",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"orderbook_market_state\""
    },
    {
      "name": "MARGIN_BORROWER",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"margin_borrower\""
    },
    {
      "name": "UNDERLYING_TOKEN_VAULT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"underlying_token_vault\""
    }
  ],
  "instructions": [
    {
      "name": "authorizeCrank",
      "accounts": [
        {
          "name": "crank",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "crankAuthorization",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "revokeCrank",
      "accounts": [
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "receiver",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "initializeBondManager",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingOracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ticketOracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "InitializeBondManagerParams"
          }
        }
      ]
    },
    {
      "name": "initializeOrderbook",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "InitializeOrderbookParams"
          }
        }
      ]
    },
    {
      "name": "modifyBondManager",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "data",
          "type": "bytes"
        },
        {
          "name": "offset",
          "type": "u32"
        }
      ]
    },
    {
      "name": "pauseOrderMatching",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "resumeOrderMatching",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "initializeMarginUser",
      "accounts": [
        {
          "name": "borrowerAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "claimsMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingSettlement",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ticketSettlement",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "claimsMetadata",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "collateralMetadata",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "marginBorrowOrder",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "obligation",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "claimsMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "bondManager",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMarketState",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "eventQueue",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bids",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "asks",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
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
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        },
        {
          "name": "seed",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "marginSellTicketsOrder",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "authority",
              "isMut": false,
              "isSigner": true
            },
            {
              "name": "userTicketVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "userTokenVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMut",
              "accounts": [
                {
                  "name": "bondManager",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "orderbookMarketState",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "eventQueue",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "bids",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "asks",
                  "isMut": true,
                  "isSigner": false
                }
              ]
            },
            {
              "name": "bondTicketMint",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "tokenProgram",
              "isMut": false,
              "isSigner": false
            }
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        }
      ]
    },
    {
      "name": "marginRedeemTicket",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "ticket",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "authority",
              "isMut": true,
              "isSigner": true
            },
            {
              "name": "claimantTokenAccount",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bondManager",
              "isMut": false,
              "isSigner": false
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "tokenProgram",
              "isMut": false,
              "isSigner": false
            }
          ]
        }
      ],
      "args": []
    },
    {
      "name": "marginLendOrder",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "authority",
              "isMut": false,
              "isSigner": true
            },
            {
              "name": "orderbookMut",
              "accounts": [
                {
                  "name": "bondManager",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "orderbookMarketState",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "eventQueue",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "bids",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "asks",
                  "isMut": true,
                  "isSigner": false
                }
              ]
            },
            {
              "name": "ticketSettlement",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "lenderTokens",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "ticketMint",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "payer",
              "isMut": true,
              "isSigner": true
            },
            {
              "name": "systemProgram",
              "isMut": false,
              "isSigner": false
            },
            {
              "name": "tokenProgram",
              "isMut": false,
              "isSigner": false
            }
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        },
        {
          "name": "seed",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "refreshPosition",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingOracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ticketOracle",
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
          "name": "expectPrice",
          "type": "bool"
        }
      ]
    },
    {
      "name": "repay",
      "accounts": [
        {
          "name": "borrowerAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "obligation",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "nextObligation",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "source",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
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
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "settle",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "claimsMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingSettlement",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ticketSettlement",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "sellTicketsOrder",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "userTicketVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "bondManager",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMarketState",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "eventQueue",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bids",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "asks",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
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
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        }
      ]
    },
    {
      "name": "cancelOrder",
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "bondManager",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMarketState",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "eventQueue",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bids",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "asks",
              "isMut": true,
              "isSigner": false
            }
          ]
        }
      ],
      "args": [
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "lendOrder",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "bondManager",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMarketState",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "eventQueue",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bids",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "asks",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "ticketSettlement",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "lenderTokens",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ticketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
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
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        },
        {
          "name": "seed",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "consumeEvents",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "crankAuthorization",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "crank",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
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
          "name": "numEvents",
          "type": "u32"
        },
        {
          "name": "seedBytes",
          "type": {
            "vec": "bytes"
          }
        }
      ]
    },
    {
      "name": "exchangeTokens",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userBondTicketVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userUnderlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "redeemTicket",
      "accounts": [
        {
          "name": "ticket",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "claimantTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "stakeBondTickets",
      "accounts": [
        {
          "name": "claimTicket",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ticketHolder",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "bondTicketTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "StakeBondTicketsParams"
          }
        }
      ]
    },
    {
      "name": "tranferTicketOwnership",
      "accounts": [
        {
          "name": "ticket",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "currentOwner",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "newOwner",
          "type": "publicKey"
        }
      ]
    },
    {
      "name": "registerAdapter",
      "accounts": [
        {
          "name": "adapterQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "RegisterAdapterParams"
          }
        }
      ]
    },
    {
      "name": "popAdapterEvents",
      "accounts": [
        {
          "name": "adapterQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "numEvents",
          "type": "u32"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "BondManager",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "versionTag",
            "type": "u64"
          },
          {
            "name": "airspace",
            "type": "publicKey"
          },
          {
            "name": "orderbookMarketState",
            "type": "publicKey"
          },
          {
            "name": "eventQueue",
            "type": "publicKey"
          },
          {
            "name": "asks",
            "type": "publicKey"
          },
          {
            "name": "bids",
            "type": "publicKey"
          },
          {
            "name": "underlyingTokenMint",
            "type": "publicKey"
          },
          {
            "name": "underlyingTokenVault",
            "type": "publicKey"
          },
          {
            "name": "bondTicketMint",
            "type": "publicKey"
          },
          {
            "name": "claimsMint",
            "type": "publicKey"
          },
          {
            "name": "collateralMint",
            "type": "publicKey"
          },
          {
            "name": "underlyingOracle",
            "type": "publicKey"
          },
          {
            "name": "ticketOracle",
            "type": "publicKey"
          },
          {
            "name": "seed",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "bump",
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "orderbookPaused",
            "type": "bool"
          },
          {
            "name": "ticketsPaused",
            "type": "bool"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                28
              ]
            }
          },
          {
            "name": "borrowDuration",
            "type": "i64"
          },
          {
            "name": "lendDuration",
            "type": "i64"
          },
          {
            "name": "nonce",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "CrankAuthorization",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "crank",
            "type": "publicKey"
          },
          {
            "name": "airspace",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "MarginUser",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "type": "u8"
          },
          {
            "name": "marginAccount",
            "type": "publicKey"
          },
          {
            "name": "bondManager",
            "type": "publicKey"
          },
          {
            "name": "claims",
            "type": "publicKey"
          },
          {
            "name": "collateral",
            "type": "publicKey"
          },
          {
            "name": "underlyingSettlement",
            "type": "publicKey"
          },
          {
            "name": "ticketSettlement",
            "type": "publicKey"
          },
          {
            "name": "debt",
            "type": {
              "defined": "Debt"
            }
          },
          {
            "name": "assets",
            "type": {
              "defined": "Assets"
            }
          }
        ]
      }
    },
    {
      "name": "Obligation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "sequenceNumber",
            "type": "u64"
          },
          {
            "name": "borrowerAccount",
            "type": "publicKey"
          },
          {
            "name": "bondManager",
            "type": "publicKey"
          },
          {
            "name": "orderTag",
            "type": {
              "array": ["u8", 16]
            }
          },
          {
            "name": "maturationTimestamp",
            "type": "u64"
          },
          {
            "name": "balance",
            "type": "u64"
          },
          {
            "name": "flags",
            "docs": [
              "Any boolean flags for this data type compressed to a single byte"
            ],
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "EventAdapterMetadata",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "manager",
            "type": "publicKey"
          },
          {
            "name": "orderbookUser",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "ClaimTicket",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "bondManager",
            "type": "publicKey"
          },
          {
            "name": "maturationTimestamp",
            "type": "i64"
          },
          {
            "name": "redeemable",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "SplitTicket",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "bondManager",
            "type": "publicKey"
          },
          {
            "name": "orderTag",
            "type": {
              "array": ["u8", 16]
            }
          },
          {
            "name": "struckTimestamp",
            "type": "i64"
          },
          {
            "name": "maturationTimestamp",
            "type": "i64"
          },
          {
            "name": "principal",
            "type": "u64"
          },
          {
            "name": "interest",
            "type": "u64"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "InitializeBondManagerParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "versionTag",
            "type": "u64"
          },
          {
            "name": "seed",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "borrowDuration",
            "type": "i64"
          },
          {
            "name": "lendDuration",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "InitializeOrderbookParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "minBaseOrderSize",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "Debt",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "nextNewObligationSeqno",
            "type": "u64"
          },
          {
            "name": "nextUnpaidObligationSeqno",
            "type": "u64"
          },
          {
            "name": "nextObligationMaturity",
            "type": "u64"
          },
          {
            "name": "pending",
            "type": "u64"
          },
          {
            "name": "committed",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "Assets",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "entitledTokens",
            "type": "u64"
          },
          {
            "name": "entitledTickets",
            "type": "u64"
          },
          {
            "name": "ticketsStaked",
            "type": "u64"
          },
          {
            "name": "postedQuote",
            "type": "u64"
          },
          {
            "name": "reserved0",
            "type": {
              "array": [
                "u8",
                64
              ]
            }
          }
        ]
      }
    },
    {
      "name": "RegisterAdapterParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "numEvents",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "OrderParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxBondTicketQty",
            "type": "u64"
          },
          {
            "name": "maxUnderlyingTokenQty",
            "type": "u64"
          },
          {
            "name": "limitPrice",
            "type": "u64"
          },
          {
            "name": "matchLimit",
            "type": "u64"
          },
          {
            "name": "postOnly",
            "type": "bool"
          },
          {
            "name": "postAllowed",
            "type": "bool"
          },
          {
            "name": "autoStake",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "StakeBondTicketsParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "ticketSeed",
            "type": "bytes"
          }
        ]
      }
    },
    {
      "name": "OrderType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "MarginBorrow"
          },
          {
            "name": "MarginLend"
          },
          {
            "name": "MarginSellTickets"
          },
          {
            "name": "Lend"
          },
          {
            "name": "SellTickets"
          }
        ]
      }
    },
    {
      "name": "EventAccounts",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill",
            "fields": [
              {
                "defined": "Box<FillAccounts<'info>>"
              }
            ]
          },
          {
            "name": "Out",
            "fields": [
              {
                "defined": "Box<OutAccounts<'info>>"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "LoanAccount",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "AutoStake",
            "fields": [
              {
                "defined": "AnchorAccount<'info,SplitTicket,Mut>"
              }
            ]
          },
          {
            "name": "NewDebt",
            "fields": [
              {
                "defined": "AnchorAccount<'info,Obligation,Mut>"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "EventTag",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill"
          },
          {
            "name": "Out"
          }
        ]
      }
    },
    {
      "name": "OrderbookEvent",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill",
            "fields": [
              {
                "defined": "FillInfo"
              }
            ]
          },
          {
            "name": "Out",
            "fields": [
              {
                "defined": "OutInfo"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "TicketKind",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Claim",
            "fields": [
              {
                "defined": "Account<'info,ClaimTicket>"
              }
            ]
          },
          {
            "name": "Split",
            "fields": [
              {
                "defined": "Account<'info,SplitTicket>"
              }
            ]
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "BondManagerInitialized",
      "fields": [
        {
          "name": "version",
          "type": "u64",
          "index": false
        },
        {
          "name": "address",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "airspace",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "underlyingTokenMint",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "underlyingOracle",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "ticketOracle",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "borrowDuration",
          "type": "i64",
          "index": false
        },
        {
          "name": "lendDuration",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "OrderbookInitialized",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "orderbookMarketState",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "eventQueue",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "bids",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "asks",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "minBaseOrderSize",
          "type": "u64",
          "index": false
        },
        {
          "name": "tickSize",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "PositionRefreshed",
      "fields": [
        {
          "name": "borrowerAccount",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "ToggleOrderMatching",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "isOrderbookPaused",
          "type": "bool",
          "index": false
        }
      ]
    },
    {
      "name": "SkippedError",
      "fields": [
        {
          "name": "message",
          "type": "string",
          "index": false
        }
      ]
    },
    {
      "name": "MarginUserInitialized",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "borrowerAccount",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "marginAccount",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "underlyingSettlement",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "ticketSettlement",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "OrderPlaced",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "authority",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "marginUser",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "orderType",
          "type": {
            "defined": "OrderType"
          },
          "index": false
        },
        // {
        //   "name": "orderSummary",
        //   "type": {
        //     "defined": "OrderSummary"
        //   },
        //   "index": false
        // },
        {
          "name": "limitPrice",
          "type": "u64",
          "index": false
        },
        {
          "name": "autoStake",
          "type": "bool",
          "index": false
        },
        {
          "name": "postOnly",
          "type": "bool",
          "index": false
        },
        {
          "name": "postAllowed",
          "type": "bool",
          "index": false
        }
      ]
    },
    {
      "name": "ObligationRepay",
      "fields": [
        {
          "name": "orderbookUser",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "obligation",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "repaymentAmount",
          "type": "u64",
          "index": false
        },
        {
          "name": "finalBalance",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "ObligationFulfilled",
      "fields": [
        {
          "name": "obligation",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "orderbookUser",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "borrower",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "timestamp",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "OrderCancelled",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "authority",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "orderId",
          "type": "u128",
          "index": false
        }
      ]
    },
    {
      "name": "EventAdapterRegistered",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "owner",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "adapter",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "TokensExchanged",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "user",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "TicketRedeemed",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "ticketHolder",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "redeemedValue",
          "type": "u64",
          "index": false
        },
        {
          "name": "maturationTimestamp",
          "type": "i64",
          "index": false
        },
        {
          "name": "redeemedTimestamp",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "TicketsStaked",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "ticketHolder",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "TicketTransferred",
      "fields": [
        {
          "name": "ticket",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "previousOwner",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "newOwner",
          "type": "publicKey",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "ArithmeticOverflow",
      "msg": "overflow occured on checked_add"
    },
    {
      "code": 6001,
      "name": "ArithmeticUnderflow",
      "msg": "underflow occured on checked_sub"
    },
    {
      "code": 6002,
      "name": "FixedPointDivision",
      "msg": "bad fixed-point division"
    },
    {
      "code": 6003,
      "name": "DoesNotOwnTicket",
      "msg": "owner does not own the ticket"
    },
    {
      "code": 6004,
      "name": "DoesNotOwnEventAdapter",
      "msg": "signer does not own the event adapter"
    },
    {
      "code": 6005,
      "name": "EventQueueFull",
      "msg": "queue does not have room for another event"
    },
    {
      "code": 6006,
      "name": "FailedToDeserializeTicket",
      "msg": "failed to deserialize the SplitTicket or ClaimTicket"
    },
    {
      "code": 6007,
      "name": "ImmatureBond",
      "msg": "bond is not mature and cannot be claimed"
    },
    {
      "code": 6008,
      "name": "InsufficientSeeds",
      "msg": "not enough seeds were provided for the accounts that need to be initialized"
    },
    {
      "code": 6009,
      "name": "InvalidEvent",
      "msg": "the wrong event type was unwrapped\\nthis condition should be impossible, and does not result from invalid input"
    },
    {
      "code": 6010,
      "name": "InvalidOrderPrice",
      "msg": "order price is prohibited"
    },
    {
      "code": 6011,
      "name": "InvokeCreateAccount",
      "msg": "failed to invoke account creation"
    },
    {
      "code": 6012,
      "name": "IoError",
      "msg": "failed to properly serialize or deserialize a data structure"
    },
    {
      "code": 6013,
      "name": "MarketStateNotProgramOwned",
      "msg": "this market state account is not owned by the current program"
    },
    {
      "code": 6014,
      "name": "MissingEventAdapter",
      "msg": "tried to access a missing adapter account"
    },
    {
      "code": 6015,
      "name": "MissingSplitTicket",
      "msg": "tried to access a missing split ticket account"
    },
    {
      "code": 6016,
      "name": "NoEvents",
      "msg": "consume_events instruction failed to consume a single event"
    },
    {
      "code": 6017,
      "name": "NoMoreAccounts",
      "msg": "expected additional remaining accounts, but there were none"
    },
    {
      "code": 6018,
      "name": "ObligationHasWrongSequenceNumber",
      "msg": "expected an obligation with a different sequence number"
    },
    {
      "code": 6019,
      "name": "OracleError",
      "msg": "there was a problem loading the price oracle"
    },
    {
      "code": 6020,
      "name": "OrderNotFound",
      "msg": "id was not found in the user's open orders"
    },
    {
      "code": 6021,
      "name": "OrderbookPaused",
      "msg": "Orderbook is not taking orders"
    },
    {
      "code": 6022,
      "name": "OrderRejected",
      "msg": "aaob did not match or post the order. either posting is disabled or the order was too small"
    },
    {
      "code": 6023,
      "name": "PriceMissing",
      "msg": "price could not be accessed from oracle"
    },
    {
      "code": 6024,
      "name": "TicketNotFromManager",
      "msg": "claim ticket is not from this manager"
    },
    {
      "code": 6025,
      "name": "TicketsPaused",
      "msg": "tickets are paused"
    },
    {
      "code": 6026,
      "name": "UnauthorizedCaller",
      "msg": "this signer is not authorized to place a permissioned order"
    },
    {
      "code": 6027,
      "name": "UserDoesNotOwnAccount",
      "msg": "this user does not own the user account"
    },
    {
      "code": 6028,
      "name": "UserDoesNotOwnAdapter",
      "msg": "this adapter does not belong to the user"
    },
    {
      "code": 6029,
      "name": "UserNotInMarket",
      "msg": "this user account is not associated with this bond market"
    },
    {
      "code": 6030,
      "name": "WrongAdapter",
      "msg": "the wrong adapter account was passed to this instruction"
    },
    {
      "code": 6031,
      "name": "WrongAsks",
      "msg": "asks account does not belong to this market"
    },
    {
      "code": 6032,
      "name": "WrongAirspace",
      "msg": "the market is configured for a different airspace"
    },
    {
      "code": 6033,
      "name": "WrongAirspaceAuthorization",
      "msg": "the signer is not authorized to perform this action in the current airspace"
    },
    {
      "code": 6034,
      "name": "WrongBids",
      "msg": "bids account does not belong to this market"
    },
    {
      "code": 6035,
      "name": "WrongBondManager",
      "msg": "adapter does not belong to given bond manager"
    },
    {
      "code": 6036,
      "name": "WrongCrankAuthority",
      "msg": "wrong authority for this crank instruction"
    },
    {
      "code": 6037,
      "name": "WrongEventQueue",
      "msg": "event queue account does not belong to this market"
    },
    {
      "code": 6038,
      "name": "WrongMarketState",
      "msg": "this market state is not associated with this market"
    },
    {
      "code": 6039,
      "name": "WrongTicketManager",
      "msg": "wrong TicketManager account provided"
    },
    {
      "code": 6040,
      "name": "DoesNotOwnMarket",
      "msg": "this market owner does not own this market"
    },
    {
      "code": 6041,
      "name": "WrongClaimAccount",
      "msg": "the wrong account was provided for the token account that represents a user's claims"
    },
    {
      "code": 6042,
      "name": "WrongCollateralAccount",
      "msg": "the wrong account was provided for the token account that represents a user's collateral"
    },
    {
      "code": 6043,
      "name": "WrongClaimMint",
      "msg": "the wrong account was provided for the claims token mint"
    },
    {
      "code": 6044,
      "name": "WrongCollateralMint",
      "msg": "the wrong account was provided for the collateral token mint"
    },
    {
      "code": 6045,
      "name": "WrongOracle",
      "msg": "wrong oracle address was sent to instruction"
    },
    {
      "code": 6046,
      "name": "WrongMarginUser",
      "msg": "wrong margin user account address was sent to instruction"
    },
    {
      "code": 6047,
      "name": "WrongMarginUserAuthority",
      "msg": "wrong authority for the margin user account address was sent to instruction"
    },
    {
      "code": 6048,
      "name": "WrongProgramAuthority",
      "msg": "incorrect authority account"
    },
    {
      "code": 6049,
      "name": "WrongTicketMint",
      "msg": "not the ticket mint for this bond market"
    },
    {
      "code": 6050,
      "name": "WrongTicketSettlementAccount",
      "msg": "wrong ticket settlement account"
    },
    {
      "code": 6051,
      "name": "WrongUnderlyingSettlementAccount",
      "msg": "wrong underlying settlement account"
    },
    {
      "code": 6052,
      "name": "WrongUnderlyingTokenMint",
      "msg": "wrong underlying token mint for this bond market"
    },
    {
      "code": 6053,
      "name": "WrongUserAccount",
      "msg": "wrong user account address was sent to instruction"
    },
    {
      "code": 6054,
      "name": "WrongVault",
      "msg": "wrong vault address was sent to instruction"
    },
    {
      "code": 6055,
      "name": "ZeroDivision",
      "msg": "attempted to divide with zero"
    }
  ]
};

export const IDL: JetBonds = {
  "version": "0.1.0",
  "name": "jet_bonds",
  "constants": [
    {
      "name": "BOND_MANAGER",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"bond_manager\""
    },
    {
      "name": "BOND_TICKET_ACCOUNT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"bond_ticket_account\""
    },
    {
      "name": "BOND_TICKET_MINT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"bond_ticket_mint\""
    },
    {
      "name": "CLAIM_TICKET",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"claim_ticket\""
    },
    {
      "name": "CRANK_AUTHORIZATION",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"crank_authorization\""
    },
    {
      "name": "CLAIM_NOTES",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"claim_notes\""
    },
    {
      "name": "COLLATERAL_NOTES",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"collateral_notes\""
    },
    {
      "name": "SPLIT_TICKET",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"split_ticket\""
    },
    {
      "name": "EVENT_ADAPTER",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"event_adapter\""
    },
    {
      "name": "OBLIGATION",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"obligation\""
    },
    {
      "name": "ORDERBOOK_MARKET_STATE",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"orderbook_market_state\""
    },
    {
      "name": "MARGIN_BORROWER",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"margin_borrower\""
    },
    {
      "name": "UNDERLYING_TOKEN_VAULT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"underlying_token_vault\""
    }
  ],
  "instructions": [
    {
      "name": "authorizeCrank",
      "accounts": [
        {
          "name": "crank",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "crankAuthorization",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "revokeCrank",
      "accounts": [
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "receiver",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "initializeBondManager",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingOracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ticketOracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "InitializeBondManagerParams"
          }
        }
      ]
    },
    {
      "name": "initializeOrderbook",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "InitializeOrderbookParams"
          }
        }
      ]
    },
    {
      "name": "modifyBondManager",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "data",
          "type": "bytes"
        },
        {
          "name": "offset",
          "type": "u32"
        }
      ]
    },
    {
      "name": "pauseOrderMatching",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "resumeOrderMatching",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "initializeMarginUser",
      "accounts": [
        {
          "name": "borrowerAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "claimsMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingSettlement",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ticketSettlement",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "claimsMetadata",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "collateralMetadata",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "marginBorrowOrder",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "obligation",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "claimsMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "bondManager",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMarketState",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "eventQueue",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bids",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "asks",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
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
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        },
        {
          "name": "seed",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "marginSellTicketsOrder",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "authority",
              "isMut": false,
              "isSigner": true
            },
            {
              "name": "userTicketVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "userTokenVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMut",
              "accounts": [
                {
                  "name": "bondManager",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "orderbookMarketState",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "eventQueue",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "bids",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "asks",
                  "isMut": true,
                  "isSigner": false
                }
              ]
            },
            {
              "name": "bondTicketMint",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "tokenProgram",
              "isMut": false,
              "isSigner": false
            }
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        }
      ]
    },
    {
      "name": "marginRedeemTicket",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "ticket",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "authority",
              "isMut": true,
              "isSigner": true
            },
            {
              "name": "claimantTokenAccount",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bondManager",
              "isMut": false,
              "isSigner": false
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "tokenProgram",
              "isMut": false,
              "isSigner": false
            }
          ]
        }
      ],
      "args": []
    },
    {
      "name": "marginLendOrder",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "authority",
              "isMut": false,
              "isSigner": true
            },
            {
              "name": "orderbookMut",
              "accounts": [
                {
                  "name": "bondManager",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "orderbookMarketState",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "eventQueue",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "bids",
                  "isMut": true,
                  "isSigner": false
                },
                {
                  "name": "asks",
                  "isMut": true,
                  "isSigner": false
                }
              ]
            },
            {
              "name": "ticketSettlement",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "lenderTokens",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "ticketMint",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "payer",
              "isMut": true,
              "isSigner": true
            },
            {
              "name": "systemProgram",
              "isMut": false,
              "isSigner": false
            },
            {
              "name": "tokenProgram",
              "isMut": false,
              "isSigner": false
            }
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        },
        {
          "name": "seed",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "refreshPosition",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingOracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ticketOracle",
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
          "name": "expectPrice",
          "type": "bool"
        }
      ]
    },
    {
      "name": "repay",
      "accounts": [
        {
          "name": "borrowerAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "obligation",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "nextObligation",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "source",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
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
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "settle",
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "claimsMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingSettlement",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ticketSettlement",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "sellTicketsOrder",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "userTicketVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "bondManager",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMarketState",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "eventQueue",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bids",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "asks",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
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
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        }
      ]
    },
    {
      "name": "cancelOrder",
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "bondManager",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMarketState",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "eventQueue",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bids",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "asks",
              "isMut": true,
              "isSigner": false
            }
          ]
        }
      ],
      "args": [
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "lendOrder",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "bondManager",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "orderbookMarketState",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "eventQueue",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "bids",
              "isMut": true,
              "isSigner": false
            },
            {
              "name": "asks",
              "isMut": true,
              "isSigner": false
            }
          ]
        },
        {
          "name": "ticketSettlement",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "lenderTokens",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ticketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
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
          "name": "params",
          "type": {
            "defined": "OrderParams"
          }
        },
        {
          "name": "seed",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "consumeEvents",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "crankAuthorization",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "crank",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
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
          "name": "numEvents",
          "type": "u32"
        },
        {
          "name": "seedBytes",
          "type": {
            "vec": "bytes"
          }
        }
      ]
    },
    {
      "name": "exchangeTokens",
      "accounts": [
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userBondTicketVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userUnderlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "userAuthority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "redeemTicket",
      "accounts": [
        {
          "name": "ticket",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "claimantTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "stakeBondTickets",
      "accounts": [
        {
          "name": "claimTicket",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ticketHolder",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "bondTicketTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "StakeBondTicketsParams"
          }
        }
      ]
    },
    {
      "name": "tranferTicketOwnership",
      "accounts": [
        {
          "name": "ticket",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "currentOwner",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "newOwner",
          "type": "publicKey"
        }
      ]
    },
    {
      "name": "registerAdapter",
      "accounts": [
        {
          "name": "adapterQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondManager",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "RegisterAdapterParams"
          }
        }
      ]
    },
    {
      "name": "popAdapterEvents",
      "accounts": [
        {
          "name": "adapterQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": [
        {
          "name": "numEvents",
          "type": "u32"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "BondManager",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "versionTag",
            "type": "u64"
          },
          {
            "name": "airspace",
            "type": "publicKey"
          },
          {
            "name": "orderbookMarketState",
            "type": "publicKey"
          },
          {
            "name": "eventQueue",
            "type": "publicKey"
          },
          {
            "name": "asks",
            "type": "publicKey"
          },
          {
            "name": "bids",
            "type": "publicKey"
          },
          {
            "name": "underlyingTokenMint",
            "type": "publicKey"
          },
          {
            "name": "underlyingTokenVault",
            "type": "publicKey"
          },
          {
            "name": "bondTicketMint",
            "type": "publicKey"
          },
          {
            "name": "claimsMint",
            "type": "publicKey"
          },
          {
            "name": "collateralMint",
            "type": "publicKey"
          },
          {
            "name": "underlyingOracle",
            "type": "publicKey"
          },
          {
            "name": "ticketOracle",
            "type": "publicKey"
          },
          {
            "name": "seed",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "bump",
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "orderbookPaused",
            "type": "bool"
          },
          {
            "name": "ticketsPaused",
            "type": "bool"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                28
              ]
            }
          },
          {
            "name": "borrowDuration",
            "type": "i64"
          },
          {
            "name": "lendDuration",
            "type": "i64"
          },
          {
            "name": "nonce",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "CrankAuthorization",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "crank",
            "type": "publicKey"
          },
          {
            "name": "airspace",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "MarginUser",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "type": "u8"
          },
          {
            "name": "marginAccount",
            "type": "publicKey"
          },
          {
            "name": "bondManager",
            "type": "publicKey"
          },
          {
            "name": "claims",
            "type": "publicKey"
          },
          {
            "name": "collateral",
            "type": "publicKey"
          },
          {
            "name": "underlyingSettlement",
            "type": "publicKey"
          },
          {
            "name": "ticketSettlement",
            "type": "publicKey"
          },
          {
            "name": "debt",
            "type": {
              "defined": "Debt"
            }
          },
          {
            "name": "assets",
            "type": {
              "defined": "Assets"
            }
          }
        ]
      }
    },
    {
      "name": "Obligation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "sequenceNumber",
            "type": "u64"
          },
          {
            "name": "borrowerAccount",
            "type": "publicKey"
          },
          {
            "name": "bondManager",
            "type": "publicKey"
          },
          {
            "name": "orderTag",
            "type": {
              "array": ["u8", 16]
            }
          },
          {
            "name": "maturationTimestamp",
            "type": "u64"
          },
          {
            "name": "balance",
            "type": "u64"
          },
          {
            "name": "flags",
            "docs": [
              "Any boolean flags for this data type compressed to a single byte"
            ],
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "EventAdapterMetadata",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "manager",
            "type": "publicKey"
          },
          {
            "name": "orderbookUser",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "ClaimTicket",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "bondManager",
            "type": "publicKey"
          },
          {
            "name": "maturationTimestamp",
            "type": "i64"
          },
          {
            "name": "redeemable",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "SplitTicket",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "bondManager",
            "type": "publicKey"
          },
          {
            "name": "orderTag",
            "type": {
              "array": ["u8", 16]
            }
          },
          {
            "name": "struckTimestamp",
            "type": "i64"
          },
          {
            "name": "maturationTimestamp",
            "type": "i64"
          },
          {
            "name": "principal",
            "type": "u64"
          },
          {
            "name": "interest",
            "type": "u64"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "InitializeBondManagerParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "versionTag",
            "type": "u64"
          },
          {
            "name": "seed",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "borrowDuration",
            "type": "i64"
          },
          {
            "name": "lendDuration",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "InitializeOrderbookParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "minBaseOrderSize",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "Debt",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "nextNewObligationSeqno",
            "type": "u64"
          },
          {
            "name": "nextUnpaidObligationSeqno",
            "type": "u64"
          },
          {
            "name": "nextObligationMaturity",
            "type": "u64"
          },
          {
            "name": "pending",
            "type": "u64"
          },
          {
            "name": "committed",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "Assets",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "entitledTokens",
            "type": "u64"
          },
          {
            "name": "entitledTickets",
            "type": "u64"
          },
          {
            "name": "ticketsStaked",
            "type": "u64"
          },
          {
            "name": "postedQuote",
            "type": "u64"
          },
          {
            "name": "reserved0",
            "type": {
              "array": [
                "u8",
                64
              ]
            }
          }
        ]
      }
    },
    {
      "name": "RegisterAdapterParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "numEvents",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "OrderParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxBondTicketQty",
            "type": "u64"
          },
          {
            "name": "maxUnderlyingTokenQty",
            "type": "u64"
          },
          {
            "name": "limitPrice",
            "type": "u64"
          },
          {
            "name": "matchLimit",
            "type": "u64"
          },
          {
            "name": "postOnly",
            "type": "bool"
          },
          {
            "name": "postAllowed",
            "type": "bool"
          },
          {
            "name": "autoStake",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "StakeBondTicketsParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "ticketSeed",
            "type": "bytes"
          }
        ]
      }
    },
    {
      "name": "OrderType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "MarginBorrow"
          },
          {
            "name": "MarginLend"
          },
          {
            "name": "MarginSellTickets"
          },
          {
            "name": "Lend"
          },
          {
            "name": "SellTickets"
          }
        ]
      }
    },
    {
      "name": "EventAccounts",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill",
            "fields": [
              {
                "defined": "Box<FillAccounts<'info>>"
              }
            ]
          },
          {
            "name": "Out",
            "fields": [
              {
                "defined": "Box<OutAccounts<'info>>"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "LoanAccount",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "AutoStake",
            "fields": [
              {
                "defined": "AnchorAccount<'info,SplitTicket,Mut>"
              }
            ]
          },
          {
            "name": "NewDebt",
            "fields": [
              {
                "defined": "AnchorAccount<'info,Obligation,Mut>"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "EventTag",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill"
          },
          {
            "name": "Out"
          }
        ]
      }
    },
    {
      "name": "OrderbookEvent",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill",
            "fields": [
              {
                "defined": "FillInfo"
              }
            ]
          },
          {
            "name": "Out",
            "fields": [
              {
                "defined": "OutInfo"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "TicketKind",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Claim",
            "fields": [
              {
                "defined": "Account<'info,ClaimTicket>"
              }
            ]
          },
          {
            "name": "Split",
            "fields": [
              {
                "defined": "Account<'info,SplitTicket>"
              }
            ]
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "BondManagerInitialized",
      "fields": [
        {
          "name": "version",
          "type": "u64",
          "index": false
        },
        {
          "name": "address",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "airspace",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "underlyingTokenMint",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "underlyingOracle",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "ticketOracle",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "borrowDuration",
          "type": "i64",
          "index": false
        },
        {
          "name": "lendDuration",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "OrderbookInitialized",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "orderbookMarketState",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "eventQueue",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "bids",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "asks",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "minBaseOrderSize",
          "type": "u64",
          "index": false
        },
        {
          "name": "tickSize",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "PositionRefreshed",
      "fields": [
        {
          "name": "borrowerAccount",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "ToggleOrderMatching",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "isOrderbookPaused",
          "type": "bool",
          "index": false
        }
      ]
    },
    {
      "name": "SkippedError",
      "fields": [
        {
          "name": "message",
          "type": "string",
          "index": false
        }
      ]
    },
    {
      "name": "MarginUserInitialized",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "borrowerAccount",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "marginAccount",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "underlyingSettlement",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "ticketSettlement",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "OrderPlaced",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "authority",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "marginUser",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "orderType",
          "type": {
            "defined": "OrderType"
          },
          "index": false
        },
        // {
        //   "name": "orderSummary",
        //   "type": {
        //     "defined": "OrderSummary"
        //   },
        //   "index": false
        // },
        {
          "name": "limitPrice",
          "type": "u64",
          "index": false
        },
        {
          "name": "autoStake",
          "type": "bool",
          "index": false
        },
        {
          "name": "postOnly",
          "type": "bool",
          "index": false
        },
        {
          "name": "postAllowed",
          "type": "bool",
          "index": false
        }
      ]
    },
    {
      "name": "ObligationRepay",
      "fields": [
        {
          "name": "orderbookUser",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "obligation",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "repaymentAmount",
          "type": "u64",
          "index": false
        },
        {
          "name": "finalBalance",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "ObligationFulfilled",
      "fields": [
        {
          "name": "obligation",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "orderbookUser",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "borrower",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "timestamp",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "OrderCancelled",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "authority",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "orderId",
          "type": "u128",
          "index": false
        }
      ]
    },
    {
      "name": "EventAdapterRegistered",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "owner",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "adapter",
          "type": "publicKey",
          "index": false
        }
      ]
    },
    {
      "name": "TokensExchanged",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "user",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "TicketRedeemed",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "ticketHolder",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "redeemedValue",
          "type": "u64",
          "index": false
        },
        {
          "name": "maturationTimestamp",
          "type": "i64",
          "index": false
        },
        {
          "name": "redeemedTimestamp",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "TicketsStaked",
      "fields": [
        {
          "name": "bondManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "ticketHolder",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "TicketTransferred",
      "fields": [
        {
          "name": "ticket",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "previousOwner",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "newOwner",
          "type": "publicKey",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "ArithmeticOverflow",
      "msg": "overflow occured on checked_add"
    },
    {
      "code": 6001,
      "name": "ArithmeticUnderflow",
      "msg": "underflow occured on checked_sub"
    },
    {
      "code": 6002,
      "name": "FixedPointDivision",
      "msg": "bad fixed-point division"
    },
    {
      "code": 6003,
      "name": "DoesNotOwnTicket",
      "msg": "owner does not own the ticket"
    },
    {
      "code": 6004,
      "name": "DoesNotOwnEventAdapter",
      "msg": "signer does not own the event adapter"
    },
    {
      "code": 6005,
      "name": "EventQueueFull",
      "msg": "queue does not have room for another event"
    },
    {
      "code": 6006,
      "name": "FailedToDeserializeTicket",
      "msg": "failed to deserialize the SplitTicket or ClaimTicket"
    },
    {
      "code": 6007,
      "name": "ImmatureBond",
      "msg": "bond is not mature and cannot be claimed"
    },
    {
      "code": 6008,
      "name": "InsufficientSeeds",
      "msg": "not enough seeds were provided for the accounts that need to be initialized"
    },
    {
      "code": 6009,
      "name": "InvalidEvent",
      "msg": "the wrong event type was unwrapped\\nthis condition should be impossible, and does not result from invalid input"
    },
    {
      "code": 6010,
      "name": "InvalidOrderPrice",
      "msg": "order price is prohibited"
    },
    {
      "code": 6011,
      "name": "InvokeCreateAccount",
      "msg": "failed to invoke account creation"
    },
    {
      "code": 6012,
      "name": "IoError",
      "msg": "failed to properly serialize or deserialize a data structure"
    },
    {
      "code": 6013,
      "name": "MarketStateNotProgramOwned",
      "msg": "this market state account is not owned by the current program"
    },
    {
      "code": 6014,
      "name": "MissingEventAdapter",
      "msg": "tried to access a missing adapter account"
    },
    {
      "code": 6015,
      "name": "MissingSplitTicket",
      "msg": "tried to access a missing split ticket account"
    },
    {
      "code": 6016,
      "name": "NoEvents",
      "msg": "consume_events instruction failed to consume a single event"
    },
    {
      "code": 6017,
      "name": "NoMoreAccounts",
      "msg": "expected additional remaining accounts, but there were none"
    },
    {
      "code": 6018,
      "name": "ObligationHasWrongSequenceNumber",
      "msg": "expected an obligation with a different sequence number"
    },
    {
      "code": 6019,
      "name": "OracleError",
      "msg": "there was a problem loading the price oracle"
    },
    {
      "code": 6020,
      "name": "OrderNotFound",
      "msg": "id was not found in the user's open orders"
    },
    {
      "code": 6021,
      "name": "OrderbookPaused",
      "msg": "Orderbook is not taking orders"
    },
    {
      "code": 6022,
      "name": "OrderRejected",
      "msg": "aaob did not match or post the order. either posting is disabled or the order was too small"
    },
    {
      "code": 6023,
      "name": "PriceMissing",
      "msg": "price could not be accessed from oracle"
    },
    {
      "code": 6024,
      "name": "TicketNotFromManager",
      "msg": "claim ticket is not from this manager"
    },
    {
      "code": 6025,
      "name": "TicketsPaused",
      "msg": "tickets are paused"
    },
    {
      "code": 6026,
      "name": "UnauthorizedCaller",
      "msg": "this signer is not authorized to place a permissioned order"
    },
    {
      "code": 6027,
      "name": "UserDoesNotOwnAccount",
      "msg": "this user does not own the user account"
    },
    {
      "code": 6028,
      "name": "UserDoesNotOwnAdapter",
      "msg": "this adapter does not belong to the user"
    },
    {
      "code": 6029,
      "name": "UserNotInMarket",
      "msg": "this user account is not associated with this bond market"
    },
    {
      "code": 6030,
      "name": "WrongAdapter",
      "msg": "the wrong adapter account was passed to this instruction"
    },
    {
      "code": 6031,
      "name": "WrongAsks",
      "msg": "asks account does not belong to this market"
    },
    {
      "code": 6032,
      "name": "WrongAirspace",
      "msg": "the market is configured for a different airspace"
    },
    {
      "code": 6033,
      "name": "WrongAirspaceAuthorization",
      "msg": "the signer is not authorized to perform this action in the current airspace"
    },
    {
      "code": 6034,
      "name": "WrongBids",
      "msg": "bids account does not belong to this market"
    },
    {
      "code": 6035,
      "name": "WrongBondManager",
      "msg": "adapter does not belong to given bond manager"
    },
    {
      "code": 6036,
      "name": "WrongCrankAuthority",
      "msg": "wrong authority for this crank instruction"
    },
    {
      "code": 6037,
      "name": "WrongEventQueue",
      "msg": "event queue account does not belong to this market"
    },
    {
      "code": 6038,
      "name": "WrongMarketState",
      "msg": "this market state is not associated with this market"
    },
    {
      "code": 6039,
      "name": "WrongTicketManager",
      "msg": "wrong TicketManager account provided"
    },
    {
      "code": 6040,
      "name": "DoesNotOwnMarket",
      "msg": "this market owner does not own this market"
    },
    {
      "code": 6041,
      "name": "WrongClaimAccount",
      "msg": "the wrong account was provided for the token account that represents a user's claims"
    },
    {
      "code": 6042,
      "name": "WrongCollateralAccount",
      "msg": "the wrong account was provided for the token account that represents a user's collateral"
    },
    {
      "code": 6043,
      "name": "WrongClaimMint",
      "msg": "the wrong account was provided for the claims token mint"
    },
    {
      "code": 6044,
      "name": "WrongCollateralMint",
      "msg": "the wrong account was provided for the collateral token mint"
    },
    {
      "code": 6045,
      "name": "WrongOracle",
      "msg": "wrong oracle address was sent to instruction"
    },
    {
      "code": 6046,
      "name": "WrongMarginUser",
      "msg": "wrong margin user account address was sent to instruction"
    },
    {
      "code": 6047,
      "name": "WrongMarginUserAuthority",
      "msg": "wrong authority for the margin user account address was sent to instruction"
    },
    {
      "code": 6048,
      "name": "WrongProgramAuthority",
      "msg": "incorrect authority account"
    },
    {
      "code": 6049,
      "name": "WrongTicketMint",
      "msg": "not the ticket mint for this bond market"
    },
    {
      "code": 6050,
      "name": "WrongTicketSettlementAccount",
      "msg": "wrong ticket settlement account"
    },
    {
      "code": 6051,
      "name": "WrongUnderlyingSettlementAccount",
      "msg": "wrong underlying settlement account"
    },
    {
      "code": 6052,
      "name": "WrongUnderlyingTokenMint",
      "msg": "wrong underlying token mint for this bond market"
    },
    {
      "code": 6053,
      "name": "WrongUserAccount",
      "msg": "wrong user account address was sent to instruction"
    },
    {
      "code": 6054,
      "name": "WrongVault",
      "msg": "wrong vault address was sent to instruction"
    },
    {
      "code": 6055,
      "name": "ZeroDivision",
      "msg": "attempted to divide with zero"
    }
  ]
};
