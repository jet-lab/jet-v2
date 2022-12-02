export type JetMarket = {
  "version": "0.1.0",
  "name": "jet_market",
  "constants": [
    {
      "name": "MARKET_MANAGER",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"market_manager\""
    },
    {
      "name": "MARKET_TICKET_ACCOUNT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"market_ticket_account\""
    },
    {
      "name": "MARKET_TICKET_MINT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"market_ticket_mint\""
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
      "docs": [
        "authorize an address to run orderbook consume_event instructions"
      ],
      "accounts": [
        {
          "name": "crank",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The crank signer pubkey"
          ]
        },
        {
          "name": "crankAuthorization",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account containing the metadata for the key"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The address paying the rent for the account"
          ]
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
      "docs": [
        "unauthorize an address to run orderbook consume_event instructions"
      ],
      "accounts": [
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account containing the metadata for the key"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
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
      "name": "initializeMarketManager",
      "docs": [
        "Initializes a MarketManager for a fixed market"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The `MarketManager` manages asset tokens for a particular market tenor"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault for storing the token underlying the market tickets"
          ]
        },
        {
          "name": "underlyingTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint for the assets underlying the market tickets"
          ]
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The minting account for the market tickets"
          ]
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Mints tokens to a margin account to represent debt that must be collateralized"
          ]
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Mints tokens to a margin account to represent debt that must be collateralized"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        },
        {
          "name": "underlyingOracle",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The oracle for the underlying asset price"
          ]
        },
        {
          "name": "ticketOracle",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The oracle for the market ticket price"
          ]
        },
        {
          "name": "feeDestination",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The account where fees are allowed to be withdrawn"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The account paying rent for PDA initialization"
          ]
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Rent sysvar"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Solana system program"
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "InitializeMarketManagerParams"
          }
        }
      ]
    },
    {
      "name": "initializeOrderbook",
      "docs": [
        "Initializes a new orderbook"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The `MarketManager` account tracks global information related to this particular fixed market"
          ]
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AOB market state"
          ]
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AOB market event queue",
            "",
            "Must be initialized"
          ]
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AOB market bids"
          ]
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AOB market asks"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The account paying rent for PDA initialization"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Solana system program"
          ]
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
      "name": "modifyMarketManager",
      "docs": [
        "Modify a `MarketManager` account",
        "Authority use only"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The `MarketManager` manages asset tokens for a particular market tenor"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        }
      ],
      "args": [
        {
          "name": "data",
          "type": "bytes"
        },
        {
          "name": "offset",
          "type": "u64" // should be "u64"
        }
      ]
    },
    {
      "name": "pauseOrderMatching",
      "docs": [
        "Pause matching of orders placed in the orderbook"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The `MarketManager` manages asset tokens for a particular market tenor"
          ]
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "resumeOrderMatching",
      "docs": [
        "Resume matching of orders placed in the orderbook",
        "NOTE: This instruction may have to be run several times to clear the",
        "existing matches. Check the `orderbook_market_state.pause_matching` variable",
        "to determine success"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The `MarketManager` manages asset tokens for a particular market tenor"
          ]
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
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "initializeMarginUser",
      "docs": [
        "Create a new borrower account"
      ],
      "accounts": [
        {
          "name": "borrowerAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking information related to this particular user"
          ]
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The signing authority for this user account"
          ]
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The Boheader account"
          ]
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt",
            "that must be collateralized"
          ]
        },
        {
          "name": "claimsMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track owned assets"
          ]
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
          "isSigner": false,
          "docs": [
            "Token metadata account needed by the margin program to register the claim position"
          ]
        },
        {
          "name": "collateralMetadata",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token metadata account needed by the margin program to register the collateral position"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "marginBorrowOrder",
      "docs": [
        "Place a borrow order by leveraging margin account value"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking borrower debts"
          ]
        },
        {
          "name": "obligation",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Obligation account minted upon match"
          ]
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The margin account for this borrow order"
          ]
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "claimsMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
          ]
        },
        {
          "name": "underlyingSettlement",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
          ]
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "marketManager",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The `MarketManager` account tracks global information related to this particular fixed market"
              ]
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
          "isSigner": true,
          "docs": [
            "payer for `Obligation` initialization"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Solana system program"
          ]
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
      "docs": [
        "Sell tickets that are already owned"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking borrower debts"
          ]
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "authority",
              "isMut": false,
              "isSigner": true,
              "docs": [
                "Signing authority over the ticket vault transferring for a borrow order"
              ]
            },
            {
              "name": "userTicketVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "Account containing the market tickets being sold"
              ]
            },
            {
              "name": "userTokenVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The account to receive the matched tokens"
              ]
            },
            {
              "name": "orderbookMut",
              "accounts": [
                {
                  "name": "marketManager",
                  "isMut": true,
                  "isSigner": false,
                  "docs": [
                    "The `MarketManager` account tracks global information related to this particular fixed market"
                  ]
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
              "name": "marketTicketMint",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The market ticket mint"
              ]
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The market ticket mint"
              ]
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
      "docs": [
        "Redeem a staked ticket"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the collateral value of assets custodied by Jet markets"
          ]
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the collateral value of assets custodied by Jet markets"
          ]
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "ticket",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "One of either `SplitTicket` or `ClaimTicket` for redemption"
              ]
            },
            {
              "name": "authority",
              "isMut": true,
              "isSigner": true,
              "docs": [
                "The account that must sign to redeem the ticket"
              ]
            },
            {
              "name": "claimantTokenAccount",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The token account designated to receive the assets underlying the claim"
              ]
            },
            {
              "name": "marketManager",
              "isMut": false,
              "isSigner": false,
              "docs": [
                "The MarketManager responsible for the asset"
              ]
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The vault stores the tokens of the underlying asset managed by the MarketManager"
              ]
            },
            {
              "name": "tokenProgram",
              "isMut": false,
              "isSigner": false,
              "docs": [
                "SPL token program"
              ]
            }
          ]
        }
      ],
      "args": []
    },
    {
      "name": "marginLendOrder",
      "docs": [
        "Place a `Lend` order to the book by depositing tokens"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking borrower debts"
          ]
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "authority",
              "isMut": false,
              "isSigner": true,
              "docs": [
                "Signing authority over the token vault transferring for a lend order"
              ]
            },
            {
              "name": "orderbookMut",
              "accounts": [
                {
                  "name": "marketManager",
                  "isMut": true,
                  "isSigner": false,
                  "docs": [
                    "The `MarketManager` account tracks global information related to this particular fixed market"
                  ]
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
              "isSigner": false,
              "docs": [
                "where to settle tickets on match:",
                "- SplitTicket that will be created if the order is filled as a taker and `auto_stake` is enabled",
                "- ticket token account to receive market tickets",
                "be careful to check this properly. one way is by using lender_tickets_token_account"
              ]
            },
            {
              "name": "lenderTokens",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "where to loan tokens from"
              ]
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The market token vault"
              ]
            },
            {
              "name": "ticketMint",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The market token vault"
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
      "docs": [
        "Refresh the associated margin account `claims` for a given `MarginUser` account"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The account tracking information related to this particular user"
          ]
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The `MarketManager` account tracks global information related to this particular fixed market"
          ]
        },
        {
          "name": "underlyingOracle",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The pyth price account"
          ]
        },
        {
          "name": "ticketOracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
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
      "docs": [
        "Repay debt on an Obligation"
      ],
      "accounts": [
        {
          "name": "borrowerAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking information related to this particular user"
          ]
        },
        {
          "name": "obligation",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "nextObligation",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "No payment will be made towards next_obligation: it is needed purely for bookkeeping.",
            "if the user has additional obligations, this must be the one with the following sequence number.",
            "otherwise, put whatever address you want in here"
          ]
        },
        {
          "name": "source",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The token account to deposit tokens from"
          ]
        },
        {
          "name": "payer",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The signing authority for the source_account"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The token vault holding the underlying token of the market ticket"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
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
      "docs": [
        "Settle payments to a margin account"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking information related to this particular user"
          ]
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The `MarketManager` account tracks global information related to this particular fixed market"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "claimsMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
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
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingSettlement",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ticketSettlement",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "sellTicketsOrder",
      "docs": [
        "Place an order to the book to sell tickets, which will burn them"
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Signing authority over the ticket vault transferring for a borrow order"
          ]
        },
        {
          "name": "userTicketVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Account containing the market tickets being sold"
          ]
        },
        {
          "name": "userTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account to receive the matched tokens"
          ]
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "marketManager",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The `MarketManager` account tracks global information related to this particular fixed market"
              ]
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
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market ticket mint"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market ticket mint"
          ]
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
      "docs": [
        "Cancels an order on the book"
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The owner of the order"
          ]
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "marketManager",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The `MarketManager` account tracks global information related to this particular fixed market"
              ]
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
      "docs": [
        "Place a `Lend` order to the book by depositing tokens"
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Signing authority over the token vault transferring for a lend order"
          ]
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "marketManager",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The `MarketManager` account tracks global information related to this particular fixed market"
              ]
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
          "isSigner": false,
          "docs": [
            "where to settle tickets on match:",
            "- SplitTicket that will be created if the order is filled as a taker and `auto_stake` is enabled",
            "- ticket token account to receive market tickets",
            "be careful to check this properly. one way is by using lender_tickets_token_account"
          ]
        },
        {
          "name": "lenderTokens",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "where to loan tokens from"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
          ]
        },
        {
          "name": "ticketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
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
      "name": "consumeEvents",
      "docs": [
        "Crank specific instruction, processes the event queue"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The `MarketManager` account tracks global information related to this particular fixed market"
          ]
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market ticket mint"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
          ]
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
          "isSigner": true,
          "docs": [
            "The account paying rent for PDA initialization"
          ]
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
      "docs": [
        "Exchange underlying token for fixed market tickets",
        "WARNING: tickets must be staked for redeption of underlying"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The MarketManager manages asset tokens for a particular market tenor"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault stores the tokens of the underlying asset managed by the MarketManager"
          ]
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The minting account for the market tickets"
          ]
        },
        {
          "name": "userMarketTicketVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The token account to receive the exchanged market tickets"
          ]
        },
        {
          "name": "userUnderlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user controlled token account to exchange for market tickets"
          ]
        },
        {
          "name": "userAuthority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The signing authority in charge of the user's underlying token vault"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
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
      "docs": [
        "Redeems staked tickets for their underlying value"
      ],
      "accounts": [
        {
          "name": "ticket",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "One of either `SplitTicket` or `ClaimTicket` for redemption"
          ]
        },
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The account that must sign to redeem the ticket"
          ]
        },
        {
          "name": "claimantTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The token account designated to receive the assets underlying the claim"
          ]
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The MarketManager responsible for the asset"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault stores the tokens of the underlying asset managed by the MarketManager"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "stakeMarketTickets",
      "docs": [
        "Stakes market tickets for later redemption"
      ],
      "accounts": [
        {
          "name": "claimTicket",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "A struct used to track maturation and total claimable funds"
          ]
        },
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The MarketManager account tracks fixed market assets of a particular tenor"
          ]
        },
        {
          "name": "ticketHolder",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The owner of market tickets that wishes to stake them for a redeemable ticket"
          ]
        },
        {
          "name": "marketTicketTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking the ticket_holder's market tickets"
          ]
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The mint for the market tickets for this instruction",
            "A mint is a specific instance of the token program for both the underlying asset and the market tenor"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The payer for account initialization"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The global on-chain `TokenProgram` for account authority transfer."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The global on-chain `SystemProgram` for program account initialization."
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "StakeMarketTicketsParams"
          }
        }
      ]
    },
    {
      "name": "tranferTicketOwnership",
      "docs": [
        "Transfer staked tickets to a new owner"
      ],
      "accounts": [
        {
          "name": "ticket",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The ticket to transfer, either a ClaimTicket or SplitTicket"
          ]
        },
        {
          "name": "currentOwner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The current owner of the ticket"
          ]
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
      "docs": [
        "Register a new EventAdapter for syncing to the orderbook events"
      ],
      "accounts": [
        {
          "name": "adapterQueue",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AdapterEventQueue account owned by outside user or program"
          ]
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "MarketManager for this Adapter"
          ]
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Signing authority over this queue"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Payer for the initialization rent of the queue"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "solana system program"
          ]
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
      "docs": [
        "Pop the given number of events off the adapter queue",
        "Event logic is left to the outside program"
      ],
      "accounts": [
        {
          "name": "adapterQueue",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AdapterEventQueue account owned by outside user or program"
          ]
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Signing authority over the AdapterEventQueue"
          ]
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
      "name": "MarketManager", // should be capitalized
      "docs": [
        "The `MarketManager` contains all the information necessary to run the fixed market",
        "",
        "Utilized by program instructions to verify given transaction accounts are correct. Contains data",
        "about the fixed market including the tenor and ticket<->token conversion rate"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "versionTag",
            "docs": [
              "Versioning and tag information"
            ],
            "type": "u64"
          },
          {
            "name": "airspace",
            "docs": [
              "The airspace the market is a part of"
            ],
            "type": "publicKey"
          },
          {
            "name": "orderbookMarketState",
            "docs": [
              "The market state of the agnostic orderbook"
            ],
            "type": "publicKey"
          },
          {
            "name": "eventQueue",
            "docs": [
              "The orderbook event queue"
            ],
            "type": "publicKey"
          },
          {
            "name": "asks",
            "docs": [
              "The orderbook asks byteslab"
            ],
            "type": "publicKey"
          },
          {
            "name": "bids",
            "docs": [
              "The orderbook bids byteslab"
            ],
            "type": "publicKey"
          },
          {
            "name": "underlyingTokenMint",
            "docs": [
              "The token mint for the underlying asset of the market tickets"
            ],
            "type": "publicKey"
          },
          {
            "name": "underlyingTokenVault",
            "docs": [
              "Token account storing the underlying asset accounted for by this ticket program"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketTicketMint",
            "docs": [
              "The token mint for the market tickets"
            ],
            "type": "publicKey"
          },
          {
            "name": "claimsMint",
            "docs": [
              "Mint owned by Jet markets to issue claims against a user.",
              "These claim notes are monitored by margin to ensure claims are repaid."
            ],
            "type": "publicKey"
          },
          {
            "name": "collateralMint",
            "docs": [
              "Mint owned by Jet markets to issue collateral value to a user",
              "The collateral notes are monitored by the margin program to track value"
            ],
            "type": "publicKey"
          },
          {
            "name": "underlyingOracle",
            "docs": [
              "oracle that defines the value of the underlying asset"
            ],
            "type": "publicKey"
          },
          {
            "name": "ticketOracle",
            "docs": [
              "oracle that defines the value of the market tickets"
            ],
            "type": "publicKey"
          },
          {
            "name": "feeDestination",
            "docs": [
              "where fees can be withdrawn to"
            ],
            "type": "publicKey"
          },
          {
            "name": "seed",
            "docs": [
              "The user-defined part of the seed that generated this market manager's PDA"
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "bump",
            "docs": [
              "The bump seed value for generating the authority address."
            ],
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "orderbookPaused",
            "docs": [
              "Is the market taking orders"
            ],
            "type": "bool"
          },
          {
            "name": "ticketsPaused",
            "docs": [
              "Can tickets be redeemed"
            ],
            "type": "bool"
          },
          {
            "name": "reserved",
            "docs": [
              "reserved for future use"
            ],
            "type": {
              "array": [
                "u8",
                28
              ]
            }
          },
          {
            "name": "borrowTenor",
            "docs": [
              "Length of time before a borrow is marked as due, in seconds"
            ],
            "type": "i64"
          },
          {
            "name": "lendTenor",
            "docs": [
              "Length of time before a claim is marked as mature, in seconds"
            ],
            "type": "i64"
          },
          {
            "name": "originationFee",
            "docs": [
              "assessed on borrows. scaled by origination_fee::FEE_UNIT"
            ],
            "type": "u64"
          },
          {
            "name": "collectedFees",
            "docs": [
              "amount of fees currently available to be withdrawn by market owner"
            ],
            "type": "u64"
          },
          {
            "name": "nonce",
            "docs": [
              "Used to generate unique order tags"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "CrankAuthorization", // should be capitalized
      "docs": [
        "This authorizes a crank to act on any orderbook within the airspace"
      ],
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
      "name": "MarginUser", // should be capitalized
      "docs": [
        "An acocunt used to track margin users of the market"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "docs": [
              "used to determine if a migration step is needed before user actions are allowed"
            ],
            "type": "u8"
          },
          {
            "name": "marginAccount",
            "docs": [
              "The margin account used for signing actions"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketManager",
            "docs": [
              "The `MarketManager` for the market"
            ],
            "type": "publicKey"
          },
          {
            "name": "claims",
            "docs": [
              "Token account used by the margin program to track the debt"
            ],
            "type": "publicKey"
          },
          {
            "name": "collateral",
            "docs": [
              "Token account used by the margin program to track the collateral value of positions",
              "which are internal to Jet markets, such as SplitTicket, ClaimTicket, and open orders.",
              "this does *not* represent underlying tokens or market ticket tokens, those are registered independently in margin"
            ],
            "type": "publicKey"
          },
          {
            "name": "underlyingSettlement",
            "docs": [
              "The `settle` instruction is permissionless, therefore the user must specify upon margin account creation",
              "the address to send owed tokens"
            ],
            "type": "publicKey"
          },
          {
            "name": "ticketSettlement",
            "docs": [
              "The `settle` instruction is permissionless, therefore the user must specify upon margin account creation",
              "the address to send owed tickets"
            ],
            "type": "publicKey"
          },
          {
            "name": "debt",
            "docs": [
              "The amount of debt that must be collateralized or repaid",
              "This debt is expressed in terms of the underlying token - not market tickets"
            ],
            "type": {
              "defined": "Debt"
            }
          },
          {
            "name": "assets",
            "docs": [
              "Accounting used to track assets in custody of the fixed market"
            ],
            "type": {
              "defined": "Assets"
            }
          }
        ]
      }
    },
    {
      "name": "Obligation", // should be capitalized
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "sequenceNumber",
            "type": "u64" // should be "u64"
          },
          {
            "name": "borrowerAccount",
            "docs": [
              "The user borrower account this obligation is assigned to"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketManager",
            "docs": [
              "The market manager where the obligation was created"
            ],
            "type": "publicKey"
          },
          {
            "name": "orderTag",
            "docs": [
              "The `OrderTag` associated with the creation of this `Obligation`"
            ],
            "type": {
              "array": ["u8", 16] // should be ["u8", 16] 
            }
          },
          {
            "name": "maturationTimestamp",
            "docs": [
              "The time that the obligation must be repaid"
            ],
            "type": "u64" // should be "u64"
          },
          {
            "name": "balance",
            "docs": [
              "The remaining amount due by the end of the loan term"
            ],
            "type": "u64"
          },
          {
            "name": "flags",
            "docs": [
              "Any boolean flags for this data type compressed to a single byte"
            ],
            "type": "u8" // should be "u8"
          }
        ]
      }
    },
    {
      "name": "EventAdapterMetadata", // should be capitalized
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "docs": [
              "Signing authority over this Adapter"
            ],
            "type": "publicKey"
          },
          {
            "name": "manager",
            "docs": [
              "The `MarketManager` this adapter belongs to"
            ],
            "type": "publicKey"
          },
          {
            "name": "orderbookUser",
            "docs": [
              "The `MarginUser` account this adapter is registered for"
            ],
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "ClaimTicket",  // should be capitalized
      "docs": [
        "A `ClaimTicket` represents a claim of tickets that have been staked with the program",
        "This account is generated by the `StakeMarketTickets` program instruction"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "docs": [
              "The account registered as owner of this claim"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketManager",
            "docs": [
              "The `TicketManager` this claim ticket was established under",
              "Determines the asset this ticket will be redeemed for"
            ],
            "type": "publicKey"
          },
          {
            "name": "maturationTimestamp",
            "docs": [
              "The slot after which this claim can be redeemed for the underlying value"
            ],
            "type": "i64"
          },
          {
            "name": "redeemable",
            "docs": [
              "The number of tokens this claim  is redeemable for"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "SplitTicket", // should be capitalized
      "docs": [
        "A split ticket represents a claim of underlying tokens as the result of a lending action.",
        "",
        "The split ticket is generated when a user places a matched order with the `auto_stake` flag set to true.",
        "By taking the difference between the matched base and quote quantities, the split ticket assigns principal and",
        "interest values."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "docs": [
              "The account registered as owner of this claim"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketManager",
            "docs": [
              "The `TicketManager` this claim ticket was established under",
              "Determines the asset this ticket will be redeemed for"
            ],
            "type": "publicKey"
          },
          {
            "name": "orderTag",
            "docs": [
              "The `OrderTag` associated with the creation of this struct"
            ],
            "type": {
              "array": ["u8", 16] // should be ["u8", 16]
            }
          },
          {
            "name": "struckTimestamp",
            "docs": [
              "The time slot during which the ticket was struck"
            ],
            "type": "i64"
          },
          {
            "name": "maturationTimestamp",
            "docs": [
              "The slot after which this claim can be redeemed for the underlying value"
            ],
            "type": "i64"
          },
          {
            "name": "principal",
            "docs": [
              "The total number of principal tokens the market ticket was struck for"
            ],
            "type": "u64"
          },
          {
            "name": "interest",
            "docs": [
              "The total number of interest tokens struck for this market ticket",
              "same underlying asset as the principal token"
            ],
            "type": "u64"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "InitializeMarketManagerParams",
      "docs": [
        "Parameters for the initialization of the [MarketManager]"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "versionTag",
            "docs": [
              "Tag information for the `MarketManager` account"
            ],
            "type": "u64"
          },
          {
            "name": "seed",
            "docs": [
              "This seed allows the creation of many separate ticket managers tracking different",
              "parameters, such as staking tenor"
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "borrowTenor",
            "docs": [
              "Length of time before a borrow is marked as due, in seconds"
            ],
            "type": "i64"
          },
          {
            "name": "lendTenor",
            "docs": [
              "Length of time before a claim is marked as mature, in seconds"
            ],
            "type": "i64"
          },
          {
            "name": "originationFee",
            "docs": [
              "assessed on borrows. scaled by origination_fee::FEE_UNIT"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "InitializeOrderbookParams",
      "docs": [
        "Parameters necessary for orderbook initialization"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "minBaseOrderSize",
            "docs": [
              "The minimum order size that can be inserted into the orderbook after matching."
            ],
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
            "docs": [
              "The sequence number for the next obligation to be created"
            ],
            "type": "u64"
          },
          {
            "name": "nextUnpaidObligationSeqno",
            "docs": [
              "The sequence number of the next obligation to be paid"
            ],
            "type": "u64"
          },
          {
            "name": "nextObligationMaturity",
            "docs": [
              "The maturation timestamp of the next obligation that is unpaid"
            ],
            "type": "u64" // should be "u64"
          },
          {
            "name": "pending",
            "docs": [
              "Amount that must be collateralized because there is an open order for it.",
              "Does not accrue interest because the loan has not been received yet."
            ],
            "type": "u64"
          },
          {
            "name": "committed",
            "docs": [
              "Debt that has already been borrowed because the order was matched.",
              "This debt will be due when the loan term ends.",
              "This includes all debt, including past due debt"
            ],
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
            "docs": [
              "tokens to transfer into settlement account"
            ],
            "type": "u64"
          },
          {
            "name": "entitledTickets",
            "docs": [
              "tickets to transfer into settlement account"
            ],
            "type": "u64"
          },
          {
            "name": "ticketsStaked",
            "docs": [
              "The number of market tickets locked up in ClaimTicket or SplitTicket"
            ],
            "type": "u64"
          },
          {
            "name": "postedQuote",
            "docs": [
              "The amount of quote included in all orders posted by the user for both",
              "bids and asks. Since the orderbook tracks base, not quote, this is only",
              "an approximation. This value must always be less than or equal to the",
              "actual posted quote."
            ],
            "type": "u64"
          },
          {
            "name": "reserved0",
            "docs": [
              "reserved data that may be used to determine the size of a user's collateral",
              "pessimistically prepared to persist aggregated values for:",
              "base and quote quantities, separately for bid/ask, on open orders and unsettled fills",
              "2^3 = 8 u64's"
            ],
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
            "docs": [
              "Total capacity of the adapter",
              "Increases rent cost"
            ],
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "OrderParams",
      "docs": [
        "Parameters needed for order placement"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxMarketTicketQty",
            "docs": [
              "The maximum quantity of market tickets to be traded."
            ],
            "type": "u64"
          },
          {
            "name": "maxUnderlyingTokenQty",
            "docs": [
              "The maximum quantity of underlying token to be traded."
            ],
            "type": "u64"
          },
          {
            "name": "limitPrice",
            "docs": [
              "The limit price of the order. This value is understood as a 32-bit fixed point number."
            ],
            "type": "u64"
          },
          {
            "name": "matchLimit",
            "docs": [
              "The maximum number of orderbook postings to match in order to fulfill the order"
            ],
            "type": "u64"
          },
          {
            "name": "postOnly",
            "docs": [
              "The order will not be matched against the orderbook and will be direcly written into it.",
              "",
              "The operation will fail if the order's limit_price crosses the spread."
            ],
            "type": "bool"
          },
          {
            "name": "postAllowed",
            "docs": [
              "Should the unfilled portion of the order be reposted to the orderbook"
            ],
            "type": "bool"
          },
          {
            "name": "autoStake",
            "docs": [
              "Should the purchased tickets be automatically staked with the ticket program"
            ],
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "StakeMarketTicketsParams",
      "docs": [
        "Params needed to stake market tickets"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "amount",
            "docs": [
              "number of tickets to stake"
            ],
            "type": "u64"
          },
          {
            "name": "ticketSeed",
            "docs": [
              "uniqueness seed to allow a user to have many `ClaimTicket`s"
            ],
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
      "docs": [
        "These are the additional accounts that need to be provided in the ix",
        "for every event that will be processed.",
        "For a fill, 2-6 accounts need to be appended to remaining_accounts",
        "For an out, 1 account needs to be appended to remaining_accounts"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill",
            "fields": [
              {
                "defined": "FillAccounts<'info>"
              }
            ]
          },
          {
            "name": "Out",
            "fields": [
              {
                "defined": "OutAccounts<'info>"
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
      "name": "PreparedEvent",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill",
            "fields": [
              {
                "defined": "FillAccounts<'info>"
              },
              {
                "defined": "FillInfo"
              }
            ]
          },
          {
            "name": "Out",
            "fields": [
              {
                "defined": "OutAccounts<'info>"
              },
              {
                "defined": "OutInfo"
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
      "docs": [
        "Enum used for pattern matching a ticket deserialization"
      ],
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
      "name": "MarketManagerInitialized",
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
          "name": "borrowTenor",
          "type": "i64",
          "index": false
        },
        {
          "name": "lendTenor",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "OrderbookInitialized",
      "fields": [
        {
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "type": {
            "option": "publicKey"
          },
          "index": false
        },
        {
          "name": "orderType",
          "type": {
            "defined": "OrderType"
          },
          "index": false
        },
        {
          "name": "orderSummary",
          "type": {
            "array": ["u8", 48] // should be ["u8", 48]
          },
          "index": false
        },
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
      "name": "ObligationCreated",
      "fields": [
        {
          "name": "obligation",
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
          "type": {
            "option": "u128"
          },
          "index": false
        },
        {
          "name": "sequenceNumber",
          "type": "u64",
          "index": false
        },
        {
          "name": "marketManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "maturationTimestamp",
          "type": "i64",
          "index": false
        },
        {
          "name": "quoteFilled",
          "type": "u64",
          "index": false
        },
        {
          "name": "baseFilled",
          "type": "u64",
          "index": false
        },
        {
          "name": "flags",
          "type": "u8", // should be "u8"
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
      "name": "ImmatureMarketTicket",
      "msg": "market ticket is not mature and cannot be claimed"
    },
    {
      "code": 6008,
      "name": "InsufficientSeeds",
      "msg": "not enough seeds were provided for the accounts that need to be initialized"
    },
    {
      "code": 6009,
      "name": "InvalidOrderPrice",
      "msg": "order price is prohibited"
    },
    {
      "code": 6010,
      "name": "InvokeCreateAccount",
      "msg": "failed to invoke account creation"
    },
    {
      "code": 6011,
      "name": "IoError",
      "msg": "failed to properly serialize or deserialize a data structure"
    },
    {
      "code": 6012,
      "name": "MarketStateNotProgramOwned",
      "msg": "this market state account is not owned by the current program"
    },
    {
      "code": 6013,
      "name": "MissingEventAdapter",
      "msg": "tried to access a missing adapter account"
    },
    {
      "code": 6014,
      "name": "MissingSplitTicket",
      "msg": "tried to access a missing split ticket account"
    },
    {
      "code": 6015,
      "name": "NoEvents",
      "msg": "consume_events instruction failed to consume a single event"
    },
    {
      "code": 6016,
      "name": "NoMoreAccounts",
      "msg": "expected additional remaining accounts, but there were none"
    },
    {
      "code": 6017,
      "name": "ObligationHasWrongSequenceNumber",
      "msg": "expected an obligation with a different sequence number"
    },
    {
      "code": 6018,
      "name": "OracleError",
      "msg": "there was a problem loading the price oracle"
    },
    {
      "code": 6019,
      "name": "OrderNotFound",
      "msg": "id was not found in the user's open orders"
    },
    {
      "code": 6020,
      "name": "OrderbookPaused",
      "msg": "Orderbook is not taking orders"
    },
    {
      "code": 6021,
      "name": "OrderRejected",
      "msg": "aaob did not match or post the order. either posting is disabled or the order was too small"
    },
    {
      "code": 6022,
      "name": "PriceMissing",
      "msg": "price could not be accessed from oracle"
    },
    {
      "code": 6023,
      "name": "TicketNotFromManager",
      "msg": "claim ticket is not from this manager"
    },
    {
      "code": 6024,
      "name": "TicketsPaused",
      "msg": "tickets are paused"
    },
    {
      "code": 6025,
      "name": "UnauthorizedCaller",
      "msg": "this signer is not authorized to place a permissioned order"
    },
    {
      "code": 6026,
      "name": "UserDoesNotOwnAccount",
      "msg": "this user does not own the user account"
    },
    {
      "code": 6027,
      "name": "UserDoesNotOwnAdapter",
      "msg": "this adapter does not belong to the user"
    },
    {
      "code": 6028,
      "name": "UserNotInMarket",
      "msg": "this user account is not associated with this fixed market"
    },
    {
      "code": 6029,
      "name": "WrongAdapter",
      "msg": "the wrong adapter account was passed to this instruction"
    },
    {
      "code": 6030,
      "name": "WrongAsks",
      "msg": "asks account does not belong to this market"
    },
    {
      "code": 6031,
      "name": "WrongAirspace",
      "msg": "the market is configured for a different airspace"
    },
    {
      "code": 6032,
      "name": "WrongAirspaceAuthorization",
      "msg": "the signer is not authorized to perform this action in the current airspace"
    },
    {
      "code": 6033,
      "name": "WrongBids",
      "msg": "bids account does not belong to this market"
    },
    {
      "code": 6035,
      "name": "WrongMarketManager",
      "msg": "adapter does not belong to given market manager"
    },
    {
      "code": 6035,
      "name": "WrongCrankAuthority",
      "msg": "wrong authority for this crank instruction"
    },
    {
      "code": 6036,
      "name": "WrongEventQueue",
      "msg": "event queue account does not belong to this market"
    },
    {
      "code": 6037,
      "name": "WrongMarketState",
      "msg": "this market state is not associated with this market"
    },
    {
      "code": 6038,
      "name": "WrongTicketManager",
      "msg": "wrong TicketManager account provided"
    },
    {
      "code": 6039,
      "name": "DoesNotOwnMarket",
      "msg": "this market owner does not own this market"
    },
    {
      "code": 6040,
      "name": "WrongClaimAccount",
      "msg": "the wrong account was provided for the token account that represents a user's claims"
    },
    {
      "code": 6041,
      "name": "WrongCollateralAccount",
      "msg": "the wrong account was provided for the token account that represents a user's collateral"
    },
    {
      "code": 6042,
      "name": "WrongClaimMint",
      "msg": "the wrong account was provided for the claims token mint"
    },
    {
      "code": 6043,
      "name": "WrongCollateralMint",
      "msg": "the wrong account was provided for the collateral token mint"
    },
    {
      "code": 6044,
      "name": "WrongFeeDestination",
      "msg": "wrong fee destination"
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
      "msg": "not the ticket mint for this fixed market"
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
      "msg": "wrong underlying token mint for this fixed market"
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

export const IDL: JetMarket = {
  "version": "0.1.0",
  "name": "jet_market",
  "constants": [
    {
      "name": "MARKET_MANAGER",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"market_manager\""
    },
    {
      "name": "MARKET_TICKET_ACCOUNT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"market_ticket_account\""
    },
    {
      "name": "MARKET_TICKET_MINT",
      "type": {
        "defined": "&[u8]"
      },
      "value": "b\"market_ticket_mint\""
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
      "docs": [
        "authorize an address to run orderbook consume_event instructions"
      ],
      "accounts": [
        {
          "name": "crank",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The crank signer pubkey"
          ]
        },
        {
          "name": "crankAuthorization",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account containing the metadata for the key"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The address paying the rent for the account"
          ]
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
      "docs": [
        "unauthorize an address to run orderbook consume_event instructions"
      ],
      "accounts": [
        {
          "name": "metadataAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account containing the metadata for the key"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
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
      "name": "initializeMarketManager",
      "docs": [
        "Initializes a MarketManager for a fixed market"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The `MarketManager` manages asset tokens for a particular market tenor"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault for storing the token underlying the market tickets"
          ]
        },
        {
          "name": "underlyingTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint for the assets underlying the market tickets"
          ]
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The minting account for the market tickets"
          ]
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Mints tokens to a margin account to represent debt that must be collateralized"
          ]
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Mints tokens to a margin account to represent debt that must be collateralized"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        },
        {
          "name": "underlyingOracle",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The oracle for the underlying asset price"
          ]
        },
        {
          "name": "ticketOracle",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The oracle for the market ticket price"
          ]
        },
        {
          "name": "feeDestination",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The account where fees are allowed to be withdrawn"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The account paying rent for PDA initialization"
          ]
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Rent sysvar"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Solana system program"
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "InitializeMarketManagerParams"
          }
        }
      ]
    },
    {
      "name": "initializeOrderbook",
      "docs": [
        "Initializes a new orderbook"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The `MarketManager` account tracks global information related to this particular fixed market"
          ]
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AOB market state"
          ]
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AOB market event queue",
            "",
            "Must be initialized"
          ]
        },
        {
          "name": "bids",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AOB market bids"
          ]
        },
        {
          "name": "asks",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AOB market asks"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The account paying rent for PDA initialization"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Solana system program"
          ]
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
      "name": "modifyMarketManager",
      "docs": [
        "Modify a `MarketManager` account",
        "Authority use only"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The `MarketManager` manages asset tokens for a particular market tenor"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        }
      ],
      "args": [
        {
          "name": "data",
          "type": "bytes"
        },
        {
          "name": "offset",
          "type": "u64" // should be "u64"
        }
      ]
    },
    {
      "name": "pauseOrderMatching",
      "docs": [
        "Pause matching of orders placed in the orderbook"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The `MarketManager` manages asset tokens for a particular market tenor"
          ]
        },
        {
          "name": "orderbookMarketState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "resumeOrderMatching",
      "docs": [
        "Resume matching of orders placed in the orderbook",
        "NOTE: This instruction may have to be run several times to clear the",
        "existing matches. Check the `orderbook_market_state.pause_matching` variable",
        "to determine success"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The `MarketManager` manages asset tokens for a particular market tenor"
          ]
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
          "isSigner": true,
          "docs": [
            "The authority that must sign to make this change"
          ]
        },
        {
          "name": "airspace",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The airspace being modified"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "initializeMarginUser",
      "docs": [
        "Create a new borrower account"
      ],
      "accounts": [
        {
          "name": "borrowerAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking information related to this particular user"
          ]
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The signing authority for this user account"
          ]
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The Boheader account"
          ]
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt",
            "that must be collateralized"
          ]
        },
        {
          "name": "claimsMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track owned assets"
          ]
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
          "isSigner": false,
          "docs": [
            "Token metadata account needed by the margin program to register the claim position"
          ]
        },
        {
          "name": "collateralMetadata",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token metadata account needed by the margin program to register the collateral position"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "marginBorrowOrder",
      "docs": [
        "Place a borrow order by leveraging margin account value"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking borrower debts"
          ]
        },
        {
          "name": "obligation",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Obligation account minted upon match"
          ]
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The margin account for this borrow order"
          ]
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "claimsMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
          ]
        },
        {
          "name": "underlyingSettlement",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
          ]
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "marketManager",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The `MarketManager` account tracks global information related to this particular fixed market"
              ]
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
          "isSigner": true,
          "docs": [
            "payer for `Obligation` initialization"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Solana system program"
          ]
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
      "docs": [
        "Sell tickets that are already owned"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking borrower debts"
          ]
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "authority",
              "isMut": false,
              "isSigner": true,
              "docs": [
                "Signing authority over the ticket vault transferring for a borrow order"
              ]
            },
            {
              "name": "userTicketVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "Account containing the market tickets being sold"
              ]
            },
            {
              "name": "userTokenVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The account to receive the matched tokens"
              ]
            },
            {
              "name": "orderbookMut",
              "accounts": [
                {
                  "name": "marketManager",
                  "isMut": true,
                  "isSigner": false,
                  "docs": [
                    "The `MarketManager` account tracks global information related to this particular fixed market"
                  ]
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
              "name": "marketTicketMint",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The market ticket mint"
              ]
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The market ticket mint"
              ]
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
      "docs": [
        "Redeem a staked ticket"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the collateral value of assets custodied by Jet markets"
          ]
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the collateral value of assets custodied by Jet markets"
          ]
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "ticket",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "One of either `SplitTicket` or `ClaimTicket` for redemption"
              ]
            },
            {
              "name": "authority",
              "isMut": true,
              "isSigner": true,
              "docs": [
                "The account that must sign to redeem the ticket"
              ]
            },
            {
              "name": "claimantTokenAccount",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The token account designated to receive the assets underlying the claim"
              ]
            },
            {
              "name": "marketManager",
              "isMut": false,
              "isSigner": false,
              "docs": [
                "The MarketManager responsible for the asset"
              ]
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The vault stores the tokens of the underlying asset managed by the MarketManager"
              ]
            },
            {
              "name": "tokenProgram",
              "isMut": false,
              "isSigner": false,
              "docs": [
                "SPL token program"
              ]
            }
          ]
        }
      ],
      "args": []
    },
    {
      "name": "marginLendOrder",
      "docs": [
        "Place a `Lend` order to the book by depositing tokens"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking borrower debts"
          ]
        },
        {
          "name": "collateral",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "collateralMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "inner",
          "accounts": [
            {
              "name": "authority",
              "isMut": false,
              "isSigner": true,
              "docs": [
                "Signing authority over the token vault transferring for a lend order"
              ]
            },
            {
              "name": "orderbookMut",
              "accounts": [
                {
                  "name": "marketManager",
                  "isMut": true,
                  "isSigner": false,
                  "docs": [
                    "The `MarketManager` account tracks global information related to this particular fixed market"
                  ]
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
              "isSigner": false,
              "docs": [
                "where to settle tickets on match:",
                "- SplitTicket that will be created if the order is filled as a taker and `auto_stake` is enabled",
                "- ticket token account to receive market tickets",
                "be careful to check this properly. one way is by using lender_tickets_token_account"
              ]
            },
            {
              "name": "lenderTokens",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "where to loan tokens from"
              ]
            },
            {
              "name": "underlyingTokenVault",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The market token vault"
              ]
            },
            {
              "name": "ticketMint",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The market token vault"
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
      "docs": [
        "Refresh the associated margin account `claims` for a given `MarginUser` account"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The account tracking information related to this particular user"
          ]
        },
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The `MarketManager` account tracks global information related to this particular fixed market"
          ]
        },
        {
          "name": "underlyingOracle",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The pyth price account"
          ]
        },
        {
          "name": "ticketOracle",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
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
      "docs": [
        "Repay debt on an Obligation"
      ],
      "accounts": [
        {
          "name": "borrowerAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking information related to this particular user"
          ]
        },
        {
          "name": "obligation",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "nextObligation",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "No payment will be made towards next_obligation: it is needed purely for bookkeeping.",
            "if the user has additional obligations, this must be the one with the following sequence number.",
            "otherwise, put whatever address you want in here"
          ]
        },
        {
          "name": "source",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The token account to deposit tokens from"
          ]
        },
        {
          "name": "payer",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The signing authority for the source_account"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The token vault holding the underlying token of the market ticket"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
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
      "docs": [
        "Settle payments to a margin account"
      ],
      "accounts": [
        {
          "name": "marginUser",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking information related to this particular user"
          ]
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The `MarketManager` account tracks global information related to this particular fixed market"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
        },
        {
          "name": "claims",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token account used by the margin program to track the debt that must be collateralized"
          ]
        },
        {
          "name": "claimsMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ]
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
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "underlyingSettlement",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ticketSettlement",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "sellTicketsOrder",
      "docs": [
        "Place an order to the book to sell tickets, which will burn them"
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Signing authority over the ticket vault transferring for a borrow order"
          ]
        },
        {
          "name": "userTicketVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Account containing the market tickets being sold"
          ]
        },
        {
          "name": "userTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account to receive the matched tokens"
          ]
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "marketManager",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The `MarketManager` account tracks global information related to this particular fixed market"
              ]
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
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market ticket mint"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market ticket mint"
          ]
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
      "docs": [
        "Cancels an order on the book"
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The owner of the order"
          ]
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "marketManager",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The `MarketManager` account tracks global information related to this particular fixed market"
              ]
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
      "docs": [
        "Place a `Lend` order to the book by depositing tokens"
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Signing authority over the token vault transferring for a lend order"
          ]
        },
        {
          "name": "orderbookMut",
          "accounts": [
            {
              "name": "marketManager",
              "isMut": true,
              "isSigner": false,
              "docs": [
                "The `MarketManager` account tracks global information related to this particular fixed market"
              ]
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
          "isSigner": false,
          "docs": [
            "where to settle tickets on match:",
            "- SplitTicket that will be created if the order is filled as a taker and `auto_stake` is enabled",
            "- ticket token account to receive market tickets",
            "be careful to check this properly. one way is by using lender_tickets_token_account"
          ]
        },
        {
          "name": "lenderTokens",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "where to loan tokens from"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
          ]
        },
        {
          "name": "ticketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
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
      "name": "consumeEvents",
      "docs": [
        "Crank specific instruction, processes the event queue"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The `MarketManager` account tracks global information related to this particular fixed market"
          ]
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market ticket mint"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The market token vault"
          ]
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
          "isSigner": true,
          "docs": [
            "The account paying rent for PDA initialization"
          ]
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
      "docs": [
        "Exchange underlying token for fixed market tickets",
        "WARNING: tickets must be staked for redeption of underlying"
      ],
      "accounts": [
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The MarketManager manages asset tokens for a particular market tenor"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault stores the tokens of the underlying asset managed by the MarketManager"
          ]
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The minting account for the market tickets"
          ]
        },
        {
          "name": "userMarketTicketVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The token account to receive the exchanged market tickets"
          ]
        },
        {
          "name": "userUnderlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user controlled token account to exchange for market tickets"
          ]
        },
        {
          "name": "userAuthority",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The signing authority in charge of the user's underlying token vault"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
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
      "docs": [
        "Redeems staked tickets for their underlying value"
      ],
      "accounts": [
        {
          "name": "ticket",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "One of either `SplitTicket` or `ClaimTicket` for redemption"
          ]
        },
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The account that must sign to redeem the ticket"
          ]
        },
        {
          "name": "claimantTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The token account designated to receive the assets underlying the claim"
          ]
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The MarketManager responsible for the asset"
          ]
        },
        {
          "name": "underlyingTokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault stores the tokens of the underlying asset managed by the MarketManager"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL token program"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "stakeMarketTickets",
      "docs": [
        "Stakes market tickets for later redemption"
      ],
      "accounts": [
        {
          "name": "claimTicket",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "A struct used to track maturation and total claimable funds"
          ]
        },
        {
          "name": "marketManager",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The MarketManager account tracks fixed market assets of a particular tenor"
          ]
        },
        {
          "name": "ticketHolder",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The owner of market tickets that wishes to stake them for a redeemable ticket"
          ]
        },
        {
          "name": "marketTicketTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The account tracking the ticket_holder's market tickets"
          ]
        },
        {
          "name": "marketTicketMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The mint for the market tickets for this instruction",
            "A mint is a specific instance of the token program for both the underlying asset and the market tenor"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "The payer for account initialization"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The global on-chain `TokenProgram` for account authority transfer."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The global on-chain `SystemProgram` for program account initialization."
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "StakeMarketTicketsParams"
          }
        }
      ]
    },
    {
      "name": "tranferTicketOwnership",
      "docs": [
        "Transfer staked tickets to a new owner"
      ],
      "accounts": [
        {
          "name": "ticket",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The ticket to transfer, either a ClaimTicket or SplitTicket"
          ]
        },
        {
          "name": "currentOwner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The current owner of the ticket"
          ]
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
      "docs": [
        "Register a new EventAdapter for syncing to the orderbook events"
      ],
      "accounts": [
        {
          "name": "adapterQueue",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AdapterEventQueue account owned by outside user or program"
          ]
        },
        {
          "name": "marketManager",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "MarketManager for this Adapter"
          ]
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Signing authority over this queue"
          ]
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Payer for the initialization rent of the queue"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "solana system program"
          ]
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
      "docs": [
        "Pop the given number of events off the adapter queue",
        "Event logic is left to the outside program"
      ],
      "accounts": [
        {
          "name": "adapterQueue",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "AdapterEventQueue account owned by outside user or program"
          ]
        },
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Signing authority over the AdapterEventQueue"
          ]
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
      "name": "MarketManager", // should be capitalized
      "docs": [
        "The `MarketManager` contains all the information necessary to run the fixed market",
        "",
        "Utilized by program instructions to verify given transaction accounts are correct. Contains data",
        "about the fixed market including the tenor and ticket<->token conversion rate"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "versionTag",
            "docs": [
              "Versioning and tag information"
            ],
            "type": "u64"
          },
          {
            "name": "airspace",
            "docs": [
              "The airspace the market is a part of"
            ],
            "type": "publicKey"
          },
          {
            "name": "orderbookMarketState",
            "docs": [
              "The market state of the agnostic orderbook"
            ],
            "type": "publicKey"
          },
          {
            "name": "eventQueue",
            "docs": [
              "The orderbook event queue"
            ],
            "type": "publicKey"
          },
          {
            "name": "asks",
            "docs": [
              "The orderbook asks byteslab"
            ],
            "type": "publicKey"
          },
          {
            "name": "bids",
            "docs": [
              "The orderbook bids byteslab"
            ],
            "type": "publicKey"
          },
          {
            "name": "underlyingTokenMint",
            "docs": [
              "The token mint for the underlying asset of the market tickets"
            ],
            "type": "publicKey"
          },
          {
            "name": "underlyingTokenVault",
            "docs": [
              "Token account storing the underlying asset accounted for by this ticket program"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketTicketMint",
            "docs": [
              "The token mint for the market tickets"
            ],
            "type": "publicKey"
          },
          {
            "name": "claimsMint",
            "docs": [
              "Mint owned by Jet markets to issue claims against a user.",
              "These claim notes are monitored by margin to ensure claims are repaid."
            ],
            "type": "publicKey"
          },
          {
            "name": "collateralMint",
            "docs": [
              "Mint owned by Jet markets to issue collateral value to a user",
              "The collateral notes are monitored by the margin program to track value"
            ],
            "type": "publicKey"
          },
          {
            "name": "underlyingOracle",
            "docs": [
              "oracle that defines the value of the underlying asset"
            ],
            "type": "publicKey"
          },
          {
            "name": "ticketOracle",
            "docs": [
              "oracle that defines the value of the market tickets"
            ],
            "type": "publicKey"
          },
          {
            "name": "feeDestination",
            "docs": [
              "where fees can be withdrawn to"
            ],
            "type": "publicKey"
          },
          {
            "name": "seed",
            "docs": [
              "The user-defined part of the seed that generated this market manager's PDA"
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "bump",
            "docs": [
              "The bump seed value for generating the authority address."
            ],
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "orderbookPaused",
            "docs": [
              "Is the market taking orders"
            ],
            "type": "bool"
          },
          {
            "name": "ticketsPaused",
            "docs": [
              "Can tickets be redeemed"
            ],
            "type": "bool"
          },
          {
            "name": "reserved",
            "docs": [
              "reserved for future use"
            ],
            "type": {
              "array": [
                "u8",
                28
              ]
            }
          },
          {
            "name": "borrowTenor",
            "docs": [
              "Length of time before a borrow is marked as due, in seconds"
            ],
            "type": "i64"
          },
          {
            "name": "lendTenor",
            "docs": [
              "Length of time before a claim is marked as mature, in seconds"
            ],
            "type": "i64"
          },
          {
            "name": "originationFee",
            "docs": [
              "assessed on borrows. scaled by origination_fee::FEE_UNIT"
            ],
            "type": "u64"
          },
          {
            "name": "collectedFees",
            "docs": [
              "amount of fees currently available to be withdrawn by market owner"
            ],
            "type": "u64"
          },
          {
            "name": "nonce",
            "docs": [
              "Used to generate unique order tags"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "CrankAuthorization", // should be capitalized
      "docs": [
        "This authorizes a crank to act on any orderbook within the airspace"
      ],
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
      "name": "MarginUser", // should be capitalized
      "docs": [
        "An acocunt used to track margin users of the market"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "docs": [
              "used to determine if a migration step is needed before user actions are allowed"
            ],
            "type": "u8"
          },
          {
            "name": "marginAccount",
            "docs": [
              "The margin account used for signing actions"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketManager",
            "docs": [
              "The `MarketManager` for the market"
            ],
            "type": "publicKey"
          },
          {
            "name": "claims",
            "docs": [
              "Token account used by the margin program to track the debt"
            ],
            "type": "publicKey"
          },
          {
            "name": "collateral",
            "docs": [
              "Token account used by the margin program to track the collateral value of positions",
              "which are internal to Jet markets, such as SplitTicket, ClaimTicket, and open orders.",
              "this does *not* represent underlying tokens or market ticket tokens, those are registered independently in margin"
            ],
            "type": "publicKey"
          },
          {
            "name": "underlyingSettlement",
            "docs": [
              "The `settle` instruction is permissionless, therefore the user must specify upon margin account creation",
              "the address to send owed tokens"
            ],
            "type": "publicKey"
          },
          {
            "name": "ticketSettlement",
            "docs": [
              "The `settle` instruction is permissionless, therefore the user must specify upon margin account creation",
              "the address to send owed tickets"
            ],
            "type": "publicKey"
          },
          {
            "name": "debt",
            "docs": [
              "The amount of debt that must be collateralized or repaid",
              "This debt is expressed in terms of the underlying token - not market tickets"
            ],
            "type": {
              "defined": "Debt"
            }
          },
          {
            "name": "assets",
            "docs": [
              "Accounting used to track assets in custody of the fixed market"
            ],
            "type": {
              "defined": "Assets"
            }
          }
        ]
      }
    },
    {
      "name": "Obligation", // should be capitalized
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "sequenceNumber",
            "type": "u64" // should be "u64"
          },
          {
            "name": "borrowerAccount",
            "docs": [
              "The user borrower account this obligation is assigned to"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketManager",
            "docs": [
              "The market manager where the obligation was created"
            ],
            "type": "publicKey"
          },
          {
            "name": "orderTag",
            "docs": [
              "The `OrderTag` associated with the creation of this `Obligation`"
            ],
            "type": {
              "array": ["u8", 16] // should be ["u8", 16]
            }
          },
          {
            "name": "maturationTimestamp",
            "docs": [
              "The time that the obligation must be repaid"
            ],
            "type": "u64" // should be "u64"
          },
          {
            "name": "balance",
            "docs": [
              "The remaining amount due by the end of the loan term"
            ],
            "type": "u64"
          },
          {
            "name": "flags",
            "docs": [
              "Any boolean flags for this data type compressed to a single byte"
            ],
            "type": "u8" // should be "u8"
          }
        ]
      }
    },
    {
      "name": "EventAdapterMetadata", // should be capitalized
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "docs": [
              "Signing authority over this Adapter"
            ],
            "type": "publicKey"
          },
          {
            "name": "manager",
            "docs": [
              "The `MarketManager` this adapter belongs to"
            ],
            "type": "publicKey"
          },
          {
            "name": "orderbookUser",
            "docs": [
              "The `MarginUser` account this adapter is registered for"
            ],
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "ClaimTicket", // should be capitalized
      "docs": [
        "A `ClaimTicket` represents a claim of tickets that have been staked with the program",
        "This account is generated by the `StakeMarketTickets` program instruction"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "docs": [
              "The account registered as owner of this claim"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketManager",
            "docs": [
              "The `TicketManager` this claim ticket was established under",
              "Determines the asset this ticket will be redeemed for"
            ],
            "type": "publicKey"
          },
          {
            "name": "maturationTimestamp",
            "docs": [
              "The slot after which this claim can be redeemed for the underlying value"
            ],
            "type": "i64"
          },
          {
            "name": "redeemable",
            "docs": [
              "The number of tokens this claim  is redeemable for"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "SplitTicket", // should be capitalized
      "docs": [
        "A split ticket represents a claim of underlying tokens as the result of a lending action.",
        "",
        "The split ticket is generated when a user places a matched order with the `auto_stake` flag set to true.",
        "By taking the difference between the matched base and quote quantities, the split ticket assigns principal and",
        "interest values."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "owner",
            "docs": [
              "The account registered as owner of this claim"
            ],
            "type": "publicKey"
          },
          {
            "name": "marketManager",
            "docs": [
              "The `TicketManager` this claim ticket was established under",
              "Determines the asset this ticket will be redeemed for"
            ],
            "type": "publicKey"
          },
          {
            "name": "orderTag",
            "docs": [
              "The `OrderTag` associated with the creation of this struct"
            ],
            "type": {
              "array": ["u8", 16] // should be ["u8", 16]
            }
          },
          {
            "name": "struckTimestamp",
            "docs": [
              "The time slot during which the ticket was struck"
            ],
            "type": "i64"
          },
          {
            "name": "maturationTimestamp",
            "docs": [
              "The slot after which this claim can be redeemed for the underlying value"
            ],
            "type": "i64"
          },
          {
            "name": "principal",
            "docs": [
              "The total number of principal tokens the market ticket was struck for"
            ],
            "type": "u64"
          },
          {
            "name": "interest",
            "docs": [
              "The total number of interest tokens struck for this market ticket",
              "same underlying asset as the principal token"
            ],
            "type": "u64"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "InitializeMarketManagerParams",
      "docs": [
        "Parameters for the initialization of the [MarketManager]"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "versionTag",
            "docs": [
              "Tag information for the `MarketManager` account"
            ],
            "type": "u64"
          },
          {
            "name": "seed",
            "docs": [
              "This seed allows the creation of many separate ticket managers tracking different",
              "parameters, such as staking tenor"
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "borrowTenor",
            "docs": [
              "Length of time before a borrow is marked as due, in seconds"
            ],
            "type": "i64"
          },
          {
            "name": "lendTenor",
            "docs": [
              "Length of time before a claim is marked as mature, in seconds"
            ],
            "type": "i64"
          },
          {
            "name": "originationFee",
            "docs": [
              "assessed on borrows. scaled by origination_fee::FEE_UNIT"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "InitializeOrderbookParams",
      "docs": [
        "Parameters necessary for orderbook initialization"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "minBaseOrderSize",
            "docs": [
              "The minimum order size that can be inserted into the orderbook after matching."
            ],
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
            "docs": [
              "The sequence number for the next obligation to be created"
            ],
            "type": "u64"
          },
          {
            "name": "nextUnpaidObligationSeqno",
            "docs": [
              "The sequence number of the next obligation to be paid"
            ],
            "type": "u64"
          },
          {
            "name": "nextObligationMaturity",
            "docs": [
              "The maturation timestamp of the next obligation that is unpaid"
            ],
            "type": "u64" // should be "u64"
          },
          {
            "name": "pending",
            "docs": [
              "Amount that must be collateralized because there is an open order for it.",
              "Does not accrue interest because the loan has not been received yet."
            ],
            "type": "u64"
          },
          {
            "name": "committed",
            "docs": [
              "Debt that has already been borrowed because the order was matched.",
              "This debt will be due when the loan term ends.",
              "This includes all debt, including past due debt"
            ],
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
            "docs": [
              "tokens to transfer into settlement account"
            ],
            "type": "u64"
          },
          {
            "name": "entitledTickets",
            "docs": [
              "tickets to transfer into settlement account"
            ],
            "type": "u64"
          },
          {
            "name": "ticketsStaked",
            "docs": [
              "The number of market tickets locked up in ClaimTicket or SplitTicket"
            ],
            "type": "u64"
          },
          {
            "name": "postedQuote",
            "docs": [
              "The amount of quote included in all orders posted by the user for both",
              "bids and asks. Since the orderbook tracks base, not quote, this is only",
              "an approximation. This value must always be less than or equal to the",
              "actual posted quote."
            ],
            "type": "u64"
          },
          {
            "name": "reserved0",
            "docs": [
              "reserved data that may be used to determine the size of a user's collateral",
              "pessimistically prepared to persist aggregated values for:",
              "base and quote quantities, separately for bid/ask, on open orders and unsettled fills",
              "2^3 = 8 u64's"
            ],
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
            "docs": [
              "Total capacity of the adapter",
              "Increases rent cost"
            ],
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "OrderParams",
      "docs": [
        "Parameters needed for order placement"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maxMarketTicketQty",
            "docs": [
              "The maximum quantity of market tickets to be traded."
            ],
            "type": "u64"
          },
          {
            "name": "maxUnderlyingTokenQty",
            "docs": [
              "The maximum quantity of underlying token to be traded."
            ],
            "type": "u64"
          },
          {
            "name": "limitPrice",
            "docs": [
              "The limit price of the order. This value is understood as a 32-bit fixed point number."
            ],
            "type": "u64"
          },
          {
            "name": "matchLimit",
            "docs": [
              "The maximum number of orderbook postings to match in order to fulfill the order"
            ],
            "type": "u64"
          },
          {
            "name": "postOnly",
            "docs": [
              "The order will not be matched against the orderbook and will be direcly written into it.",
              "",
              "The operation will fail if the order's limit_price crosses the spread."
            ],
            "type": "bool"
          },
          {
            "name": "postAllowed",
            "docs": [
              "Should the unfilled portion of the order be reposted to the orderbook"
            ],
            "type": "bool"
          },
          {
            "name": "autoStake",
            "docs": [
              "Should the purchased tickets be automatically staked with the ticket program"
            ],
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "StakeMarketTicketsParams",
      "docs": [
        "Params needed to stake market tickets"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "amount",
            "docs": [
              "number of tickets to stake"
            ],
            "type": "u64"
          },
          {
            "name": "ticketSeed",
            "docs": [
              "uniqueness seed to allow a user to have many `ClaimTicket`s"
            ],
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
      "docs": [
        "These are the additional accounts that need to be provided in the ix",
        "for every event that will be processed.",
        "For a fill, 2-6 accounts need to be appended to remaining_accounts",
        "For an out, 1 account needs to be appended to remaining_accounts"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill",
            "fields": [
              {
                "defined": "FillAccounts<'info>"
              }
            ]
          },
          {
            "name": "Out",
            "fields": [
              {
                "defined": "OutAccounts<'info>"
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
      "name": "PreparedEvent",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Fill",
            "fields": [
              {
                "defined": "FillAccounts<'info>"
              },
              {
                "defined": "FillInfo"
              }
            ]
          },
          {
            "name": "Out",
            "fields": [
              {
                "defined": "OutAccounts<'info>"
              },
              {
                "defined": "OutInfo"
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
      "docs": [
        "Enum used for pattern matching a ticket deserialization"
      ],
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
      "name": "MarketManagerInitialized",
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
          "name": "borrowTenor",
          "type": "i64",
          "index": false
        },
        {
          "name": "lendTenor",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "OrderbookInitialized",
      "fields": [
        {
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "type": {
            "option": "publicKey"
          },
          "index": false
        },
        {
          "name": "orderType",
          "type": {
            "defined": "OrderType"
          },
          "index": false
        },
        {
          "name": "orderSummary",
          "type": {
            "array": ["u8", 48] // should be ["u8", 48] 
          },
          "index": false
        },
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
      "name": "ObligationCreated",
      "fields": [
        {
          "name": "obligation",
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
          "type": {
            "option": "u128"
          },
          "index": false
        },
        {
          "name": "sequenceNumber",
          "type": "u64",
          "index": false
        },
        {
          "name": "marketManager",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "maturationTimestamp",
          "type": "i64",
          "index": false
        },
        {
          "name": "quoteFilled",
          "type": "u64",
          "index": false
        },
        {
          "name": "baseFilled",
          "type": "u64",
          "index": false
        },
        {
          "name": "flags",
          "type": "u8", // should be "u8"
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
          "name": "marketManager",
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
      "name": "ImmatureMarketTicket",
      "msg": "market ticket is not mature and cannot be claimed"
    },
    {
      "code": 6008,
      "name": "InsufficientSeeds",
      "msg": "not enough seeds were provided for the accounts that need to be initialized"
    },
    {
      "code": 6009,
      "name": "InvalidOrderPrice",
      "msg": "order price is prohibited"
    },
    {
      "code": 6010,
      "name": "InvokeCreateAccount",
      "msg": "failed to invoke account creation"
    },
    {
      "code": 6011,
      "name": "IoError",
      "msg": "failed to properly serialize or deserialize a data structure"
    },
    {
      "code": 6012,
      "name": "MarketStateNotProgramOwned",
      "msg": "this market state account is not owned by the current program"
    },
    {
      "code": 6013,
      "name": "MissingEventAdapter",
      "msg": "tried to access a missing adapter account"
    },
    {
      "code": 6014,
      "name": "MissingSplitTicket",
      "msg": "tried to access a missing split ticket account"
    },
    {
      "code": 6015,
      "name": "NoEvents",
      "msg": "consume_events instruction failed to consume a single event"
    },
    {
      "code": 6016,
      "name": "NoMoreAccounts",
      "msg": "expected additional remaining accounts, but there were none"
    },
    {
      "code": 6017,
      "name": "ObligationHasWrongSequenceNumber",
      "msg": "expected an obligation with a different sequence number"
    },
    {
      "code": 6018,
      "name": "OracleError",
      "msg": "there was a problem loading the price oracle"
    },
    {
      "code": 6019,
      "name": "OrderNotFound",
      "msg": "id was not found in the user's open orders"
    },
    {
      "code": 6020,
      "name": "OrderbookPaused",
      "msg": "Orderbook is not taking orders"
    },
    {
      "code": 6021,
      "name": "OrderRejected",
      "msg": "aaob did not match or post the order. either posting is disabled or the order was too small"
    },
    {
      "code": 6022,
      "name": "PriceMissing",
      "msg": "price could not be accessed from oracle"
    },
    {
      "code": 6023,
      "name": "TicketNotFromManager",
      "msg": "claim ticket is not from this manager"
    },
    {
      "code": 6024,
      "name": "TicketsPaused",
      "msg": "tickets are paused"
    },
    {
      "code": 6025,
      "name": "UnauthorizedCaller",
      "msg": "this signer is not authorized to place a permissioned order"
    },
    {
      "code": 6026,
      "name": "UserDoesNotOwnAccount",
      "msg": "this user does not own the user account"
    },
    {
      "code": 6027,
      "name": "UserDoesNotOwnAdapter",
      "msg": "this adapter does not belong to the user"
    },
    {
      "code": 6028,
      "name": "UserNotInMarket",
      "msg": "this user account is not associated with this fixed market"
    },
    {
      "code": 6029,
      "name": "WrongAdapter",
      "msg": "the wrong adapter account was passed to this instruction"
    },
    {
      "code": 6030,
      "name": "WrongAsks",
      "msg": "asks account does not belong to this market"
    },
    {
      "code": 6031,
      "name": "WrongAirspace",
      "msg": "the market is configured for a different airspace"
    },
    {
      "code": 6032,
      "name": "WrongAirspaceAuthorization",
      "msg": "the signer is not authorized to perform this action in the current airspace"
    },
    {
      "code": 6033,
      "name": "WrongBids",
      "msg": "bids account does not belong to this market"
    },
    {
      "code": 6035,
      "name": "WrongMarketManager",
      "msg": "adapter does not belong to given market manager"
    },
    {
      "code": 6035,
      "name": "WrongCrankAuthority",
      "msg": "wrong authority for this crank instruction"
    },
    {
      "code": 6036,
      "name": "WrongEventQueue",
      "msg": "event queue account does not belong to this market"
    },
    {
      "code": 6037,
      "name": "WrongMarketState",
      "msg": "this market state is not associated with this market"
    },
    {
      "code": 6038,
      "name": "WrongTicketManager",
      "msg": "wrong TicketManager account provided"
    },
    {
      "code": 6039,
      "name": "DoesNotOwnMarket",
      "msg": "this market owner does not own this market"
    },
    {
      "code": 6040,
      "name": "WrongClaimAccount",
      "msg": "the wrong account was provided for the token account that represents a user's claims"
    },
    {
      "code": 6041,
      "name": "WrongCollateralAccount",
      "msg": "the wrong account was provided for the token account that represents a user's collateral"
    },
    {
      "code": 6042,
      "name": "WrongClaimMint",
      "msg": "the wrong account was provided for the claims token mint"
    },
    {
      "code": 6043,
      "name": "WrongCollateralMint",
      "msg": "the wrong account was provided for the collateral token mint"
    },
    {
      "code": 6044,
      "name": "WrongFeeDestination",
      "msg": "wrong fee destination"
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
      "msg": "not the ticket mint for this fixed market"
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
      "msg": "wrong underlying token mint for this fixed market"
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
