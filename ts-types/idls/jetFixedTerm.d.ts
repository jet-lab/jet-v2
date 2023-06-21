type JetFixedTermIDL = {
  version: "0.1.0"
  name: "jet_fixed_term"
  constants: [
    {
      name: "MARKET"
      type: "bytes"
      value: "[109, 97, 114, 107, 101, 116]"
    },
    {
      name: "TICKET_ACCOUNT"
      type: "bytes"
      value: "[116, 105, 99, 107, 101, 116, 95, 97, 99, 99, 111, 117, 110, 116]"
    },
    {
      name: "TICKET_MINT"
      type: "bytes"
      value: "[116, 105, 99, 107, 101, 116, 95, 109, 105, 110, 116]"
    },
    {
      name: "CRANK_AUTHORIZATION"
      type: "bytes"
      value: "[99, 114, 97, 110, 107, 95, 97, 117, 116, 104, 111, 114, 105, 122, 97, 116, 105, 111, 110]"
    },
    {
      name: "CLAIM_NOTES"
      type: "bytes"
      value: "[99, 108, 97, 105, 109, 95, 110, 111, 116, 101, 115]"
    },
    {
      name: "TICKET_COLLATERAL_NOTES"
      type: "bytes"
      value: "[116, 105, 99, 107, 101, 116, 95, 99, 111, 108, 108, 97, 116, 101, 114, 97, 108, 95, 110, 111, 116, 101, 115]"
    },
    {
      name: "EVENT_ADAPTER"
      type: "bytes"
      value: "[101, 118, 101, 110, 116, 95, 97, 100, 97, 112, 116, 101, 114]"
    },
    {
      name: "TERM_LOAN"
      type: "bytes"
      value: "[116, 101, 114, 109, 95, 108, 111, 97, 110]"
    },
    {
      name: "TERM_DEPOSIT"
      type: "bytes"
      value: "[116, 101, 114, 109, 95, 100, 101, 112, 111, 115, 105, 116]"
    },
    {
      name: "ORDERBOOK_MARKET_STATE"
      type: "bytes"
      value: "[111, 114, 100, 101, 114, 98, 111, 111, 107, 95, 109, 97, 114, 107, 101, 116, 95, 115, 116, 97, 116, 101]"
    },
    {
      name: "MARGIN_USER"
      type: "bytes"
      value: "[109, 97, 114, 103, 105, 110, 95, 117, 115, 101, 114]"
    },
    {
      name: "UNDERLYING_TOKEN_VAULT"
      type: "bytes"
      value: "[117, 110, 100, 101, 114, 108, 121, 105, 110, 103, 95, 116, 111, 107, 101, 110, 95, 118, 97, 117, 108, 116]"
    },
    {
      name: "FEE_VAULT"
      type: "bytes"
      value: "[102, 101, 101, 95, 118, 97, 117, 108, 116]"
    }
  ]
  instructions: [
    {
      name: "authorizeCrank"
      docs: ["authorize an address to run orderbook consume_event instructions"]
      accounts: [
        {
          name: "crank"
          isMut: false
          isSigner: false
          docs: ["The crank signer pubkey"]
        },
        {
          name: "crankAuthorization"
          isMut: true
          isSigner: false
          docs: ["The account containing the metadata for the key"]
        },
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The market this signer is authorized to send instructions to"]
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that must sign to make this change"]
        },
        {
          name: "airspace"
          isMut: false
          isSigner: false
          docs: ["The airspace being modified"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["The address paying the rent for the account"]
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
      name: "revokeCrank"
      docs: ["unauthorize an address to run orderbook consume_event instructions"]
      accounts: [
        {
          name: "metadataAccount"
          isMut: true
          isSigner: false
          docs: ["The account containing the metadata for the key"]
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that must sign to make this change"]
        },
        {
          name: "airspace"
          isMut: false
          isSigner: false
          docs: ["The airspace being modified"]
        },
        {
          name: "receiver"
          isMut: true
          isSigner: false
        }
      ]
      args: []
    },
    {
      name: "initializeMarket"
      docs: ["Initializes a Market for a fixed term market"]
      accounts: [
        {
          name: "market"
          isMut: true
          isSigner: false
          docs: ["The `Market` manages asset tokens for a particular tenor"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The vault for storing the token underlying the tickets"]
        },
        {
          name: "underlyingTokenMint"
          isMut: false
          isSigner: false
          docs: ["The mint for the assets underlying the tickets"]
        },
        {
          name: "ticketMint"
          isMut: true
          isSigner: false
          docs: ["The minting account for the tickets"]
        },
        {
          name: "claims"
          isMut: true
          isSigner: false
          docs: ["Mints tokens to a margin account to represent debt that must be collateralized"]
        },
        {
          name: "collateral"
          isMut: true
          isSigner: false
          docs: ["Mints tokens to a margin account to represent debt that must be collateralized"]
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that must sign to make this change"]
        },
        {
          name: "airspace"
          isMut: false
          isSigner: false
          docs: ["The airspace being modified"]
        },
        {
          name: "underlyingOracle"
          isMut: false
          isSigner: false
          docs: ["The oracle for the underlying asset price"]
        },
        {
          name: "ticketOracle"
          isMut: false
          isSigner: false
          docs: ["The oracle for the ticket price"]
        },
        {
          name: "feeDestination"
          isMut: false
          isSigner: false
          docs: ["The account where fees are allowed to be withdrawn"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["The account paying rent for PDA initialization"]
        },
        {
          name: "rent"
          isMut: false
          isSigner: false
          docs: ["Rent sysvar"]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
          docs: ["SPL token program"]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
          docs: ["Solana system program"]
        }
      ]
      args: [
        {
          name: "params"
          type: {
            defined: "InitializeMarketParams"
          }
        }
      ]
    },
    {
      name: "initializeOrderbook"
      docs: ["Initializes a new orderbook"]
      accounts: [
        {
          name: "market"
          isMut: true
          isSigner: false
          docs: ["The `Market` account tracks global information related to this particular Jet market"]
        },
        {
          name: "orderbookMarketState"
          isMut: true
          isSigner: false
          docs: ["AOB market state"]
        },
        {
          name: "eventQueue"
          isMut: true
          isSigner: false
          docs: ["AOB market event queue", "", "Must be initialized"]
        },
        {
          name: "bids"
          isMut: true
          isSigner: false
          docs: ["AOB market bids"]
        },
        {
          name: "asks"
          isMut: true
          isSigner: false
          docs: ["AOB market asks"]
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that must sign to make this change"]
        },
        {
          name: "airspace"
          isMut: false
          isSigner: false
          docs: ["The airspace being modified"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["The account paying rent for PDA initialization"]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
          docs: ["Solana system program"]
        }
      ]
      args: [
        {
          name: "params"
          type: {
            defined: "InitializeOrderbookParams"
          }
        }
      ]
    },
    {
      name: "modifyMarket"
      docs: ["Modify a `Market` account", "Authority use only"]
      accounts: [
        {
          name: "market"
          isMut: true
          isSigner: false
          docs: ["The `Market` manages asset tokens for a particular tenor"]
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that must sign to make this change"]
        },
        {
          name: "airspace"
          isMut: false
          isSigner: false
          docs: ["The airspace being modified"]
        }
      ]
      args: [
        {
          name: "data"
          type: "bytes"
        },
        {
          name: "offset"
          type: "u32"
        }
      ]
    },
    {
      name: "pauseOrderMatching"
      docs: ["Pause matching of orders placed in the orderbook"]
      accounts: [
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The `Market` manages asset tokens for a particular tenor"]
        },
        {
          name: "orderbookMarketState"
          isMut: true
          isSigner: false
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that must sign to make this change"]
        },
        {
          name: "airspace"
          isMut: false
          isSigner: false
          docs: ["The airspace being modified"]
        }
      ]
      args: []
    },
    {
      name: "resumeOrderMatching"
      docs: [
        "Resume matching of orders placed in the orderbook",
        "NOTE: This instruction may have to be run several times to clear the",
        "existing matches. Check the `orderbook_market_state.pause_matching` variable",
        "to determine success"
      ]
      accounts: [
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The `Market` manages asset tokens for a particular tenor"]
        },
        {
          name: "orderbookMarketState"
          isMut: true
          isSigner: false
        },
        {
          name: "eventQueue"
          isMut: true
          isSigner: false
        },
        {
          name: "bids"
          isMut: true
          isSigner: false
        },
        {
          name: "asks"
          isMut: true
          isSigner: false
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that must sign to make this change"]
        },
        {
          name: "airspace"
          isMut: false
          isSigner: false
          docs: ["The airspace being modified"]
        }
      ]
      args: []
    },
    {
      name: "autoRollBorrowOrder"
      docs: ["Instruction for authorized servicer to auto roll a `TermLoan` into another order"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The `MarginUser` account for this market"]
        },
        {
          name: "marginAccount"
          isMut: false
          isSigner: false
          docs: ["The `MarginAccount` this `TermDeposit` belongs to"]
        },
        {
          name: "claims"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "claimsMint"
          isMut: true
          isSigner: false
          docs: ["Token mint used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "underlyingCollateral"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "underlyingCollateralMint"
          isMut: true
          isSigner: false
          docs: ["Token mint used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The market token vault"]
        },
        {
          name: "feeVault"
          isMut: true
          isSigner: false
          docs: ["The market fee vault"]
        },
        {
          name: "loan"
          isMut: true
          isSigner: false
          docs: ["The `TermDeposit` account to roll"]
        },
        {
          name: "newLoan"
          isMut: true
          isSigner: false
          docs: ["In the case the order matches, the new `TermLoan` to account for"]
        },
        {
          name: "rentReceiver"
          isMut: true
          isSigner: false
          docs: ["Reciever for rent from the closing of the TermDeposit"]
        },
        {
          name: "orderbookMut"
          accounts: [
            {
              name: "market"
              isMut: true
              isSigner: false
              docs: ["The `Market` account tracks global information related to this particular fixed term market"]
            },
            {
              name: "orderbookMarketState"
              isMut: true
              isSigner: false
            },
            {
              name: "eventQueue"
              isMut: true
              isSigner: false
            },
            {
              name: "bids"
              isMut: true
              isSigner: false
            },
            {
              name: "asks"
              isMut: true
              isSigner: false
            }
          ]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["Payer for PDA initialization"]
        },
        {
          name: "systemProgram"
          isMut: false
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
      name: "autoRollLendOrder"
      docs: ["Instruction for authorized servicer to auto roll a matured `TermDeposit` into another order"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The `MarginUser` account for this market"]
        },
        {
          name: "marginAccount"
          isMut: false
          isSigner: false
          docs: ["The `MarginAccount` this `TermDeposit` belongs to"]
        },
        {
          name: "deposit"
          isMut: true
          isSigner: false
          docs: ["The `TermDeposit` account to roll"]
        },
        {
          name: "newDeposit"
          isMut: true
          isSigner: false
          docs: ["In the case the order matches, the new `TermDeposit` to account for"]
        },
        {
          name: "ticketCollateral"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "ticketCollateralMint"
          isMut: true
          isSigner: false
          docs: ["Token mint used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "ticketMint"
          isMut: true
          isSigner: false
          docs: ["The market token vault"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The market token vault"]
        },
        {
          name: "rentReceiver"
          isMut: true
          isSigner: false
          docs: ["Reciever for rent from the closing of the TermDeposit"]
        },
        {
          name: "orderbookMut"
          accounts: [
            {
              name: "market"
              isMut: true
              isSigner: false
              docs: ["The `Market` account tracks global information related to this particular fixed term market"]
            },
            {
              name: "orderbookMarketState"
              isMut: true
              isSigner: false
            },
            {
              name: "eventQueue"
              isMut: true
              isSigner: false
            },
            {
              name: "bids"
              isMut: true
              isSigner: false
            },
            {
              name: "asks"
              isMut: true
              isSigner: false
            }
          ]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["Payer for PDA initialization"]
        },
        {
          name: "systemProgram"
          isMut: false
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
      name: "configureAutoRollBorrow"
      docs: ["Configure settings for rolling orders"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The `MarginUser` account.", "This account is specific to a particular fixed-term market"]
        },
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The signing authority for this user account"]
        },
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The fixed-term market this user belongs to"]
        }
      ]
      args: [
        {
          name: "config"
          type: {
            defined: "BorrowAutoRollConfig"
          }
        }
      ]
    },
    {
      name: "configureAutoRollLend"
      docs: ["Configure settings for rolling orders"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The `MarginUser` account.", "This account is specific to a particular fixed-term market"]
        },
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The signing authority for this user account"]
        },
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The fixed-term market this user belongs to"]
        }
      ]
      args: [
        {
          name: "config"
          type: {
            defined: "LendAutoRollConfig"
          }
        }
      ]
    },
    {
      name: "toggleAutoRollDeposit"
      docs: ["Toggle the status of a term deposit's auto-roll"]
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The signing authority for this user account"]
        },
        {
          name: "deposit"
          isMut: true
          isSigner: false
          docs: ["The fixed-term market this user belongs to"]
        }
      ]
      args: []
    },
    {
      name: "toggleAutoRollLoan"
      docs: ["Toggle the status of a term loan's auto-roll"]
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The signing authority for this user account"]
        },
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The fixed-term market state for the user"]
        },
        {
          name: "loan"
          isMut: true
          isSigner: false
          docs: ["The fixed-term market this user belongs to"]
        }
      ]
      args: []
    },
    {
      name: "initializeMarginUser"
      docs: ["Create a new borrower account"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The account tracking information related to this particular user"]
        },
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The signing authority for this user account"]
        },
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The fixed-term header account"]
        },
        {
          name: "claims"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track the debt", "that must be collateralized"]
        },
        {
          name: "claimsMint"
          isMut: false
          isSigner: false
        },
        {
          name: "ticketCollateral"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track owned assets"]
        },
        {
          name: "ticketCollateralMint"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenCollateral"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track owned assets"]
        },
        {
          name: "tokenCollateralMint"
          isMut: false
          isSigner: false
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
        },
        {
          name: "rent"
          isMut: false
          isSigner: false
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "claimsMetadata"
          isMut: false
          isSigner: false
          docs: ["Token metadata account needed by the margin program to register the claim position"]
        },
        {
          name: "ticketCollateralMetadata"
          isMut: false
          isSigner: false
          docs: ["Token metadata account needed by the margin program to register the collateral position"]
        },
        {
          name: "tokenCollateralMetadata"
          isMut: false
          isSigner: false
          docs: ["Token metadata account needed by the margin program to register the collateral position"]
        }
      ]
      args: []
    },
    {
      name: "marginBorrowOrder"
      docs: ["Place a borrow order by leveraging margin account value"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The account tracking borrower debts"]
        },
        {
          name: "termLoan"
          isMut: true
          isSigner: false
          docs: ["TermLoan account minted upon match"]
        },
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The margin account for this borrow order"]
        },
        {
          name: "claims"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "claimsMint"
          isMut: true
          isSigner: false
          docs: ["Token mint used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "tokenCollateral"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "tokenCollateralMint"
          isMut: true
          isSigner: false
          docs: ["Token mint used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The market token vault"]
        },
        {
          name: "feeVault"
          isMut: true
          isSigner: false
          docs: ["The market fee vault"]
        },
        {
          name: "underlyingSettlement"
          isMut: true
          isSigner: false
          docs: ["Where to receive borrowed tokens"]
        },
        {
          name: "orderbookMut"
          accounts: [
            {
              name: "market"
              isMut: true
              isSigner: false
              docs: ["The `Market` account tracks global information related to this particular fixed term market"]
            },
            {
              name: "orderbookMarketState"
              isMut: true
              isSigner: false
            },
            {
              name: "eventQueue"
              isMut: true
              isSigner: false
            },
            {
              name: "bids"
              isMut: true
              isSigner: false
            },
            {
              name: "asks"
              isMut: true
              isSigner: false
            }
          ]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["payer for `TermLoan` initialization"]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
          docs: ["Solana system program"]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "params"
          type: {
            defined: "OrderParams"
          }
        }
      ]
    },
    {
      name: "marginSellTicketsOrder"
      docs: ["Sell tickets that are already owned"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The account tracking borrower debts"]
        },
        {
          name: "tokenCollateral"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "tokenCollateralMint"
          isMut: true
          isSigner: false
          docs: ["Token mint used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "inner"
          accounts: [
            {
              name: "authority"
              isMut: false
              isSigner: true
              docs: ["Signing authority over the ticket vault transferring for a borrow order"]
            },
            {
              name: "userTicketVault"
              isMut: true
              isSigner: false
              docs: ["Account containing the tickets being sold"]
            },
            {
              name: "userTokenVault"
              isMut: true
              isSigner: false
              docs: ["The account to receive the matched tokens"]
            },
            {
              name: "orderbookMut"
              accounts: [
                {
                  name: "market"
                  isMut: true
                  isSigner: false
                  docs: ["The `Market` account tracks global information related to this particular fixed term market"]
                },
                {
                  name: "orderbookMarketState"
                  isMut: true
                  isSigner: false
                },
                {
                  name: "eventQueue"
                  isMut: true
                  isSigner: false
                },
                {
                  name: "bids"
                  isMut: true
                  isSigner: false
                },
                {
                  name: "asks"
                  isMut: true
                  isSigner: false
                }
              ]
            },
            {
              name: "ticketMint"
              isMut: true
              isSigner: false
              docs: ["The ticket mint"]
            },
            {
              name: "underlyingTokenVault"
              isMut: true
              isSigner: false
              docs: ["The token vault holding the underlying token of the ticket"]
            },
            {
              name: "tokenProgram"
              isMut: false
              isSigner: false
            }
          ]
        }
      ]
      args: [
        {
          name: "params"
          type: {
            defined: "OrderParams"
          }
        }
      ]
    },
    {
      name: "marginRedeemDeposit"
      docs: ["Redeem a staked ticket"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
        },
        {
          isMut: false,
          isSigner: true,
          name: "marginAccount"
        },
        {
          name: "ticketCollateral"
          isMut: true
          isSigner: false
        },
        {
          name: "ticketCollateralMint"
          isMut: true
          isSigner: false
        },
        {
          isMut: true
          isSigner: false
          name: "deposit"
        },
        {
          isMut: true
          isSigner: false
          name: "payer"
        },
        {
          isMut: true
          isSigner: false
          name: "tokenAccount"
        },
        {
          isMut: false
          isSigner: false
          name: "market"
        },
        {
          isMut: true
          isSigner: false
          name: "underlyingTokenVault"
        },
        {
          isMut: false
          isSigner: false
          name: "tokenProgram"
        }
      ]
      args: []
    },
    {
      name: "marginLendOrder"
      docs: ["Place a `Lend` order to the book by depositing tokens"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The account tracking borrower debts"]
        },
        {
          name: "ticketCollateral"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "ticketCollateralMint"
          isMut: true
          isSigner: false
          docs: ["Token mint used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "marginAccount"
          isMut: false
          isSigner: true
          docs: ["The margin account responsible for this order"]
        },
        {
          name: "orderbookMut"
          accounts: [
            {
              name: "market"
              isMut: true
              isSigner: false
              docs: ["The `Market` account tracks global information related to this particular fixed term market"]
            },
            {
              name: "orderbookMarketState"
              isMut: true
              isSigner: false
            },
            {
              name: "eventQueue"
              isMut: true
              isSigner: false
            },
            {
              name: "bids"
              isMut: true
              isSigner: false
            },
            {
              name: "asks"
              isMut: true
              isSigner: false
            }
          ]
        },
        {
          name: "ticketSettlement"
          isMut: true
          isSigner: false
          docs: [
            "where to settle tickets on match:",
            "- TermDeposit that will be created if the order is filled as a taker and `auto_stake` is enabled",
            "- ticket token account to receive tickets",
            "be careful to check this properly. one way is by using lender_tickets_token_account"
          ]
        },
        {
          name: "lenderTokens"
          isMut: true
          isSigner: false
          docs: ["where to loan tokens from"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The market token vault"]
        },
        {
          name: "ticketMint"
          isMut: true
          isSigner: false
          docs: ["The market token vault"]
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
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "params"
          type: {
            defined: "OrderParams"
          }
        }
      ]
    },
    {
      name: "refreshPosition"
      docs: ["Refresh the associated margin account `claims` for a given `MarginUser` account"]
      accounts: [
        {
          name: "marginUser"
          isMut: false
          isSigner: false
          docs: ["The account tracking information related to this particular user"]
        },
        {
          name: "marginAccount"
          isMut: false
          isSigner: false
        },
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The `Market` account tracks global information related to this particular fixed term market"]
        },
        {
          name: "underlyingOracle"
          isMut: false
          isSigner: false
          docs: ["The pyth price account"]
        },
        {
          name: "ticketOracle"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
          docs: ["SPL token program"]
        }
      ]
      args: [
        {
          name: "expectPrice"
          type: "bool"
        }
      ]
    },
    {
      name: "repay"
      docs: ["Repay debt on an TermLoan"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The account tracking information related to this particular user"]
        },
        {
          name: "termLoan"
          isMut: true
          isSigner: false
        },
        {
          name: "nextTermLoan"
          isMut: false
          isSigner: false
          docs: [
            "No payment will be made towards next_term_loan: it is needed purely for bookkeeping.",
            "if the user has additional term_loan, this must be the one with the following sequence number.",
            "otherwise, put whatever address you want in here"
          ]
        },
        {
          name: "source"
          isMut: true
          isSigner: false
          docs: ["The token account to deposit tokens from"]
        },
        {
          name: "sourceAuthority"
          isMut: false
          isSigner: true
          docs: ["The signing authority for the source_account"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: false
          docs: ["The payer for the `TermLoan` to return rent to"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The token vault holding the underlying token of the ticket"]
        },
        {
          name: "claims"
          isMut: true
          isSigner: false
          docs: ["The token account representing claims for this margin user"]
        },
        {
          name: "claimsMint"
          isMut: true
          isSigner: false
          docs: ["The token account representing claims for this margin user"]
        },
        {
          name: "market"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
          docs: ["SPL token program"]
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
      name: "settle"
      docs: ["Settle payments to a margin account"]
      accounts: [
        {
          name: "marginUser"
          isMut: true
          isSigner: false
          docs: ["The account tracking information related to this particular user"]
        },
        {
          name: "marginAccount"
          isMut: false
          isSigner: false
          docs: ["use accounting_invoke"]
        },
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The `Market` account tracks global information related to this particular fixed term market"]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
          docs: ["SPL token program"]
        },
        {
          name: "claims"
          isMut: true
          isSigner: false
          docs: ["Token account used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "claimsMint"
          isMut: true
          isSigner: false
          docs: ["Token mint used by the margin program to track the debt that must be collateralized"]
        },
        {
          name: "ticketCollateral"
          isMut: true
          isSigner: false
        },
        {
          name: "ticketCollateralMint"
          isMut: true
          isSigner: false
        },
        {
          name: "tokenCollateral"
          isMut: true
          isSigner: false
        },
        {
          name: "tokenCollateralMint"
          isMut: true
          isSigner: false
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
        },
        {
          name: "ticketMint"
          isMut: true
          isSigner: false
        },
        {
          name: "underlyingSettlement"
          isMut: true
          isSigner: false
          docs: ["Where to receive owed tokens"]
        },
        {
          name: "ticketSettlement"
          isMut: true
          isSigner: false
          docs: ["Where to receive owed tickets"]
        }
      ]
      args: []
    },
    {
      name: "sellTicketsOrder"
      docs: ["Place an order to the book to sell tickets, which will burn them"]
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["Signing authority over the ticket vault transferring for a borrow order"]
        },
        {
          name: "userTicketVault"
          isMut: true
          isSigner: false
          docs: ["Account containing the tickets being sold"]
        },
        {
          name: "userTokenVault"
          isMut: true
          isSigner: false
          docs: ["The account to receive the matched tokens"]
        },
        {
          name: "orderbookMut"
          accounts: [
            {
              name: "market"
              isMut: true
              isSigner: false
              docs: ["The `Market` account tracks global information related to this particular fixed term market"]
            },
            {
              name: "orderbookMarketState"
              isMut: true
              isSigner: false
            },
            {
              name: "eventQueue"
              isMut: true
              isSigner: false
            },
            {
              name: "bids"
              isMut: true
              isSigner: false
            },
            {
              name: "asks"
              isMut: true
              isSigner: false
            }
          ]
        },
        {
          name: "ticketMint"
          isMut: true
          isSigner: false
          docs: ["The ticket mint"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The token vault holding the underlying token of the ticket"]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "params"
          type: {
            defined: "OrderParams"
          }
        }
      ]
    },
    {
      name: "cancelOrder"
      docs: ["Cancels an order on the book"]
      accounts: [
        {
          name: "owner"
          isMut: false
          isSigner: true
          docs: ["The owner of the order"]
        },
        {
          name: "orderbookMut"
          accounts: [
            {
              name: "market"
              isMut: true
              isSigner: false
              docs: ["The `Market` account tracks global information related to this particular fixed term market"]
            },
            {
              name: "orderbookMarketState"
              isMut: true
              isSigner: false
            },
            {
              name: "eventQueue"
              isMut: true
              isSigner: false
            },
            {
              name: "bids"
              isMut: true
              isSigner: false
            },
            {
              name: "asks"
              isMut: true
              isSigner: false
            }
          ]
        }
      ]
      args: [
        {
          name: "orderId"
          type: "u128"
        }
      ]
    },
    {
      name: "lendOrder"
      docs: ["Place a `Lend` order to the book by depositing tokens"]
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["Signing authority over the token vault transferring for a lend order"]
        },
        {
          name: "orderbookMut"
          accounts: [
            {
              name: "market"
              isMut: true
              isSigner: false
              docs: ["The `Market` account tracks global information related to this particular fixed term market"]
            },
            {
              name: "orderbookMarketState"
              isMut: true
              isSigner: false
            },
            {
              name: "eventQueue"
              isMut: true
              isSigner: false
            },
            {
              name: "bids"
              isMut: true
              isSigner: false
            },
            {
              name: "asks"
              isMut: true
              isSigner: false
            }
          ]
        },
        {
          name: "ticketSettlement"
          isMut: true
          isSigner: false
          docs: [
            "where to settle tickets on match:",
            "- TermDeposit that will be created if the order is filled as a taker and `auto_stake` is enabled",
            "- ticket token account to receive tickets",
            "be careful to check this properly. one way is by using lender_tickets_token_account"
          ]
        },
        {
          name: "lenderTokens"
          isMut: true
          isSigner: false
          docs: ["where to loan tokens from"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The market token vault"]
        },
        {
          name: "ticketMint"
          isMut: true
          isSigner: false
          docs: ["The market token vault"]
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
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "params"
          type: {
            defined: "OrderParams"
          }
        },
        {
          name: "seed"
          type: "bytes"
        }
      ]
    },
    {
      name: "consumeEvents"
      docs: ["Crank specific instruction, processes the event queue"]
      accounts: [
        {
          name: "market"
          isMut: true
          isSigner: false
          docs: ["The `Market` account tracks global information related to this particular fixed term market"]
        },
        {
          name: "ticketMint"
          isMut: true
          isSigner: false
          docs: ["The ticket mint"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The market token vault"]
        },
        {
          name: "orderbookMarketState"
          isMut: true
          isSigner: false
        },
        {
          name: "eventQueue"
          isMut: true
          isSigner: false
        },
        {
          name: "crankAuthorization"
          isMut: false
          isSigner: false
        },
        {
          name: "crank"
          isMut: false
          isSigner: true
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["The account paying rent for PDA initialization"]
        },
        {
          name: "systemProgram"
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
          name: "numEvents"
          type: "u32"
        },
        {
          name: "seedBytes"
          type: "bytes"
        }
      ]
    },
    {
      name: "exchangeTokens"
      docs: [
        "Exchange underlying token for fixed term tickets",
        "WARNING: tickets must be staked for redeption of underlying"
      ]
      accounts: [
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The Market manages asset tokens for a particular tenor"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The vault stores the tokens of the underlying asset managed by the Market"]
        },
        {
          name: "ticketMint"
          isMut: true
          isSigner: false
          docs: ["The minting account for the tickets"]
        },
        {
          name: "userTicketVault"
          isMut: true
          isSigner: false
          docs: ["The token account to receive the exchanged tickets"]
        },
        {
          name: "userUnderlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The user controlled token account to exchange for tickets"]
        },
        {
          name: "userAuthority"
          isMut: false
          isSigner: true
          docs: ["The signing authority in charge of the user's underlying token vault"]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
          docs: ["SPL token program"]
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
      name: "redeemDeposit"
      docs: ["Redeems deposit previously created by staking tickets for their underlying value"]
      accounts: [
        {
          name: "deposit"
          isMut: true
          isSigner: false
          docs: ["The tracking account for the deposit"]
        },
        {
          name: "owner"
          isMut: true
          isSigner: false
          docs: ["The account that owns the deposit"]
        },
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that must sign to redeem the deposit"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: false
          docs: ["Receiver for the rent used to track the deposit"]
        },
        {
          name: "tokenAccount"
          isMut: true
          isSigner: false
          docs: ["The token account designated to receive the assets underlying the claim"]
        },
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["The Market responsible for the asset"]
        },
        {
          name: "underlyingTokenVault"
          isMut: true
          isSigner: false
          docs: ["The vault stores the tokens of the underlying asset managed by the Market"]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
          docs: ["SPL token program"]
        }
      ]
      args: []
    },
    {
      name: "stakeTickets"
      docs: ["Stakes tickets for later redemption"]
      accounts: [
        {
          name: "deposit"
          isMut: true
          isSigner: false
          docs: ["A struct used to track maturation and total claimable funds"]
        },
        {
          name: "market"
          isMut: true
          isSigner: false
          docs: ["The Market account tracks fixed term market assets of a particular tenor"]
        },
        {
          name: "ticketHolder"
          isMut: false
          isSigner: true
          docs: ["The owner of tickets that wishes to stake them for a redeemable ticket"]
        },
        {
          name: "ticketTokenAccount"
          isMut: true
          isSigner: false
          docs: ["The account tracking the ticket_holder's tickets"]
        },
        {
          name: "ticketMint"
          isMut: true
          isSigner: false
          docs: [
            "The mint for the tickets for this instruction",
            "A mint is a specific instance of the token program for both the underlying asset and the market tenor"
          ]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["The payer for account initialization"]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
          docs: ["The global on-chain `TokenProgram` for account authority transfer."]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
          docs: ["The global on-chain `SystemProgram` for program account initialization."]
        }
      ]
      args: [
        {
          name: "params"
          type: {
            defined: "StakeTicketsParams"
          }
        }
      ]
    },
    {
      name: "tranferTicketOwnership"
      docs: ["Transfer staked tickets to a new owner"]
      accounts: [
        {
          name: "deposit"
          isMut: true
          isSigner: false
          docs: ["The deposit to transfer"]
        },
        {
          name: "owner"
          isMut: false
          isSigner: true
          docs: ["The current owner of the deposit"]
        }
      ]
      args: [
        {
          name: "newOwner"
          type: "publicKey"
        }
      ]
    },
    {
      name: "registerAdapter"
      docs: ["Register a new EventAdapter for syncing to the orderbook events"]
      accounts: [
        {
          name: "adapterQueue"
          isMut: true
          isSigner: false
          docs: ["AdapterEventQueue account owned by outside user or program"]
        },
        {
          name: "market"
          isMut: false
          isSigner: false
          docs: ["Market for this Adapter"]
        },
        {
          name: "owner"
          isMut: false
          isSigner: true
          docs: ["Signing authority over this queue"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["Payer for the initialization rent of the queue"]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
          docs: ["solana system program"]
        }
      ]
      args: [
        {
          name: "params"
          type: {
            defined: "RegisterAdapterParams"
          }
        }
      ]
    },
    {
      name: "popAdapterEvents"
      docs: ["Pop the given number of events off the adapter queue", "Event logic is left to the outside program"]
      accounts: [
        {
          name: "adapterQueue"
          isMut: true
          isSigner: false
          docs: ["AdapterEventQueue account owned by outside user or program"]
        },
        {
          name: "owner"
          isMut: false
          isSigner: true
          docs: ["Signing authority over the AdapterEventQueue"]
        }
      ]
      args: [
        {
          name: "numEvents"
          type: "u32"
        }
      ]
    }
  ]
  accounts: [
    {
      name: "market"
      docs: [
        "The `Market` contains all the information necessary to run the fixed term market",
        "",
        "Utilized by program instructions to verify given transaction accounts are correct. Contains data",
        "about the fixed term market including the tenor and ticket<->token conversion rate"
      ]
      type: {
        kind: "struct"
        fields: [
          {
            name: "versionTag"
            docs: ["Versioning and tag information"]
            type: "u64"
          },
          {
            name: "airspace"
            docs: ["The airspace the market is a part of"]
            type: "publicKey"
          },
          {
            name: "orderbookMarketState"
            docs: ["The market state of the agnostic orderbook"]
            type: "publicKey"
          },
          {
            name: "eventQueue"
            docs: ["The orderbook event queue"]
            type: "publicKey"
          },
          {
            name: "asks"
            docs: ["The orderbook asks byteslab"]
            type: "publicKey"
          },
          {
            name: "bids"
            docs: ["The orderbook bids byteslab"]
            type: "publicKey"
          },
          {
            name: "underlyingTokenMint"
            docs: ["The token mint for the underlying asset of the tickets"]
            type: "publicKey"
          },
          {
            name: "underlyingTokenVault"
            docs: ["Token account storing the underlying asset accounted for by this ticket program"]
            type: "publicKey"
          },
          {
            name: "ticketMint"
            docs: ["The token mint for the tickets"]
            type: "publicKey"
          },
          {
            name: "claimsMint"
            docs: [
              "Mint owned by fixed-term market to issue claims against a user.",
              "These claim notes are monitored by margin to ensure claims are repaid."
            ]
            type: "publicKey"
          },
          {
            name: "ticketCollateralMint"
            docs: [
              "Mint owned by fixed-term market to issue collateral value to a user for",
              "positions that are priced as tickets. The collateral notes are monitored",
              "by the margin program to track value"
            ]
            type: "publicKey"
          },
          {
            name: "tokenCollateralMint"
            docs: [
              "Mint owned by fixed-term market to issue collateral value to a user for",
              "positions that are priced as tokens. The collateral notes are monitored",
              "by the margin program to track value"
            ]
            type: "publicKey"
          },
          {
            name: "underlyingOracle"
            docs: ["oracle that defines the value of the underlying asset"]
            type: "publicKey"
          },
          {
            name: "ticketOracle"
            docs: ["oracle that defines the value of the tickets"]
            type: "publicKey"
          },
          {
            name: "feeVault"
            docs: ["vault for collected market fees"]
            type: "publicKey"
          },
          {
            name: "feeDestination"
            docs: ["where fees can be withdrawn to"]
            type: "publicKey"
          },
          {
            name: "seed"
            docs: ["The user-defined part of the seed that generated this market's PDA"]
            type: {
              array: ["u8", 32]
            }
          },
          {
            name: "bump"
            docs: ["The bump seed value for generating the authority address."]
            type: {
              array: ["u8", 1]
            }
          },
          {
            name: "orderbookPaused"
            docs: ["Is the market taking orders"]
            type: "bool"
          },
          {
            name: "ticketsPaused"
            docs: ["Can tickets be redeemed"]
            type: "bool"
          },
          {
            name: "reserved"
            docs: ["reserved for future use"]
            type: {
              array: ["u8", 28]
            }
          },
          {
            name: "borrowTenor"
            docs: ["Length of time before a borrow is marked as due, in seconds"]
            type: "u64"
          },
          {
            name: "lendTenor"
            docs: ["Length of time before a claim is marked as mature, in seconds"]
            type: "u64"
          },
          {
            name: "originationFee"
            docs: ["assessed on borrows. scaled by origination_fee::FEE_UNIT"]
            type: "u64"
          },
          {
            name: "collectedFees"
            docs: ["amount of fees currently available to be withdrawn by market owner"]
            type: "u64"
          },
          {
            name: "nonce"
            docs: ["Used to generate unique order tags"]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "crankAuthorization"
      docs: ["This authorizes a crank to act on any orderbook within the airspace"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "crank"
            type: "publicKey"
          },
          {
            name: "airspace"
            type: "publicKey"
          },
          {
            name: "market"
            type: "publicKey"
          }
        ]
      }
    },
    {
      name: "marginUser"
      docs: ["An acocunt used to track margin users of the market"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "version"
            docs: ["used to determine if a migration step is needed before user actions are allowed"]
            type: "u8"
          },
          {
            name: "marginAccount"
            docs: ["The margin account used for signing actions"]
            type: "publicKey"
          },
          {
            name: "market"
            docs: ["The `Market` for the market"]
            type: "publicKey"
          },
          {
            name: "claims"
            docs: ["Token account used by the margin program to track the debt"]
            type: "publicKey"
          },
          {
            name: "ticketCollateral"
            docs: [
              "Token account used by the margin program to track the collateral value of positions",
              "which are internal to fixed-term market, such as SplitTicket, ClaimTicket, and open orders.",
              "this does *not* represent underlying tokens or ticket tokens, those are registered independently in margin"
            ]
            type: "publicKey"
          },
          {
            name: "tokenCollateral"
            docs: [
              "Token account used by the margin program to track the collateral value of positions",
              "related to a collateralized value of a token as it rests in the control of the Fixed-Term orderbook",
              "for now this specifically tracks the tokens locked in an open borrow order"
            ]
            type: "publicKey"
          },
          {
            name: "debt"
            docs: [
              "The amount of debt that must be collateralized or repaid",
              "This debt is expressed in terms of the underlying token - not tickets"
            ]
            type: {
              defined: "Debt"
            }
          },
          {
            name: "assets"
            docs: ["Accounting used to track assets in custody of the fixed term market"]
            type: {
              defined: "Assets"
            }
          },
          {
            name: "borrowRollConfig"
            docs: ['Settings for borrow order "auto rolling"']
            type: {
              defined: "BorrowAutoRollConfig"
            }
          },
          {
            name: "lendRollConfig"
            docs: ['Settings for lend order "auto rolling"']
            type: {
              defined: "LendAutoRollConfig"
            }
          }
        ]
      }
    },
    {
      name: "termLoan"
      type: {
        kind: "struct"
        fields: [
          {
            name: "sequenceNumber"
            type: "u64" // should be "u64"
          },
          {
            name: "marginUser"
            docs: ["The user borrower account this term loan is assigned to"]
            type: "publicKey"
          },
          {
            name: "market"
            docs: ["The market where the term loan was created"]
            type: "publicKey"
          },
          {
            name: "payer"
            docs: ["Which account recieves the rent when this PDA is destructed"]
            type: "publicKey"
          },
          {
            name: "orderTag"
            docs: ["The `OrderTag` associated with the creation of this `TermLoan`"]
            type: {
              array: ["u8", 16] // should be ["u8", 16]
            }
          },
          {
            name: "maturationTimestamp"
            docs: ["The time that the term loan must be repaid"]
            type: "u64" // should be "u64"
          },
          {
            name: "balance"
            docs: ["The remaining amount due by the end of the loan term"]
            type: "u64"
          },
          {
            name: "flags"
            docs: ["Any boolean flags for this data type compressed to a single byte"]
            type: "u8" // should be "u8"
          }
        ]
      }
    },
    {
      name: "eventAdapterMetadata"
      type: {
        kind: "struct"
        fields: [
          {
            name: "owner"
            docs: ["Signing authority over this Adapter"]
            type: "publicKey"
          },
          {
            name: "market"
            docs: ["The `Market` this adapter belongs to"]
            type: "publicKey"
          },
          {
            name: "orderbookUser"
            docs: ["The `MarginUser` account this adapter is registered for"]
            type: "publicKey"
          }
        ]
      }
    },
    {
      name: "termDeposit"
      docs: ["A representation of an interest earning deposit, which can be redeemed after reaching maturity"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "owner"
            docs: [
              "The owner of the redeemable tokens",
              "",
              "This is usually a user's margin account, unless the deposit was created directly",
              "with this program."
            ]
            type: "publicKey"
          },
          {
            name: "market"
            docs: ["The relevant market for this deposit"]
            type: "publicKey"
          },
          {
            name: "payer"
            docs: ["Which account recieves the rent when this PDA is destructed"]
            type: "publicKey"
          },
          {
            name: "sequenceNumber"
            docs: [
              "The sequence number for this deposit, which serves as unique identifier for a",
              "particular user's deposits."
            ]
            type: "u64"
          },
          {
            name: "maturesAt"
            docs: ["The timestamp at which this deposit has matured, and can be redeemed"]
            type: "i64"
          },
          {
            name: "amount"
            docs: ["The number of tokens that can be reedeemed at maturity"]
            type: "u64"
          },
          {
            name: "principal"
            docs: [
              "The number tokens originally provided to create this deposit",
              "",
              "This is only accurate when using the auto-stake feature, which saves the original",
              "token amount provided in the loan order."
            ]
            type: "u64"
          }
        ]
      }
    }
  ]
  types: [
    {
      name: "InitializeMarketParams"
      docs: ["Parameters for the initialization of the [Market]"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "versionTag"
            docs: ["Tag information for the `Market` account"]
            type: "u64"
          },
          {
            name: "seed"
            docs: [
              "This seed allows the creation of many separate ticket managers tracking different",
              "parameters, such as staking tenor"
            ]
            type: {
              array: ["u8", 32]
            }
          },
          {
            name: "borrowTenor"
            docs: ["Length of time before a borrow is marked as due, in seconds"]
            type: "u64"
          },
          {
            name: "lendTenor"
            docs: ["Length of time before a claim is marked as mature, in seconds"]
            type: "u64"
          },
          {
            name: "originationFee"
            docs: ["assessed on borrows. scaled by origination_fee::FEE_UNIT"]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "InitializeOrderbookParams"
      docs: ["Parameters necessary for orderbook initialization"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "minBaseOrderSize"
            docs: ["The minimum order size that can be inserted into the orderbook after matching."]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "Debt"
      type: {
        kind: "struct"
        fields: [
          {
            name: "nextNewTermLoanSeqno"
            docs: ["The sequence number for the next term loan to be created"]
            type: "u64"
          },
          {
            name: "nextUnpaidTermLoanSeqno"
            docs: ["The sequence number of the next term loan to be paid"]
            type: "u64"
          },
          {
            name: "nextTermLoanMaturity"
            docs: ["The maturation timestamp of the next term loan that is unpaid"]
            type: "u64" // should be "u64"
          },
          {
            name: "pending"
            docs: [
              "Amount that must be collateralized because there is an open order for it.",
              "Does not accrue interest because the loan has not been received yet."
            ]
            type: "u64"
          },
          {
            name: "committed"
            docs: [
              "Debt that has already been borrowed because the order was matched.",
              "This debt will be due when the loan term ends.",
              "This includes all debt, including past due debt"
            ]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "Assets"
      type: {
        kind: "struct"
        fields: [
          {
            name: "entitledTokens"
            docs: ["tokens to transfer into settlement account"]
            type: "u64"
          },
          {
            name: "entitledTickets"
            docs: ["tickets to transfer into settlement account"]
            type: "u64"
          },
          {
            name: "nextDepositSeqno"
            docs: ["The sequence number for the next deposit"]
            type: "u64"
          },
          {
            name: "nextUnredeemedDepositSeqno"
            docs: ["The sequence number for the oldest deposit that has yet to be redeemed"]
            type: "u64"
          },
          {
            name: "ticketsStaked"
            docs: ["The number of tickets locked up in ClaimTicket or SplitTicket"]
            type: "u64"
          },
          {
            name: "postedQuote"
            docs: [
              "The amount of quote included in all orders posted by the user for both",
              "bids and asks. Since the orderbook tracks base, not quote, this is only",
              "an approximation. This value must always be less than or equal to the",
              "actual posted quote."
            ]
            type: "u64"
          },
          {
            name: "reserved0"
            docs: [
              "reserved data that may be used to determine the size of a user's collateral",
              "pessimistically prepared to persist aggregated values for:",
              "base and quote quantities, separately for bid/ask, on open orders and unsettled fills",
              "2^3 = 8 u64's"
            ]
            type: {
              array: ["u8", 64]
            }
          }
        ]
      }
    },
    {
      name: "BorrowAutoRollConfig"
      type: {
        kind: "struct"
        fields: [
          {
            name: "limitPrice"
            docs: ["the limit price at which orders may be placed by an authority"]
            type: "u64"
          },
          {
            name: "rollTenor"
            docs: ["The borrow roll tenor"]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "LendAutoRollConfig"
      type: {
        kind: "struct"
        fields: [
          {
            name: "limitPrice"
            docs: ["the limit price at which orders may be placed by an authority"]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "RegisterAdapterParams"
      type: {
        kind: "struct"
        fields: [
          {
            name: "numEvents"
            docs: ["Total capacity of the adapter", "Increases rent cost"]
            type: "u32"
          }
        ]
      }
    },
    {
      name: "OrderParams"
      docs: ["Parameters needed for order placement"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "maxTicketQty"
            docs: ["The maximum quantity of tickets to be traded."]
            type: "u64"
          },
          {
            name: "maxUnderlyingTokenQty"
            docs: ["The maximum quantity of underlying token to be traded."]
            type: "u64"
          },
          {
            name: "limitPrice"
            docs: ["The limit price of the order. This value is understood as a 32-bit fixed point number."]
            type: "u64"
          },
          {
            name: "matchLimit"
            docs: ["The maximum number of orderbook postings to match in order to fulfill the order"]
            type: "u64"
          },
          {
            name: "postOnly"
            docs: [
              "The order will not be matched against the orderbook and will be direcly written into it.",
              "",
              "The operation will fail if the order's limit_price crosses the spread."
            ]
            type: "bool"
          },
          {
            name: "postAllowed"
            docs: ["Should the unfilled portion of the order be reposted to the orderbook"]
            type: "bool"
          },
          {
            name: "autoStake"
            docs: ["Should the purchased tickets be automatically staked with the ticket program"]
            type: "bool"
          },
          {
            name: "autoRoll"
            docs: ["Should the resulting `TermLoan` or `TermDeposit` be subject to an auto roll"]
            type: "bool"
          }
        ]
      }
    },
    {
      name: "StakeTicketsParams"
      docs: ["Params needed to stake tickets"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "amount"
            docs: ["number of tickets to stake"]
            type: "u64"
          },
          {
            name: "seed"
            docs: ["uniqueness seed to allow a user to have many deposits"]
            type: "bytes"
          }
        ]
      }
    },
    {
      name: "OrderType"
      type: {
        kind: "enum"
        variants: [
          {
            name: "MarginBorrow"
          },
          {
            name: "MarginLend"
          },
          {
            name: "MarginSellTickets"
          },
          {
            name: "Lend"
          },
          {
            name: "SellTickets"
          }
        ]
      }
    },
    {
      name: "MarketSide"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Borrowing"
          },
          {
            name: "Lending"
          }
        ]
      }
    },
    {
      name: "EventAccounts"
      docs: [
        "These are the additional accounts that need to be provided in the ix",
        "for every event that will be processed.",
        "For a fill, 2-6 accounts need to be appended to remaining_accounts",
        "For an out, 1 account needs to be appended to remaining_accounts"
      ]
      type: {
        kind: "enum"
        variants: [
          {
            name: "Fill"
            fields: [
              {
                defined: "FillAccounts<'info>"
              }
            ]
          },
          {
            name: "Out"
            fields: [
              {
                defined: "OutAccounts<'info>"
              }
            ]
          }
        ]
      }
    },
    {
      name: "LoanAccount"
      type: {
        kind: "enum"
        variants: [
          {
            name: "AutoStake"
            fields: [
              {
                defined: "AnchorAccount<'info,TermDeposit,Mut>"
              }
            ]
          },
          {
            name: "NewDebt"
            fields: [
              {
                defined: "AnchorAccount<'info,TermLoan,Mut>"
              }
            ]
          }
        ]
      }
    },
    {
      name: "PreparedEvent"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Fill"
            fields: [
              {
                defined: "FillAccounts<'info>"
              },
              {
                defined: "FillInfo"
              }
            ]
          },
          {
            name: "Out"
            fields: [
              {
                defined: "OutAccounts<'info>"
              },
              {
                defined: "OutInfo"
              }
            ]
          }
        ]
      }
    },
    {
      name: "OrderbookEvent"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Fill"
            fields: [
              {
                defined: "FillInfo"
              }
            ]
          },
          {
            name: "Out"
            fields: [
              {
                defined: "OutInfo"
              }
            ]
          }
        ]
      }
    }
  ]
  events: [
    {
      name: "MarketInitialized"
      fields: [
        {
          name: "version"
          type: "u64"
          index: false
        },
        {
          name: "address"
          type: "publicKey"
          index: false
        },
        {
          name: "airspace"
          type: "publicKey"
          index: false
        },
        {
          name: "underlyingTokenMint"
          type: "publicKey"
          index: false
        },
        {
          name: "underlyingOracle"
          type: "publicKey"
          index: false
        },
        {
          name: "ticketOracle"
          type: "publicKey"
          index: false
        },
        {
          name: "borrowTenor"
          type: "u64"
          index: false
        },
        {
          name: "lendTenor"
          type: "u64"
          index: false
        }
      ]
    },
    {
      name: "OrderbookInitialized"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "orderbookMarketState"
          type: "publicKey"
          index: false
        },
        {
          name: "eventQueue"
          type: "publicKey"
          index: false
        },
        {
          name: "bids"
          type: "publicKey"
          index: false
        },
        {
          name: "asks"
          type: "publicKey"
          index: false
        },
        {
          name: "minBaseOrderSize"
          type: "u64"
          index: false
        },
        {
          name: "tickSize"
          type: "u64"
          index: false
        }
      ]
    },
    {
      name: "PositionRefreshed"
      fields: [
        {
          name: "marginUser"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "ToggleOrderMatching"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "isOrderbookPaused"
          type: "bool"
          index: false
        }
      ]
    },
    {
      name: "SkippedError"
      fields: [
        {
          name: "message"
          type: "string"
          index: false
        }
      ]
    },
    {
      name: "MarginUserInitialized"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "marginUser"
          type: "publicKey"
          index: false
        },
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "OrderPlaced"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "authority"
          type: "publicKey"
          index: false
        },
        {
          name: "marginUser"
          type: {
            option: "publicKey"
          }
          index: false
        },
        {
          name: "orderTag"
          type: "u128"
          index: false
        },
        {
          name: "orderType"
          type: {
            defined: "OrderType"
          }
          index: false
        },
        {
          name: "orderSummary"
          type: {
            array: ["u8", 48] // should be ["u8", 48]
          }
          index: false
        },
        {
          name: "limitPrice"
          type: "u64"
          index: false
        },
        {
          name: "autoStake"
          type: "bool"
          index: false
        },
        {
          name: "postOnly"
          type: "bool"
          index: false
        },
        {
          name: "postAllowed"
          type: "bool"
          index: false
        },
        {
          name: "autoRoll"
          type: "bool"
          index: false
        }
      ]
    },
    {
      name: "TermLoanCreated"
      fields: [
        {
          name: "termLoan"
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
          name: "orderTag"
          type: "u128"
          index: false
        },
        {
          name: "sequenceNumber"
          type: "u64"
          index: false
        },
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "maturationTimestamp"
          type: "i64"
          index: false
        },
        {
          name: "quoteFilled"
          type: "u64"
          index: false
        },
        {
          name: "baseFilled"
          type: "u64"
          index: false
        },
        {
          name: "flags"
          type: "u8" // should be "u8"
          index: false
        }
      ]
    },
    {
      name: "TermLoanRepay"
      fields: [
        {
          name: "orderbookUser"
          type: "publicKey"
          index: false
        },
        {
          name: "termLoan"
          type: "publicKey"
          index: false
        },
        {
          name: "repaymentAmount"
          type: "u64"
          index: false
        },
        {
          name: "finalBalance"
          type: "u64"
          index: false
        },
        {
          name: "isAutoRoll"
          type: "bool"
          index: false
        }
      ]
    },
    {
      name: "TermLoanFulfilled"
      fields: [
        {
          name: "termLoan"
          type: "publicKey"
          index: false
        },
        {
          name: "orderbookUser"
          type: "publicKey"
          index: false
        },
        {
          name: "borrower"
          type: "publicKey"
          index: false
        },
        {
          name: "repaymentAmount"
          type: "u64"
          index: false
        },
        {
          name: "timestamp"
          type: "i64"
          index: false
        },
        {
          name: "isAutoRoll"
          type: "bool"
          index: false
        }
      ]
    },
    {
      name: "TermDepositCreated"
      fields: [
        {
          name: "termDeposit"
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
          name: "orderTag"
          type: {
            option: "u128"
          }
          index: false
        },
        {
          name: "sequenceNumber"
          type: "u64"
          index: false
        },
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "maturationTimestamp"
          type: "i64"
          index: false
        },
        {
          name: "principal"
          type: "u64"
          index: false
        },
        {
          name: "amount"
          type: "u64"
          index: false
        },
        {
          name: "flags"
          type: "u8" // should be "u8"
          index: false
        }
      ]
    },
    {
      name: "DebtUpdated"
      fields: [
        {
          name: "marginUser"
          type: "publicKey"
          index: false
        },
        {
          name: "totalDebt"
          type: "u64"
          index: false
        },
        {
          name: "nextObligationToRepay"
          type: {
            option: "u64" // should be "u64"
          }
          index: false
        },
        {
          name: "outstandingObligations"
          type: "u64"
          index: false
        },
        {
          name: "isPastDue"
          type: "bool"
          index: false
        }
      ]
    },
    {
      name: "AssetsUpdated"
      fields: [
        {
          name: "marginUser"
          type: "publicKey"
          index: false
        },
        {
          name: "entitledTokens"
          type: "u64"
          index: false
        },
        {
          name: "entitledTickets"
          type: "u64"
          index: false
        },
        {
          name: "collateral"
          type: "u64"
          index: false
        }
      ]
    },
    {
      name: "OrderCancelled"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "authority"
          type: "publicKey"
          index: false
        },
        {
          name: "orderTag"
          type: "u128"
          index: false
        }
      ]
    },
    {
      name: "EventAdapterRegistered"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "owner"
          type: "publicKey"
          index: false
        },
        {
          name: "adapter"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "OrderFilled"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "makerAuthority"
          type: "publicKey"
          index: false
        },
        {
          name: "takerAuthority"
          type: "publicKey"
          index: false
        },
        {
          name: "makerOrderTag"
          type: "u128"
          index: false
        },
        {
          name: "takerOrderTag"
          type: "u128"
          index: false
        },
        {
          name: "orderType"
          type: {
            defined: "OrderType"
          }
          index: false
        },
        {
          name: "sequenceNumber"
          type: "u64"
          index: false
        },
        {
          name: "baseFilled"
          type: "u64"
          index: false
        },
        {
          name: "quoteFilled"
          type: "u64"
          index: false
        },
        {
          name: "fillTimestamp"
          type: "i64"
          index: false
        },
        {
          name: "maturationTimestamp"
          type: "i64"
          index: false
        }
      ]
    },
    {
      name: "OrderRemoved"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "authority"
          type: "publicKey"
          index: false
        },
        {
          name: "orderTag"
          type: "u128"
          index: false
        },
        {
          name: "baseRemoved"
          type: "u64"
          index: false
        },
        {
          name: "quoteRemoved"
          type: "u64"
          index: false
        }
      ]
    },
    {
      name: "TokensExchanged"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "user"
          type: "publicKey"
          index: false
        },
        {
          name: "amount"
          type: "u64"
          index: false
        }
      ]
    },
    {
      name: "DepositRedeemed"
      fields: [
        {
          name: "deposit"
          type: "publicKey"
          index: false
        },
        {
          name: "depositHolder"
          type: "publicKey"
          index: false
        },
        {
          name: "redeemedValue"
          type: "u64"
          index: false
        },
        {
          name: "redeemedTimestamp"
          type: "i64"
          index: false
        }
      ]
    },
    {
      name: "TicketsStaked"
      fields: [
        {
          name: "market"
          type: "publicKey"
          index: false
        },
        {
          name: "ticketHolder"
          type: "publicKey"
          index: false
        },
        {
          name: "amount"
          type: "u64"
          index: false
        }
      ]
    },
    {
      name: "DepositTransferred"
      fields: [
        {
          name: "deposit"
          type: "publicKey"
          index: false
        },
        {
          name: "previousOwner"
          type: "publicKey"
          index: false
        },
        {
          name: "newOwner"
          type: "publicKey"
          index: false
        }
      ]
    }
  ]
  errors: [
    {
      code: 6000
      name: "ArithmeticOverflow"
      msg: "overflow occured on checked_add"
    },
    {
      code: 6001
      name: "ArithmeticUnderflow"
      msg: "underflow occured on checked_sub"
    },
    {
      code: 6002
      name: "FixedPointDivision"
      msg: "bad fixed-point division"
    },
    {
      code: 6003
      name: "DoesNotOwnTicket"
      msg: "owner does not own the ticket"
    },
    {
      code: 6004
      name: "DoesNotOwnEventAdapter"
      msg: "signer does not own the event adapter"
    },
    {
      code: 6005
      name: "DoesNotOwnMarket"
      msg: "this market owner does not own this market"
    },
    {
      code: 6006
      name: "EventQueueFull"
      msg: "queue does not have room for another event"
    },
    {
      code: 6007
      name: "FailedToDeserializeTicket"
      msg: "failed to deserialize the SplitTicket or ClaimTicket"
    },
    {
      code: 6008
      name: "FailedToPushEvent"
      msg: "failed to add event to the queue"
    },
    {
      code: 6009
      name: "ImmatureTicket"
      msg: "ticket is not mature and cannot be claimed"
    },
    {
      code: 6010
      name: "InsufficientSeeds"
      msg: "not enough seeds were provided for the accounts that need to be initialized"
    },
    {
      code: 6011
      name: "InvalidAutoRollConfig"
      msg: "invalid auto roll configuration"
    },
    {
      code: 6012
      name: "InvalidOrderPrice"
      msg: "order price is prohibited"
    },
    {
      code: 6013
      name: "InvalidPosition"
      msg: "this token account is not a valid position for this margin user"
    },
    {
      code: 6014
      name: "InvokeCreateAccount"
      msg: "failed to invoke account creation"
    },
    {
      code: 6015
      name: "IoError"
      msg: "failed to properly serialize or deserialize a data structure"
    },
    {
      code: 6016
      name: "MarketStateNotProgramOwned"
      msg: "this market state account is not owned by the current program"
    },
    {
      code: 6017
      name: "MissingEventAdapter"
      msg: "tried to access a missing adapter account"
    },
    {
      code: 6018
      name: "MissingSplitTicket"
      msg: "tried to access a missing split ticket account"
    },
    {
      code: 6019
      name: "NoEvents"
      msg: "consume_events instruction failed to consume a single event"
    },
    {
      code: 6020
      name: "NoMoreAccounts"
      msg: "expected additional remaining accounts, but there were none"
    },
    {
      code: 6021
      name: "NonZeroDebt"
      msg: "the debt has a non-zero balance"
    },
    {
      code: 6022
      name: "OracleError"
      msg: "there was a problem loading the price oracle"
    },
    {
      code: 6023
      name: "OrderNotFound"
      msg: "id was not found in the user's open orders"
    },
    {
      code: 6024
      name: "OrderbookPaused"
      msg: "Orderbook is not taking orders"
    },
    {
      code: 6025
      name: "OrderRejected"
      msg: "aaob did not match or post the order. either posting is disabled or the order was too small"
    },
    {
      code: 6026
      name: "PriceMissing"
      msg: "price could not be accessed from oracle"
    },
    {
      code: 6027
      name: "TermDepositHasWrongSequenceNumber"
      msg: "expected a term deposit with a different sequence number"
    },
    {
      code: 6028
      name: "TermLoanHasWrongSequenceNumber"
      msg: "expected a term loan with a different sequence number"
    },
    {
      code: 6029
      name: "TicketNotFromManager"
      msg: "claim ticket is not from this manager"
    },
    {
      code: 6030
      name: "TicketSettlementAccountNotRegistered"
      msg: "ticket settlement account is not registered as a position in the margin account"
    },
    {
      code: 6031
      name: "TicketsPaused"
      msg: "tickets are paused"
    },
    {
      code: 6032
      name: "UnauthorizedCaller"
      msg: "this signer is not authorized to place a permissioned order"
    },
    {
      code: 6033
      name: "UnderlyingSettlementAccountNotRegistered"
      msg: "underlying settlement account is not registered as a position in the margin account"
    },
    {
      code: 6034
      name: "UserDoesNotOwnAccount"
      msg: "this user does not own the user account"
    },
    {
      code: 6035
      name: "UserDoesNotOwnAdapter"
      msg: "this adapter does not belong to the user"
    },
    {
      code: 6036
      name: "UserNotInMarket"
      msg: "this user account is not associated with this fixed term market"
    },
    {
      code: 6037
      name: "WrongAdapter"
      msg: "the wrong adapter account was passed to this instruction"
    },
    {
      code: 6038
      name: "WrongAirspace"
      msg: "the market is configured for a different airspace"
    },
    {
      code: 6039
      name: "WrongAirspaceAuthorization"
      msg: "the signer is not authorized to perform this action in the current airspace"
    },
    {
      code: 6040
      name: "WrongAsks"
      msg: "asks account does not belong to this market"
    },
    {
      code: 6041
      name: "WrongBids"
      msg: "bids account does not belong to this market"
    },
    {
      code: 6042
      name: "WrongCrankAuthority"
      msg: "wrong authority for this crank instruction"
    },
    {
      code: 6043
      name: "WrongEventQueue"
      msg: "event queue account does not belong to this market"
    },
    {
      code: 6044
      name: "WrongMarket"
      msg: "adapter does not belong to given market"
    },
    {
      code: 6045
      name: "WrongMarketState"
      msg: "this market state is not associated with this market"
    },
    {
      code: 6046
      name: "WrongTicketManager"
      msg: "wrong TicketManager account provided"
    },
    {
      code: 6047
      name: "WrongClaimAccount"
      msg: "the wrong account was provided for the token account that represents a user's claims"
    },
    {
      code: 6048
      name: "WrongTicketCollateralAccount"
      msg: "the wrong account was provided for the token account that represents a user's collateral"
    },
    {
      code: 6049
      name: "WrongClaimMint"
      msg: "the wrong account was provided for the claims token mint"
    },
    {
      code: 6050
      name: "WrongCollateralMint"
      msg: "the wrong account was provided for the collateral token mint"
    },
    {
      code: 6051
      name: "WrongFeeDestination"
      msg: "wrong fee destination"
    },
    {
      code: 6052
      name: "WrongOracle"
      msg: "wrong oracle address was sent to instruction"
    },
    {
      code: 6053
      name: "WrongMarginUser"
      msg: "wrong margin user account address was sent to instruction"
    },
    {
      code: 6054
      name: "WrongMarginUserAuthority"
      msg: "wrong authority for the margin user account address was sent to instruction"
    },
    {
      code: 6055
      name: "WrongProgramAuthority"
      msg: "incorrect authority account"
    },
    {
      code: 6056
      name: "WrongTicketMint"
      msg: "not the ticket mint for this fixed term market"
    },
    {
      code: 6057
      name: "WrongUnderlyingTokenMint"
      msg: "wrong underlying token mint for this fixed term market"
    },
    {
      code: 6058
      name: "WrongUserAccount"
      msg: "wrong user account address was sent to instruction"
    },
    {
      code: 6059
      name: "WrongVault"
      msg: "wrong vault address was sent to instruction"
    },
    {
      code: 6060
      name: "ZeroDivision"
      msg: "attempted to divide with zero"
    }
  ]
};
