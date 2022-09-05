export type JetBonds = {
  version: "0.1.0";
  name: "jet_bonds";
  constants: [
    {
      name: "BOND_MANAGER";
      type: {
        defined: "&[u8]";
      };
      value: 'b"bond_manager"';
    },
    {
      name: "BOND_TICKET_ACCOUNT";
      type: {
        defined: "&[u8]";
      };
      value: 'b"bond_ticket_account"';
    },
    {
      name: "BOND_TICKET_MINT";
      type: {
        defined: "&[u8]";
      };
      value: 'b"bond_ticket_mint"';
    },
    {
      name: "CLAIM_TICKET";
      type: {
        defined: "&[u8]";
      };
      value: 'b"claim_ticket"';
    },
    {
      name: "EVENT_ADAPTER";
      type: {
        defined: "&[u8]";
      };
      value: 'b"event_adapter"';
    },
    {
      name: "ORDERBOOK_MARKET_STATE";
      type: {
        defined: "&[u8]";
      };
      value: 'b"orderbook_market_state"';
    },
    {
      name: "ORDERBOOK_USER";
      type: {
        defined: "&[u8]";
      };
      value: 'b"orderbook_user"';
    },
    {
      name: "UNDERLYING_TOKEN_VAULT";
      type: {
        defined: "&[u8]";
      };
      value: 'b"underlying_token_vault"';
    },
    {
      name: "CLAIM_NOTES";
      type: {
        defined: "&[u8]";
      };
      value: 'b"user_claims"';
    }
  ];
  instructions: [
    {
      name: "cancelOrder";
      docs: ["Cancels an order on the book"];
      accounts: [
        {
          name: "orderbookUserAccount";
          isMut: true;
          isSigner: false;
          docs: [
            "The account tracking information related to this particular user"
          ];
        },
        {
          name: "user";
          isMut: false;
          isSigner: true;
          docs: ["The signing authority for this user account"];
        },
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market"
          ];
        },
        {
          name: "orderbookMarketState";
          isMut: true;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "bids";
          isMut: true;
          isSigner: false;
        },
        {
          name: "asks";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "orderId";
          type: "u128";
        }
      ];
    },
    {
      name: "consumeEvents";
      docs: ["Crank specific instruction, processes the event queue"];
      accounts: [
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market"
          ];
        },
        {
          name: "orderbookMarketState";
          isMut: true;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "crankMetadata";
          isMut: false;
          isSigner: false;
        },
        {
          name: "crankSigner";
          isMut: false;
          isSigner: true;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
          docs: ["The account paying rent for PDA initialization"];
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "numEvents";
          type: "u32";
        },
        {
          name: "seedBytes";
          type: {
            vec: "bytes";
          };
        }
      ];
    },
    {
      name: "deposit";
      docs: ["Deposit funds into a user account"];
      accounts: [
        {
          name: "orderbookUserAccount";
          isMut: true;
          isSigner: false;
          docs: [
            "The account tracking information related to this particular user"
          ];
        },
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market"
          ];
        },
        {
          name: "userTokenVault";
          isMut: true;
          isSigner: false;
          docs: ["The token vault to deposit tokens from"];
        },
        {
          name: "userTokenVaultAuthority";
          isMut: false;
          isSigner: true;
          docs: ["The signing authority for the user_token_vault"];
        },
        {
          name: "underlyingTokenVault";
          isMut: true;
          isSigner: false;
          docs: ["The token vault holding the underlying token of the bond"];
        },
        {
          name: "bondTicketMint";
          isMut: true;
          isSigner: false;
          docs: ["The minting account for the bond tickets"];
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
          docs: ["SPL token program"];
        }
      ];
      args: [
        {
          name: "amount";
          type: "u64";
        },
        {
          name: "kind";
          type: {
            defined: "AssetKind";
          };
        }
      ];
    },
    {
      name: "initializeOrderbookUser";
      docs: ["Create a new orderbook user account"];
      accounts: [
        {
          name: "orderbookUserAccount";
          isMut: true;
          isSigner: false;
          docs: [
            "The account tracking information related to this particular user"
          ];
        },
        {
          name: "user";
          isMut: false;
          isSigner: true;
          docs: ["The signing authority for this user account"];
        },
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: ["The Boheader account"];
        },
        {
          name: "claims";
          isMut: true;
          isSigner: false;
          docs: [
            "Token account used by the margin program to track the debt",
            "that must be collateralized"
          ];
        },
        {
          name: "claimsMint";
          isMut: false;
          isSigner: false;
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "rent";
          isMut: false;
          isSigner: false;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: "initializeOrderbook";
      docs: ["Initializes a new orderbook"];
      accounts: [
        {
          name: "bondManager";
          isMut: true;
          isSigner: false;
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market"
          ];
        },
        {
          name: "orderbookMarketState";
          isMut: true;
          isSigner: false;
          docs: [
            "Accounts for `agnostic-orderbook`",
            "Should be uninitialized, used for invoking create_account and sent to the agnostic orderbook program"
          ];
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "bids";
          isMut: true;
          isSigner: false;
        },
        {
          name: "asks";
          isMut: true;
          isSigner: false;
        },
        {
          name: "programAuthority";
          isMut: false;
          isSigner: true;
          docs: ["Signing account responsible for changes to the bond market"];
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "params";
          type: {
            defined: "InitializeOrderbookParams";
          };
        }
      ];
    },
    {
      name: "placeOrderAuthorized";
      docs: [
        "Allows authorized signers to place an order on the book without having liquidity in their user account"
      ];
      accounts: [
        {
          name: "baseAccounts";
          accounts: [
            {
              name: "orderbookUserAccount";
              isMut: true;
              isSigner: false;
              docs: [
                "The account tracking information related to this particular user"
              ];
            },
            {
              name: "user";
              isMut: false;
              isSigner: true;
              docs: ["The signing authority for this user account"];
            },
            {
              name: "bondManager";
              isMut: false;
              isSigner: false;
              docs: [
                "The `BondManager` account tracks global information related to this particular bond market"
              ];
            },
            {
              name: "orderbookMarketState";
              isMut: true;
              isSigner: false;
            },
            {
              name: "eventQueue";
              isMut: true;
              isSigner: false;
            },
            {
              name: "bids";
              isMut: true;
              isSigner: false;
            },
            {
              name: "asks";
              isMut: true;
              isSigner: false;
            }
          ];
        },
        {
          name: "claims";
          isMut: true;
          isSigner: false;
          docs: [
            "Token account used by the margin program to track the debt that must be collateralized"
          ];
        },
        {
          name: "claimsMint";
          isMut: true;
          isSigner: false;
          docs: [
            "Token mint used by the margin program to track the debt that must be collateralized"
          ];
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "side";
          type: {
            defined: "OrderSide";
          };
        },
        {
          name: "params";
          type: {
            defined: "OrderParams";
          };
        }
      ];
    },
    {
      name: "placeOrder";
      docs: ["Allows any orderbook user to place an order on the book"];
      accounts: [
        {
          name: "orderbookUserAccount";
          isMut: true;
          isSigner: false;
          docs: [
            "The account tracking information related to this particular user"
          ];
        },
        {
          name: "user";
          isMut: false;
          isSigner: true;
          docs: ["The signing authority for this user account"];
        },
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market"
          ];
        },
        {
          name: "orderbookMarketState";
          isMut: true;
          isSigner: false;
        },
        {
          name: "eventQueue";
          isMut: true;
          isSigner: false;
        },
        {
          name: "bids";
          isMut: true;
          isSigner: false;
        },
        {
          name: "asks";
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: "side";
          type: {
            defined: "OrderSide";
          };
        },
        {
          name: "params";
          type: {
            defined: "OrderParams";
          };
        }
      ];
    },
    {
      name: "repay";
      docs: ["Repay debt on an Obligation"];
      accounts: [
        {
          name: "orderbookUserAccount";
          isMut: false;
          isSigner: false;
          docs: [
            "The account tracking information related to this particular user"
          ];
        },
        {
          name: "obligation";
          isMut: true;
          isSigner: false;
        },
        {
          name: "source";
          isMut: true;
          isSigner: false;
          docs: ["The token account to deposit tokens from"];
        },
        {
          name: "payer";
          isMut: false;
          isSigner: true;
          docs: ["The signing authority for the source_account"];
        },
        {
          name: "underlyingTokenVault";
          isMut: true;
          isSigner: false;
          docs: ["The token vault holding the underlying token of the bond"];
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
          docs: ["SPL token program"];
        }
      ];
      args: [
        {
          name: "amount";
          type: "u64";
        }
      ];
    },
    {
      name: "withdraw";
      docs: ["Withdraw liquidity from the orderbook user account"];
      accounts: [
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market"
          ];
        },
        {
          name: "orderbookUserAccount";
          isMut: true;
          isSigner: false;
          docs: [
            "The account tracking information related to this particular user"
          ];
        },
        {
          name: "user";
          isMut: false;
          isSigner: true;
          docs: ["The signing authority for this user account"];
        },
        {
          name: "userTokenVault";
          isMut: true;
          isSigner: false;
          docs: [
            "The token vault to recieve excess funds, specified by the user"
          ];
        },
        {
          name: "underlyingTokenVault";
          isMut: true;
          isSigner: false;
          docs: ["The vault holding the quote tokens of this bond market"];
        },
        {
          name: "bondTicketMint";
          isMut: true;
          isSigner: false;
          docs: ["The minting account for the bond tickets"];
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
          docs: ["SPL token program"];
        }
      ];
      args: [
        {
          name: "amount";
          type: "u64";
        },
        {
          name: "kind";
          type: {
            defined: "AssetKind";
          };
        }
      ];
    },
    {
      name: "exchangeTokens";
      docs: [
        "Exchange underlying token for bond tickets",
        "WARNING: 1-to-1 rate for tickets, but tickets must be staked for redeption of underlying"
      ];
      accounts: [
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: [
            "The BondManager manages asset tokens for a particular bond duration"
          ];
        },
        {
          name: "underlyingTokenVault";
          isMut: true;
          isSigner: false;
          docs: [
            "The vault stores the tokens of the underlying asset managed by the BondManager"
          ];
        },
        {
          name: "bondTicketMint";
          isMut: true;
          isSigner: false;
          docs: ["The minting account for the bond tickets"];
        },
        {
          name: "userBondTicketVault";
          isMut: true;
          isSigner: false;
          docs: ["The token account to recieve the exchanged bond tickets"];
        },
        {
          name: "userUnderlyingTokenVault";
          isMut: true;
          isSigner: false;
          docs: [
            "The user controlled token account to exchange for bond tickets"
          ];
        },
        {
          name: "userAuthority";
          isMut: false;
          isSigner: true;
          docs: [
            "The signing authority in charge of the user's underlying token vault"
          ];
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
          docs: ["SPL token program"];
        }
      ];
      args: [
        {
          name: "amount";
          type: "u64";
        }
      ];
    },
    {
      name: "initializeBondManager";
      docs: ["Initializes a BondManager for a bond ticket market"];
      accounts: [
        {
          name: "bondManager";
          isMut: true;
          isSigner: false;
          docs: [
            "The `BondManager` manages asset tokens for a particular bond duration"
          ];
        },
        {
          name: "underlyingTokenVault";
          isMut: true;
          isSigner: false;
        },
        {
          name: "underlyingTokenMint";
          isMut: false;
          isSigner: false;
          docs: ["The mint for the assets underlying the bond tickets"];
        },
        {
          name: "bondTicketMint";
          isMut: true;
          isSigner: false;
          docs: ["The minting account for the bond tickets"];
        },
        {
          name: "claims";
          isMut: true;
          isSigner: false;
          docs: [
            "Mints tokens to a margin account to represent debt that must be collateralized"
          ];
        },
        {
          name: "programAuthority";
          isMut: false;
          isSigner: true;
          docs: ["The controlling signer for this program"];
        },
        {
          name: "oracle";
          isMut: false;
          isSigner: false;
          docs: ["The oracle for the underlying asset price"];
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
          docs: ["The account paying rent for PDA initialization"];
        },
        {
          name: "rent";
          isMut: false;
          isSigner: false;
          docs: ["Rent sysvar"];
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
          docs: ["SPL token program"];
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
          docs: ["Solana system program"];
        }
      ];
      args: [
        {
          name: "params";
          type: {
            defined: "InitializeBondManagerParams";
          };
        }
      ];
    },
    {
      name: "mintTickets";
      docs: [
        "Mints bond tickets to a specified user",
        "Only callable from the signing authority specified at BondManager creation"
      ];
      accounts: [
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: [
            "The BondManager manages asset tokens for a particular bond duration"
          ];
        },
        {
          name: "authorityMetadata";
          isMut: false;
          isSigner: false;
          docs: [
            "Metadata account signifying that the calling program is authorized to use this instruction"
          ];
        },
        {
          name: "authority";
          isMut: false;
          isSigner: true;
          docs: [
            "The signing account for market instructions requiring an external authority"
          ];
        },
        {
          name: "bondTicketMint";
          isMut: true;
          isSigner: false;
          docs: ["The minting account for the bond tickets"];
        },
        {
          name: "recipientTokenAccount";
          isMut: true;
          isSigner: false;
          docs: ["The account recieving the minted bond tickets"];
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
          docs: ["SPL token program"];
        }
      ];
      args: [
        {
          name: "amount";
          type: "u64";
        }
      ];
    },
    {
      name: "redeemTicket";
      docs: ["Redeems staked tickets for their underlying value"];
      accounts: [
        {
          name: "ticket";
          isMut: true;
          isSigner: false;
          docs: ["One of either `SplitTicket` or `ClaimTicket` for redemption"];
        },
        {
          name: "ticketHolder";
          isMut: true;
          isSigner: true;
          docs: ["The account that owns the ticket"];
        },
        {
          name: "claimantTokenAccount";
          isMut: true;
          isSigner: false;
          docs: [
            "The token account designated to recieve the assets underlying the claim"
          ];
        },
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: ["The BondManager responsible for the asset"];
        },
        {
          name: "underlyingTokenVault";
          isMut: true;
          isSigner: false;
          docs: [
            "The vault stores the tokens of the underlying asset managed by the BondManager"
          ];
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
          docs: ["SPL token program"];
        }
      ];
      args: [];
    },
    {
      name: "stakeBondTickets";
      docs: ["Stakes bond tickets for later redemption"];
      accounts: [
        {
          name: "claimTicket";
          isMut: true;
          isSigner: false;
          docs: ["A struct used to track maturation and total claimable funds"];
        },
        {
          name: "bondManager";
          isMut: true;
          isSigner: false;
          docs: [
            "The BondManager account tracks bonded assets of a particular duration"
          ];
        },
        {
          name: "ticketHolder";
          isMut: false;
          isSigner: true;
          docs: [
            "The owner of bond tickets that wishes to stake them for a redeemable ticket"
          ];
        },
        {
          name: "bondTicketTokenAccount";
          isMut: true;
          isSigner: false;
          docs: ["The account tracking the ticket_holder's bond tickets"];
        },
        {
          name: "bondTicketMint";
          isMut: true;
          isSigner: false;
          docs: [
            "The mint for the bond tickets for this instruction",
            "A mint is a specific instance of the token program for both the underlying asset and the bond duration"
          ];
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
          docs: ["The payer for account initialization"];
        },
        {
          name: "tokenProgram";
          isMut: false;
          isSigner: false;
          docs: [
            "The global on-chain `TokenProgram` for account authority transfer."
          ];
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
          docs: [
            "The global on-chain `SystemProgram` for program account initialization."
          ];
        }
      ];
      args: [
        {
          name: "params";
          type: {
            defined: "StakeBondTicketsParams";
          };
        }
      ];
    },
    {
      name: "tranferTicketOwnership";
      docs: ["Transfer staked tickets to a new owner"];
      accounts: [
        {
          name: "ticket";
          isMut: true;
          isSigner: false;
          docs: ["The ticket to transfer, either a ClaimTicket or SplitTicket"];
        },
        {
          name: "currentOwner";
          isMut: false;
          isSigner: true;
          docs: ["The current owner of the ticket"];
        }
      ];
      args: [
        {
          name: "newOwner";
          type: "publicKey";
        }
      ];
    },
    {
      name: "registerAdapter";
      docs: ["Register a new EventAdapter for syncing to the orderbook events"];
      accounts: [
        {
          name: "adapterQueue";
          isMut: true;
          isSigner: false;
          docs: ["AdapterEventQueue account owned by outside user or program"];
        },
        {
          name: "bondManager";
          isMut: false;
          isSigner: false;
          docs: ["BondManager for this Adapter"];
        },
        {
          name: "orderbookUser";
          isMut: true;
          isSigner: false;
          docs: ["The OrderbookUser this adapter is registered to"];
        },
        {
          name: "user";
          isMut: false;
          isSigner: true;
          docs: ["The owner of the orderbook_user account"];
        },
        {
          name: "owner";
          isMut: false;
          isSigner: true;
          docs: ["Signing authority over this queue"];
        },
        {
          name: "payer";
          isMut: true;
          isSigner: true;
          docs: ["Payer for the initialization rent of the queue"];
        },
        {
          name: "systemProgram";
          isMut: false;
          isSigner: false;
          docs: ["solana system program"];
        }
      ];
      args: [
        {
          name: "params";
          type: {
            defined: "RegisterAdapterParams";
          };
        }
      ];
    },
    {
      name: "popAdapterEvents";
      docs: [
        "Pop the given number of events off the adapter queue",
        "Event logic is left to the outside program"
      ];
      accounts: [
        {
          name: "adapterQueue";
          isMut: true;
          isSigner: false;
          docs: ["AdapterEventQueue account owned by outside user or program"];
        },
        {
          name: "owner";
          isMut: false;
          isSigner: true;
          docs: ["Signing authority over the AdapterEventQueue"];
        }
      ];
      args: [
        {
          name: "numEvents";
          type: "u32";
        }
      ];
    }
  ];
  accounts: [
    {
      name: "BondManager";
      type: {
        kind: "struct";
        fields: [
          {
            name: "versionTag";
            docs: ["Versioning and tag information"];
            type: "u64";
          },
          {
            name: "programAuthority";
            docs: ["The address allowed to make changes to this program state"];
            type: "publicKey";
          },
          {
            name: "orderbookMarketState";
            docs: ["The market state of the agnostic orderbook"];
            type: "publicKey";
          },
          {
            name: "eventQueue";
            docs: ["The orderbook event queue"];
            type: "publicKey";
          },
          {
            name: "asksSlab";
            docs: ["The orderbook asks byteslab"];
            type: "publicKey";
          },
          {
            name: "bidsSlab";
            docs: ["The orderbook bids byteslab"];
            type: "publicKey";
          },
          {
            name: "underlyingTokenMint";
            docs: [
              "The token mint for the underlying asset of the bond tickets"
            ];
            type: "publicKey";
          },
          {
            name: "underlyingTokenVault";
            docs: [
              "Token account storing the underlying asset accounted for by this ticket program"
            ];
            type: "publicKey";
          },
          {
            name: "bondTicketMint";
            docs: ["The token mint for the bond tickets"];
            type: "publicKey";
          },
          {
            name: "claimsMint";
            docs: [
              "Mint owned by bonds to issue claims against a user.",
              "These claim notes are monitored by margin to ensure claims are repaid."
            ];
            type: "publicKey";
          },
          {
            name: "oracle";
            docs: ["oracle that defines the value of the underlying asset"];
            type: "publicKey";
          },
          {
            name: "seed";
            docs: [
              "The user-defined part of the seed that generated this bond manager's PDA"
            ];
            type: {
              array: ["u8", 8];
            };
          },
          {
            name: "bump";
            docs: ["The bump seed value for generating the authority address."];
            type: {
              array: ["u8", 1];
            };
          },
          {
            name: "conversionFactor";
            docs: [
              "The number of decimals added or subtracted to the tickets staked when minting a `ClaimTicket`"
            ];
            type: "i8";
          },
          {
            name: "reserved";
            docs: ["reserved for future use"];
            type: {
              array: ["u8", 30];
            };
          },
          {
            name: "duration";
            docs: [
              "Units added to the initial stake timestamp to determine claim maturity"
            ];
            type: "i64";
          }
        ];
      };
    },
    {
      name: "OrderbookUser";
      docs: [
        "The orderbook user account tracks data about a user's state within the Bonds Orderbook"
      ];
      type: {
        kind: "struct";
        fields: [
          {
            name: "user";
            docs: ["The pubkey of the user. Used to verfiy withdraws, etc."];
            type: "publicKey";
          },
          {
            name: "bondManager";
            docs: [
              "The pubkey pointing to the BondMarket account tracking this user"
            ];
            type: "publicKey";
          },
          {
            name: "eventAdapter";
            docs: ["The address of the registered event adapter for this user"];
            type: "publicKey";
          },
          {
            name: "bondTicketsStored";
            docs: [
              "The quanitity of base token the user may allocate to orders or withdraws",
              "For the bonds program, this represents the bond tickets"
            ];
            type: "u64";
          },
          {
            name: "underlyingTokenStored";
            docs: [
              "The quantity of quote token the user may allocate to orders or withdraws",
              "For the bonds program, this represents the asset redeeemed for staking bond tickets"
            ];
            type: "u64";
          },
          {
            name: "outstandingObligations";
            docs: [
              "total number of outstanding obligations with committed debt"
            ];
            type: "u64";
          },
          {
            name: "debt";
            docs: [
              "The amount of debt that must be collateralized or repaid",
              "This debt is expressed in terms of the underlying token - not bond tickets"
            ];
            type: {
              defined: "Debt";
            };
          },
          {
            name: "claims";
            docs: [
              "Token account used by the margin program to track the debt"
            ];
            type: "publicKey";
          },
          {
            name: "nonce";
            docs: [
              "This nonce is used to generate unique order tags",
              "Instantiated as `0` and incremented with each order"
            ];
            type: "u64";
          }
        ];
      };
    },
    {
      name: "ClaimTicket";
      type: {
        kind: "struct";
        fields: [
          {
            name: "owner";
            docs: ["The account registered as owner of this claim"];
            type: "publicKey";
          },
          {
            name: "bondManager";
            docs: [
              "The `TicketManager` this claim ticket was established under",
              "Determines the asset this ticket will be redeemed for"
            ];
            type: "publicKey";
          },
          {
            name: "maturationTimestamp";
            docs: [
              "The slot after which this claim can be redeemed for the underlying value"
            ];
            type: "i64";
          },
          {
            name: "redeemable";
            docs: ["The number of tokens this claim  is redeemable for"];
            type: "u64";
          }
        ];
      };
    },
    {
      name: "SplitTicket";
      type: {
        kind: "struct";
        fields: [
          {
            name: "owner";
            docs: ["The account registered as owner of this claim"];
            type: "publicKey";
          },
          {
            name: "bondManager";
            docs: [
              "The `TicketManager` this claim ticket was established under",
              "Determines the asset this ticket will be redeemed for"
            ];
            type: "publicKey";
          },
          {
            name: "orderTag";
            docs: [
              "The `OrderTag` associated with the creation of this struct"
            ];
            type: {
              array: ["u8", 16];
            };
          },
          {
            name: "struckTimestamp";
            docs: ["The time slot during which the ticket was struck"];
            type: "i64";
          },
          {
            name: "maturationTimestamp";
            docs: [
              "The slot after which this claim can be redeemed for the underlying value"
            ];
            type: "i64";
          },
          {
            name: "principal";
            docs: [
              "The total number of principal tokens the bond was struck for"
            ];
            type: "u64";
          },
          {
            name: "interest";
            docs: [
              "The total number of interest tokens struck for this bond",
              "same underlying asset as the principal token"
            ];
            type: "u64";
          }
        ];
      };
    },
    {
      name: "EventAdapterMetadata";
      type: {
        kind: "struct";
        fields: [
          {
            name: "owner";
            docs: ["Signing authority over this Adapter"];
            type: "publicKey";
          },
          {
            name: "manager";
            docs: ["The `BondManager` this adapter belongs to"];
            type: "publicKey";
          },
          {
            name: "orderbookUser";
            docs: [
              "The `OrderbookUser` account this adapter is registered for"
            ];
            type: "publicKey";
          }
        ];
      };
    },
    {
      name: "Obligation";
      type: {
        kind: "struct";
        fields: [
          {
            name: "orderbookUserAccount";
            docs: ["The user (margin account) this obligation is owed by"];
            type: "publicKey";
          },
          {
            name: "bondManager";
            docs: ["The bond manager where the obligation was created"];
            type: "publicKey";
          },
          {
            name: "orderTag";
            docs: [
              "The `OrderTag` associated with the creation of this `Obligation`"
            ];
            type: {
              array: ["u8", 16];
            };
          },
          {
            name: "maturationTimestamp";
            docs: ["The time that the obligation must be repaid"];
            type: "i64";
          },
          {
            name: "balance";
            docs: ["The remaining amount due by the end of the loan term"];
            type: "u64";
          },
          {
            name: "flags";
            docs: [
              "Any boolean flags for this data type compressed to a single byte"
            ];
            type: "u8";
          }
        ];
      };
    }
  ];
  types: [
    {
      name: "RegisterAdapterParams";
      type: {
        kind: "struct";
        fields: [
          {
            name: "numEvents";
            docs: ["Total capacity of the adapter", "Increases rent cost"];
            type: "u32";
          }
        ];
      };
    },
    {
      name: "InitializeOrderbookParams";
      type: {
        kind: "struct";
        fields: [
          {
            name: "minBaseOrderSize";
            docs: [
              "The minimum order size that can be inserted into the orderbook after matching."
            ];
            type: "u64";
          }
        ];
      };
    },
    {
      name: "InitializeBondManagerSeeds";
      type: {
        kind: "struct";
        fields: [
          {
            name: "uniquenessSeed";
            docs: [
              "This seed allows the creation of many separate ticket managers tracking different",
              "parameters, such as staking duration"
            ];
            type: "bytes";
          }
        ];
      };
    },
    {
      name: "InitializeBondManagerParams";
      type: {
        kind: "struct";
        fields: [
          {
            name: "versionTag";
            docs: ["Tag information for the `BondManager` account"];
            type: "u64";
          },
          {
            name: "seed";
            docs: [
              "This seed allows the creation of many separate ticket managers tracking different",
              "parameters, such as staking duration"
            ];
            type: "u64";
          },
          {
            name: "duration";
            docs: [
              "Units added to the initial stake timestamp to determine claim maturity"
            ];
            type: "i64";
          },
          {
            name: "conversionFactor";
            docs: [
              "The number of decimals added or subtracted to the tickets staked when minting a `ClaimTicket`"
            ];
            type: "i8";
          }
        ];
      };
    },
    {
      name: "StakeBondTicketsParams";
      type: {
        kind: "struct";
        fields: [
          {
            name: "amount";
            docs: ["number of tickets to stake"];
            type: "u64";
          },
          {
            name: "ticketSeed";
            docs: [
              "uniqueness seed to allow a user to have many `ClaimTicket`s"
            ];
            type: "bytes";
          }
        ];
      };
    },
    {
      name: "Debt";
      type: {
        kind: "struct";
        fields: [
          {
            name: "pending";
            docs: [
              "Amount that must be collateralized because there is an open order for it.",
              "Does not accrue interest because the loan has not been received yet."
            ];
            type: "u64";
          },
          {
            name: "committed";
            docs: [
              "Debt that has already been borrowed because the order was matched.",
              "This debt will be due when the loan term ends.",
              "Some of this debt may actually be due already, but a crank has not yet been marked it as due."
            ];
            type: "u64";
          },
          {
            name: "pastDue";
            docs: [
              "Amount of debt that has already been discovered and marked as being due",
              "This is not guaranteed to be comprehensive. It may not include some",
              "obligations that have not yet been marked due."
            ];
            type: "u64";
          }
        ];
      };
    },
    {
      name: "OrderParams";
      type: {
        kind: "struct";
        fields: [
          {
            name: "maxBondTicketQty";
            docs: ["The maximum quantity of bond tickets to be traded."];
            type: "u64";
          },
          {
            name: "maxUnderlyingTokenQty";
            docs: ["The maximum quantity of underlying token to be traded."];
            type: "u64";
          },
          {
            name: "limitPrice";
            docs: [
              "The limit price of the order. This value is understood as a 32-bit fixed point number."
            ];
            type: "u64";
          },
          {
            name: "matchLimit";
            docs: [
              "The maximum number of orderbook postings to match in order to fulfill the order"
            ];
            type: "u64";
          },
          {
            name: "postOnly";
            docs: [
              "The order will not be matched against the orderbook and will be direcly written into it.",
              "",
              "The operation will fail if the order's limit_price crosses the spread."
            ];
            type: "bool";
          },
          {
            name: "postAllowed";
            docs: [
              "Should the unfilled portion of the order be reposted to the orderbook"
            ];
            type: "bool";
          },
          {
            name: "autoStake";
            docs: [
              "Should the purchased tickets be automatically staked with the ticket program"
            ];
            type: "bool";
          }
        ];
      };
    },
    {
      name: "OrderSide";
      type: {
        kind: "enum";
        variants: [
          {
            name: "Lend";
          },
          {
            name: "Borrow";
          }
        ];
      };
    },
    {
      name: "AssetKind";
      type: {
        kind: "enum";
        variants: [
          {
            name: "UnderlyingToken";
          },
          {
            name: "BondTicket";
          }
        ];
      };
    }
  ];
  errors: [
    {
      code: 6000;
      name: "ArithmeticOverflow";
      msg: "overflow occured on checked_add";
    },
    {
      code: 6001;
      name: "ArithmeticUnderflow";
      msg: "underflow occured on checked_sub";
    },
    {
      code: 6002;
      name: "DoesNotOwnTicket";
      msg: "owner does not own the ticket";
    },
    {
      code: 6003;
      name: "DoesNotOwnEventAdapter";
      msg: "signer does not own the event adapter";
    },
    {
      code: 6004;
      name: "EventQueueFull";
      msg: "queue does not have room for another event";
    },
    {
      code: 6005;
      name: "FailedToDeserializeTicket";
      msg: "failed to deserialize the SplitTicket or ClaimTicket";
    },
    {
      code: 6006;
      name: "ImmatureBond";
      msg: "bond is not mature and cannot be claimed";
    },
    {
      code: 6007;
      name: "InsufficientSeeds";
      msg: "not enough seeds were provided for the accounts that need to be initialized";
    },
    {
      code: 6008;
      name: "InvalidEvent";
      msg: "the wrong event type was unwrapped\\nthis condition should be impossible, and does not result from invalid input";
    },
    {
      code: 6009;
      name: "InvokeCreateAccount";
      msg: "failed to invoke account creation";
    },
    {
      code: 6010;
      name: "IoError";
      msg: "failed to properly serialize or deserialize a data structure";
    },
    {
      code: 6011;
      name: "MarketStateNotProgramOwned";
      msg: "this market state account is not owned by the current program";
    },
    {
      code: 6012;
      name: "MissingEventAdapter";
      msg: "tried to access a missing adapter account";
    },
    {
      code: 6013;
      name: "NoEvents";
      msg: "consume_events instruction failed to consume a single event";
    },
    {
      code: 6014;
      name: "OracleError";
      msg: "there was a problem loading the price oracle";
    },
    {
      code: 6015;
      name: "OrderNotFound";
      msg: "id was not found in the user's open orders";
    },
    {
      code: 6016;
      name: "PriceMissing";
      msg: "price could not be accessed from oracle";
    },
    {
      code: 6017;
      name: "TicketNotFromManager";
      msg: "claim ticket is not from this manager";
    },
    {
      code: 6018;
      name: "UnauthorizedCaller";
      msg: "this signer is not authorized to place a permissioned order";
    },
    {
      code: 6019;
      name: "UserDoesNotOwnAccount";
      msg: "this user does not own the user account";
    },
    {
      code: 6020;
      name: "UserDoesNotOwnAdapter";
      msg: "this adapter does not belong to the user";
    },
    {
      code: 6021;
      name: "UserNotInMarket";
      msg: "this user account is not associated with this bond market";
    },
    {
      code: 6022;
      name: "WrongBondManager";
      msg: "adapter does not belong to given bond manager";
    },
    {
      code: 6023;
      name: "WrongCrankAuthority";
      msg: "wrong authority for this crank instruction";
    },
    {
      code: 6024;
      name: "WrongMarketState";
      msg: "this market state is not associated with this market";
    },
    {
      code: 6025;
      name: "WrongTicketManager";
      msg: "wrong TicketManager account provided";
    },
    {
      code: 6026;
      name: "DoesNotOwnMarket";
      msg: "this market owner does not own this market";
    },
    {
      code: 6027;
      name: "WrongClaimAccount";
      msg: "the wrong account was provided for the token account that represents a user's claims";
    },
    {
      code: 6028;
      name: "WrongClaimMint";
      msg: "the wrong account was provided for the claims token mint";
    },
    {
      code: 6029;
      name: "WrongOracle";
      msg: "wrong oracle address was sent to instruction";
    },
    {
      code: 6030;
      name: "WrongOrderbookUser";
      msg: "wrong orderbook user account address was sent to instruction";
    },
    {
      code: 6031;
      name: "WrongProgramAuthority";
      msg: "incorrect authority account";
    },
    {
      code: 6032;
      name: "WrongTicketMint";
      msg: "not the ticket mint for this bond market";
    },
    {
      code: 6033;
      name: "WrongVault";
      msg: "wrong vault address was sent to instruction";
    },
    {
      code: 6034;
      name: "ZeroDivision";
      msg: "attempted to divide with zero";
    }
  ];
};

export const IDL: JetBonds = {
  version: "0.1.0",
  name: "jet_bonds",
  constants: [
    {
      name: "BOND_MANAGER",
      type: {
        defined: "&[u8]",
      },
      value: 'b"bond_manager"',
    },
    {
      name: "BOND_TICKET_ACCOUNT",
      type: {
        defined: "&[u8]",
      },
      value: 'b"bond_ticket_account"',
    },
    {
      name: "BOND_TICKET_MINT",
      type: {
        defined: "&[u8]",
      },
      value: 'b"bond_ticket_mint"',
    },
    {
      name: "CLAIM_TICKET",
      type: {
        defined: "&[u8]",
      },
      value: 'b"claim_ticket"',
    },
    {
      name: "EVENT_ADAPTER",
      type: {
        defined: "&[u8]",
      },
      value: 'b"event_adapter"',
    },
    {
      name: "ORDERBOOK_MARKET_STATE",
      type: {
        defined: "&[u8]",
      },
      value: 'b"orderbook_market_state"',
    },
    {
      name: "ORDERBOOK_USER",
      type: {
        defined: "&[u8]",
      },
      value: 'b"orderbook_user"',
    },
    {
      name: "UNDERLYING_TOKEN_VAULT",
      type: {
        defined: "&[u8]",
      },
      value: 'b"underlying_token_vault"',
    },
    {
      name: "CLAIM_NOTES",
      type: {
        defined: "&[u8]",
      },
      value: 'b"user_claims"',
    },
  ],
  instructions: [
    {
      name: "cancelOrder",
      docs: ["Cancels an order on the book"],
      accounts: [
        {
          name: "orderbookUserAccount",
          isMut: true,
          isSigner: false,
          docs: [
            "The account tracking information related to this particular user",
          ],
        },
        {
          name: "user",
          isMut: false,
          isSigner: true,
          docs: ["The signing authority for this user account"],
        },
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market",
          ],
        },
        {
          name: "orderbookMarketState",
          isMut: true,
          isSigner: false,
        },
        {
          name: "eventQueue",
          isMut: true,
          isSigner: false,
        },
        {
          name: "bids",
          isMut: true,
          isSigner: false,
        },
        {
          name: "asks",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "orderId",
          type: "u128",
        },
      ],
    },
    {
      name: "consumeEvents",
      docs: ["Crank specific instruction, processes the event queue"],
      accounts: [
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market",
          ],
        },
        {
          name: "orderbookMarketState",
          isMut: true,
          isSigner: false,
        },
        {
          name: "eventQueue",
          isMut: true,
          isSigner: false,
        },
        {
          name: "crankMetadata",
          isMut: false,
          isSigner: false,
        },
        {
          name: "crankSigner",
          isMut: false,
          isSigner: true,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The account paying rent for PDA initialization"],
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "numEvents",
          type: "u32",
        },
        {
          name: "seedBytes",
          type: {
            vec: "bytes",
          },
        },
      ],
    },
    {
      name: "deposit",
      docs: ["Deposit funds into a user account"],
      accounts: [
        {
          name: "orderbookUserAccount",
          isMut: true,
          isSigner: false,
          docs: [
            "The account tracking information related to this particular user",
          ],
        },
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market",
          ],
        },
        {
          name: "userTokenVault",
          isMut: true,
          isSigner: false,
          docs: ["The token vault to deposit tokens from"],
        },
        {
          name: "userTokenVaultAuthority",
          isMut: false,
          isSigner: true,
          docs: ["The signing authority for the user_token_vault"],
        },
        {
          name: "underlyingTokenVault",
          isMut: true,
          isSigner: false,
          docs: ["The token vault holding the underlying token of the bond"],
        },
        {
          name: "bondTicketMint",
          isMut: true,
          isSigner: false,
          docs: ["The minting account for the bond tickets"],
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
          docs: ["SPL token program"],
        },
      ],
      args: [
        {
          name: "amount",
          type: "u64",
        },
        {
          name: "kind",
          type: {
            defined: "AssetKind",
          },
        },
      ],
    },
    {
      name: "initializeOrderbookUser",
      docs: ["Create a new orderbook user account"],
      accounts: [
        {
          name: "orderbookUserAccount",
          isMut: true,
          isSigner: false,
          docs: [
            "The account tracking information related to this particular user",
          ],
        },
        {
          name: "user",
          isMut: false,
          isSigner: true,
          docs: ["The signing authority for this user account"],
        },
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: ["The Boheader account"],
        },
        {
          name: "claims",
          isMut: true,
          isSigner: false,
          docs: [
            "Token account used by the margin program to track the debt",
            "that must be collateralized",
          ],
        },
        {
          name: "claimsMint",
          isMut: false,
          isSigner: false,
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "rent",
          isMut: false,
          isSigner: false,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: "initializeOrderbook",
      docs: ["Initializes a new orderbook"],
      accounts: [
        {
          name: "bondManager",
          isMut: true,
          isSigner: false,
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market",
          ],
        },
        {
          name: "orderbookMarketState",
          isMut: true,
          isSigner: false,
          docs: [
            "Accounts for `agnostic-orderbook`",
            "Should be uninitialized, used for invoking create_account and sent to the agnostic orderbook program",
          ],
        },
        {
          name: "eventQueue",
          isMut: true,
          isSigner: false,
        },
        {
          name: "bids",
          isMut: true,
          isSigner: false,
        },
        {
          name: "asks",
          isMut: true,
          isSigner: false,
        },
        {
          name: "programAuthority",
          isMut: false,
          isSigner: true,
          docs: ["Signing account responsible for changes to the bond market"],
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "params",
          type: {
            defined: "InitializeOrderbookParams",
          },
        },
      ],
    },
    {
      name: "placeOrderAuthorized",
      docs: [
        "Allows authorized signers to place an order on the book without having liquidity in their user account",
      ],
      accounts: [
        {
          name: "baseAccounts",
          accounts: [
            {
              name: "orderbookUserAccount",
              isMut: true,
              isSigner: false,
              docs: [
                "The account tracking information related to this particular user",
              ],
            },
            {
              name: "user",
              isMut: false,
              isSigner: true,
              docs: ["The signing authority for this user account"],
            },
            {
              name: "bondManager",
              isMut: false,
              isSigner: false,
              docs: [
                "The `BondManager` account tracks global information related to this particular bond market",
              ],
            },
            {
              name: "orderbookMarketState",
              isMut: true,
              isSigner: false,
            },
            {
              name: "eventQueue",
              isMut: true,
              isSigner: false,
            },
            {
              name: "bids",
              isMut: true,
              isSigner: false,
            },
            {
              name: "asks",
              isMut: true,
              isSigner: false,
            },
          ],
        },
        {
          name: "claims",
          isMut: true,
          isSigner: false,
          docs: [
            "Token account used by the margin program to track the debt that must be collateralized",
          ],
        },
        {
          name: "claimsMint",
          isMut: true,
          isSigner: false,
          docs: [
            "Token mint used by the margin program to track the debt that must be collateralized",
          ],
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "side",
          type: {
            defined: "OrderSide",
          },
        },
        {
          name: "params",
          type: {
            defined: "OrderParams",
          },
        },
      ],
    },
    {
      name: "placeOrder",
      docs: ["Allows any orderbook user to place an order on the book"],
      accounts: [
        {
          name: "orderbookUserAccount",
          isMut: true,
          isSigner: false,
          docs: [
            "The account tracking information related to this particular user",
          ],
        },
        {
          name: "user",
          isMut: false,
          isSigner: true,
          docs: ["The signing authority for this user account"],
        },
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market",
          ],
        },
        {
          name: "orderbookMarketState",
          isMut: true,
          isSigner: false,
        },
        {
          name: "eventQueue",
          isMut: true,
          isSigner: false,
        },
        {
          name: "bids",
          isMut: true,
          isSigner: false,
        },
        {
          name: "asks",
          isMut: true,
          isSigner: false,
        },
      ],
      args: [
        {
          name: "side",
          type: {
            defined: "OrderSide",
          },
        },
        {
          name: "params",
          type: {
            defined: "OrderParams",
          },
        },
      ],
    },
    {
      name: "repay",
      docs: ["Repay debt on an Obligation"],
      accounts: [
        {
          name: "orderbookUserAccount",
          isMut: false,
          isSigner: false,
          docs: [
            "The account tracking information related to this particular user",
          ],
        },
        {
          name: "obligation",
          isMut: true,
          isSigner: false,
        },
        {
          name: "source",
          isMut: true,
          isSigner: false,
          docs: ["The token account to deposit tokens from"],
        },
        {
          name: "payer",
          isMut: false,
          isSigner: true,
          docs: ["The signing authority for the source_account"],
        },
        {
          name: "underlyingTokenVault",
          isMut: true,
          isSigner: false,
          docs: ["The token vault holding the underlying token of the bond"],
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
          docs: ["SPL token program"],
        },
      ],
      args: [
        {
          name: "amount",
          type: "u64",
        },
      ],
    },
    {
      name: "withdraw",
      docs: ["Withdraw liquidity from the orderbook user account"],
      accounts: [
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: [
            "The `BondManager` account tracks global information related to this particular bond market",
          ],
        },
        {
          name: "orderbookUserAccount",
          isMut: true,
          isSigner: false,
          docs: [
            "The account tracking information related to this particular user",
          ],
        },
        {
          name: "user",
          isMut: false,
          isSigner: true,
          docs: ["The signing authority for this user account"],
        },
        {
          name: "userTokenVault",
          isMut: true,
          isSigner: false,
          docs: [
            "The token vault to recieve excess funds, specified by the user",
          ],
        },
        {
          name: "underlyingTokenVault",
          isMut: true,
          isSigner: false,
          docs: ["The vault holding the quote tokens of this bond market"],
        },
        {
          name: "bondTicketMint",
          isMut: true,
          isSigner: false,
          docs: ["The minting account for the bond tickets"],
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
          docs: ["SPL token program"],
        },
      ],
      args: [
        {
          name: "amount",
          type: "u64",
        },
        {
          name: "kind",
          type: {
            defined: "AssetKind",
          },
        },
      ],
    },
    {
      name: "exchangeTokens",
      docs: [
        "Exchange underlying token for bond tickets",
        "WARNING: 1-to-1 rate for tickets, but tickets must be staked for redeption of underlying",
      ],
      accounts: [
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: [
            "The BondManager manages asset tokens for a particular bond duration",
          ],
        },
        {
          name: "underlyingTokenVault",
          isMut: true,
          isSigner: false,
          docs: [
            "The vault stores the tokens of the underlying asset managed by the BondManager",
          ],
        },
        {
          name: "bondTicketMint",
          isMut: true,
          isSigner: false,
          docs: ["The minting account for the bond tickets"],
        },
        {
          name: "userBondTicketVault",
          isMut: true,
          isSigner: false,
          docs: ["The token account to recieve the exchanged bond tickets"],
        },
        {
          name: "userUnderlyingTokenVault",
          isMut: true,
          isSigner: false,
          docs: [
            "The user controlled token account to exchange for bond tickets",
          ],
        },
        {
          name: "userAuthority",
          isMut: false,
          isSigner: true,
          docs: [
            "The signing authority in charge of the user's underlying token vault",
          ],
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
          docs: ["SPL token program"],
        },
      ],
      args: [
        {
          name: "amount",
          type: "u64",
        },
      ],
    },
    {
      name: "initializeBondManager",
      docs: ["Initializes a BondManager for a bond ticket market"],
      accounts: [
        {
          name: "bondManager",
          isMut: true,
          isSigner: false,
          docs: [
            "The `BondManager` manages asset tokens for a particular bond duration",
          ],
        },
        {
          name: "underlyingTokenVault",
          isMut: true,
          isSigner: false,
        },
        {
          name: "underlyingTokenMint",
          isMut: false,
          isSigner: false,
          docs: ["The mint for the assets underlying the bond tickets"],
        },
        {
          name: "bondTicketMint",
          isMut: true,
          isSigner: false,
          docs: ["The minting account for the bond tickets"],
        },
        {
          name: "claims",
          isMut: true,
          isSigner: false,
          docs: [
            "Mints tokens to a margin account to represent debt that must be collateralized",
          ],
        },
        {
          name: "programAuthority",
          isMut: false,
          isSigner: true,
          docs: ["The controlling signer for this program"],
        },
        {
          name: "oracle",
          isMut: false,
          isSigner: false,
          docs: ["The oracle for the underlying asset price"],
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The account paying rent for PDA initialization"],
        },
        {
          name: "rent",
          isMut: false,
          isSigner: false,
          docs: ["Rent sysvar"],
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
          docs: ["SPL token program"],
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
          docs: ["Solana system program"],
        },
      ],
      args: [
        {
          name: "params",
          type: {
            defined: "InitializeBondManagerParams",
          },
        },
      ],
    },
    {
      name: "mintTickets",
      docs: [
        "Mints bond tickets to a specified user",
        "Only callable from the signing authority specified at BondManager creation",
      ],
      accounts: [
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: [
            "The BondManager manages asset tokens for a particular bond duration",
          ],
        },
        {
          name: "authorityMetadata",
          isMut: false,
          isSigner: false,
          docs: [
            "Metadata account signifying that the calling program is authorized to use this instruction",
          ],
        },
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: [
            "The signing account for market instructions requiring an external authority",
          ],
        },
        {
          name: "bondTicketMint",
          isMut: true,
          isSigner: false,
          docs: ["The minting account for the bond tickets"],
        },
        {
          name: "recipientTokenAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account recieving the minted bond tickets"],
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
          docs: ["SPL token program"],
        },
      ],
      args: [
        {
          name: "amount",
          type: "u64",
        },
      ],
    },
    {
      name: "redeemTicket",
      docs: ["Redeems staked tickets for their underlying value"],
      accounts: [
        {
          name: "ticket",
          isMut: true,
          isSigner: false,
          docs: ["One of either `SplitTicket` or `ClaimTicket` for redemption"],
        },
        {
          name: "ticketHolder",
          isMut: true,
          isSigner: true,
          docs: ["The account that owns the ticket"],
        },
        {
          name: "claimantTokenAccount",
          isMut: true,
          isSigner: false,
          docs: [
            "The token account designated to recieve the assets underlying the claim",
          ],
        },
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: ["The BondManager responsible for the asset"],
        },
        {
          name: "underlyingTokenVault",
          isMut: true,
          isSigner: false,
          docs: [
            "The vault stores the tokens of the underlying asset managed by the BondManager",
          ],
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
          docs: ["SPL token program"],
        },
      ],
      args: [],
    },
    {
      name: "stakeBondTickets",
      docs: ["Stakes bond tickets for later redemption"],
      accounts: [
        {
          name: "claimTicket",
          isMut: true,
          isSigner: false,
          docs: ["A struct used to track maturation and total claimable funds"],
        },
        {
          name: "bondManager",
          isMut: true,
          isSigner: false,
          docs: [
            "The BondManager account tracks bonded assets of a particular duration",
          ],
        },
        {
          name: "ticketHolder",
          isMut: false,
          isSigner: true,
          docs: [
            "The owner of bond tickets that wishes to stake them for a redeemable ticket",
          ],
        },
        {
          name: "bondTicketTokenAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account tracking the ticket_holder's bond tickets"],
        },
        {
          name: "bondTicketMint",
          isMut: true,
          isSigner: false,
          docs: [
            "The mint for the bond tickets for this instruction",
            "A mint is a specific instance of the token program for both the underlying asset and the bond duration",
          ],
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The payer for account initialization"],
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false,
          docs: [
            "The global on-chain `TokenProgram` for account authority transfer.",
          ],
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
          docs: [
            "The global on-chain `SystemProgram` for program account initialization.",
          ],
        },
      ],
      args: [
        {
          name: "params",
          type: {
            defined: "StakeBondTicketsParams",
          },
        },
      ],
    },
    {
      name: "tranferTicketOwnership",
      docs: ["Transfer staked tickets to a new owner"],
      accounts: [
        {
          name: "ticket",
          isMut: true,
          isSigner: false,
          docs: ["The ticket to transfer, either a ClaimTicket or SplitTicket"],
        },
        {
          name: "currentOwner",
          isMut: false,
          isSigner: true,
          docs: ["The current owner of the ticket"],
        },
      ],
      args: [
        {
          name: "newOwner",
          type: "publicKey",
        },
      ],
    },
    {
      name: "registerAdapter",
      docs: ["Register a new EventAdapter for syncing to the orderbook events"],
      accounts: [
        {
          name: "adapterQueue",
          isMut: true,
          isSigner: false,
          docs: ["AdapterEventQueue account owned by outside user or program"],
        },
        {
          name: "bondManager",
          isMut: false,
          isSigner: false,
          docs: ["BondManager for this Adapter"],
        },
        {
          name: "orderbookUser",
          isMut: true,
          isSigner: false,
          docs: ["The OrderbookUser this adapter is registered to"],
        },
        {
          name: "user",
          isMut: false,
          isSigner: true,
          docs: ["The owner of the orderbook_user account"],
        },
        {
          name: "owner",
          isMut: false,
          isSigner: true,
          docs: ["Signing authority over this queue"],
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["Payer for the initialization rent of the queue"],
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false,
          docs: ["solana system program"],
        },
      ],
      args: [
        {
          name: "params",
          type: {
            defined: "RegisterAdapterParams",
          },
        },
      ],
    },
    {
      name: "popAdapterEvents",
      docs: [
        "Pop the given number of events off the adapter queue",
        "Event logic is left to the outside program",
      ],
      accounts: [
        {
          name: "adapterQueue",
          isMut: true,
          isSigner: false,
          docs: ["AdapterEventQueue account owned by outside user or program"],
        },
        {
          name: "owner",
          isMut: false,
          isSigner: true,
          docs: ["Signing authority over the AdapterEventQueue"],
        },
      ],
      args: [
        {
          name: "numEvents",
          type: "u32",
        },
      ],
    },
  ],
  accounts: [
    {
      name: "BondManager",
      type: {
        kind: "struct",
        fields: [
          {
            name: "versionTag",
            docs: ["Versioning and tag information"],
            type: "u64",
          },
          {
            name: "programAuthority",
            docs: ["The address allowed to make changes to this program state"],
            type: "publicKey",
          },
          {
            name: "orderbookMarketState",
            docs: ["The market state of the agnostic orderbook"],
            type: "publicKey",
          },
          {
            name: "eventQueue",
            docs: ["The orderbook event queue"],
            type: "publicKey",
          },
          {
            name: "asksSlab",
            docs: ["The orderbook asks byteslab"],
            type: "publicKey",
          },
          {
            name: "bidsSlab",
            docs: ["The orderbook bids byteslab"],
            type: "publicKey",
          },
          {
            name: "underlyingTokenMint",
            docs: [
              "The token mint for the underlying asset of the bond tickets",
            ],
            type: "publicKey",
          },
          {
            name: "underlyingTokenVault",
            docs: [
              "Token account storing the underlying asset accounted for by this ticket program",
            ],
            type: "publicKey",
          },
          {
            name: "bondTicketMint",
            docs: ["The token mint for the bond tickets"],
            type: "publicKey",
          },
          {
            name: "claimsMint",
            docs: [
              "Mint owned by bonds to issue claims against a user.",
              "These claim notes are monitored by margin to ensure claims are repaid.",
            ],
            type: "publicKey",
          },
          {
            name: "oracle",
            docs: ["oracle that defines the value of the underlying asset"],
            type: "publicKey",
          },
          {
            name: "seed",
            docs: [
              "The user-defined part of the seed that generated this bond manager's PDA",
            ],
            type: {
              array: ["u8", 8],
            },
          },
          {
            name: "bump",
            docs: ["The bump seed value for generating the authority address."],
            type: {
              array: ["u8", 1],
            },
          },
          {
            name: "conversionFactor",
            docs: [
              "The number of decimals added or subtracted to the tickets staked when minting a `ClaimTicket`",
            ],
            type: "i8",
          },
          {
            name: "reserved",
            docs: ["reserved for future use"],
            type: {
              array: ["u8", 30],
            },
          },
          {
            name: "duration",
            docs: [
              "Units added to the initial stake timestamp to determine claim maturity",
            ],
            type: "i64",
          },
        ],
      },
    },
    {
      name: "OrderbookUser",
      docs: [
        "The orderbook user account tracks data about a user's state within the Bonds Orderbook",
      ],
      type: {
        kind: "struct",
        fields: [
          {
            name: "user",
            docs: ["The pubkey of the user. Used to verfiy withdraws, etc."],
            type: "publicKey",
          },
          {
            name: "bondManager",
            docs: [
              "The pubkey pointing to the BondMarket account tracking this user",
            ],
            type: "publicKey",
          },
          {
            name: "eventAdapter",
            docs: ["The address of the registered event adapter for this user"],
            type: "publicKey",
          },
          {
            name: "bondTicketsStored",
            docs: [
              "The quanitity of base token the user may allocate to orders or withdraws",
              "For the bonds program, this represents the bond tickets",
            ],
            type: "u64",
          },
          {
            name: "underlyingTokenStored",
            docs: [
              "The quantity of quote token the user may allocate to orders or withdraws",
              "For the bonds program, this represents the asset redeeemed for staking bond tickets",
            ],
            type: "u64",
          },
          {
            name: "outstandingObligations",
            docs: [
              "total number of outstanding obligations with committed debt",
            ],
            type: "u64",
          },
          {
            name: "debt",
            docs: [
              "The amount of debt that must be collateralized or repaid",
              "This debt is expressed in terms of the underlying token - not bond tickets",
            ],
            type: {
              defined: "Debt",
            },
          },
          {
            name: "claims",
            docs: [
              "Token account used by the margin program to track the debt",
            ],
            type: "publicKey",
          },
          {
            name: "nonce",
            docs: [
              "This nonce is used to generate unique order tags",
              "Instantiated as `0` and incremented with each order",
            ],
            type: "u64",
          },
        ],
      },
    },
    {
      name: "ClaimTicket",
      type: {
        kind: "struct",
        fields: [
          {
            name: "owner",
            docs: ["The account registered as owner of this claim"],
            type: "publicKey",
          },
          {
            name: "bondManager",
            docs: [
              "The `TicketManager` this claim ticket was established under",
              "Determines the asset this ticket will be redeemed for",
            ],
            type: "publicKey",
          },
          {
            name: "maturationTimestamp",
            docs: [
              "The slot after which this claim can be redeemed for the underlying value",
            ],
            type: "i64",
          },
          {
            name: "redeemable",
            docs: ["The number of tokens this claim  is redeemable for"],
            type: "u64",
          },
        ],
      },
    },
    {
      name: "SplitTicket",
      type: {
        kind: "struct",
        fields: [
          {
            name: "owner",
            docs: ["The account registered as owner of this claim"],
            type: "publicKey",
          },
          {
            name: "bondManager",
            docs: [
              "The `TicketManager` this claim ticket was established under",
              "Determines the asset this ticket will be redeemed for",
            ],
            type: "publicKey",
          },
          {
            name: "orderTag",
            docs: [
              "The `OrderTag` associated with the creation of this struct",
            ],
            type: {
              array: ["u8", 16],
            },
          },
          {
            name: "struckTimestamp",
            docs: ["The time slot during which the ticket was struck"],
            type: "i64",
          },
          {
            name: "maturationTimestamp",
            docs: [
              "The slot after which this claim can be redeemed for the underlying value",
            ],
            type: "i64",
          },
          {
            name: "principal",
            docs: [
              "The total number of principal tokens the bond was struck for",
            ],
            type: "u64",
          },
          {
            name: "interest",
            docs: [
              "The total number of interest tokens struck for this bond",
              "same underlying asset as the principal token",
            ],
            type: "u64",
          },
        ],
      },
    },
    {
      name: "EventAdapterMetadata",
      type: {
        kind: "struct",
        fields: [
          {
            name: "owner",
            docs: ["Signing authority over this Adapter"],
            type: "publicKey",
          },
          {
            name: "manager",
            docs: ["The `BondManager` this adapter belongs to"],
            type: "publicKey",
          },
          {
            name: "orderbookUser",
            docs: [
              "The `OrderbookUser` account this adapter is registered for",
            ],
            type: "publicKey",
          },
        ],
      },
    },
    {
      name: "Obligation",
      type: {
        kind: "struct",
        fields: [
          {
            name: "orderbookUserAccount",
            docs: ["The user (margin account) this obligation is owed by"],
            type: "publicKey",
          },
          {
            name: "bondManager",
            docs: ["The bond manager where the obligation was created"],
            type: "publicKey",
          },
          {
            name: "orderTag",
            docs: [
              "The `OrderTag` associated with the creation of this `Obligation`",
            ],
            type: {
              array: ["u8", 16],
            },
          },
          {
            name: "maturationTimestamp",
            docs: ["The time that the obligation must be repaid"],
            type: "i64",
          },
          {
            name: "balance",
            docs: ["The remaining amount due by the end of the loan term"],
            type: "u64",
          },
          {
            name: "flags",
            docs: [
              "Any boolean flags for this data type compressed to a single byte",
            ],
            type: "u8",
          },
        ],
      },
    },
  ],
  types: [
    {
      name: "RegisterAdapterParams",
      type: {
        kind: "struct",
        fields: [
          {
            name: "numEvents",
            docs: ["Total capacity of the adapter", "Increases rent cost"],
            type: "u32",
          },
        ],
      },
    },
    {
      name: "InitializeOrderbookParams",
      type: {
        kind: "struct",
        fields: [
          {
            name: "minBaseOrderSize",
            docs: [
              "The minimum order size that can be inserted into the orderbook after matching.",
            ],
            type: "u64",
          },
        ],
      },
    },
    {
      name: "InitializeBondManagerSeeds",
      type: {
        kind: "struct",
        fields: [
          {
            name: "uniquenessSeed",
            docs: [
              "This seed allows the creation of many separate ticket managers tracking different",
              "parameters, such as staking duration",
            ],
            type: "bytes",
          },
        ],
      },
    },
    {
      name: "InitializeBondManagerParams",
      type: {
        kind: "struct",
        fields: [
          {
            name: "versionTag",
            docs: ["Tag information for the `BondManager` account"],
            type: "u64",
          },
          {
            name: "seed",
            docs: [
              "This seed allows the creation of many separate ticket managers tracking different",
              "parameters, such as staking duration",
            ],
            type: "u64",
          },
          {
            name: "duration",
            docs: [
              "Units added to the initial stake timestamp to determine claim maturity",
            ],
            type: "i64",
          },
          {
            name: "conversionFactor",
            docs: [
              "The number of decimals added or subtracted to the tickets staked when minting a `ClaimTicket`",
            ],
            type: "i8",
          },
        ],
      },
    },
    {
      name: "StakeBondTicketsParams",
      type: {
        kind: "struct",
        fields: [
          {
            name: "amount",
            docs: ["number of tickets to stake"],
            type: "u64",
          },
          {
            name: "ticketSeed",
            docs: [
              "uniqueness seed to allow a user to have many `ClaimTicket`s",
            ],
            type: "bytes",
          },
        ],
      },
    },
    {
      name: "Debt",
      type: {
        kind: "struct",
        fields: [
          {
            name: "pending",
            docs: [
              "Amount that must be collateralized because there is an open order for it.",
              "Does not accrue interest because the loan has not been received yet.",
            ],
            type: "u64",
          },
          {
            name: "committed",
            docs: [
              "Debt that has already been borrowed because the order was matched.",
              "This debt will be due when the loan term ends.",
              "Some of this debt may actually be due already, but a crank has not yet been marked it as due.",
            ],
            type: "u64",
          },
          {
            name: "pastDue",
            docs: [
              "Amount of debt that has already been discovered and marked as being due",
              "This is not guaranteed to be comprehensive. It may not include some",
              "obligations that have not yet been marked due.",
            ],
            type: "u64",
          },
        ],
      },
    },
    {
      name: "OrderParams",
      type: {
        kind: "struct",
        fields: [
          {
            name: "maxBondTicketQty",
            docs: ["The maximum quantity of bond tickets to be traded."],
            type: "u64",
          },
          {
            name: "maxUnderlyingTokenQty",
            docs: ["The maximum quantity of underlying token to be traded."],
            type: "u64",
          },
          {
            name: "limitPrice",
            docs: [
              "The limit price of the order. This value is understood as a 32-bit fixed point number.",
            ],
            type: "u64",
          },
          {
            name: "matchLimit",
            docs: [
              "The maximum number of orderbook postings to match in order to fulfill the order",
            ],
            type: "u64",
          },
          {
            name: "postOnly",
            docs: [
              "The order will not be matched against the orderbook and will be direcly written into it.",
              "",
              "The operation will fail if the order's limit_price crosses the spread.",
            ],
            type: "bool",
          },
          {
            name: "postAllowed",
            docs: [
              "Should the unfilled portion of the order be reposted to the orderbook",
            ],
            type: "bool",
          },
          {
            name: "autoStake",
            docs: [
              "Should the purchased tickets be automatically staked with the ticket program",
            ],
            type: "bool",
          },
        ],
      },
    },
    {
      name: "OrderSide",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Lend",
          },
          {
            name: "Borrow",
          },
        ],
      },
    },
    {
      name: "AssetKind",
      type: {
        kind: "enum",
        variants: [
          {
            name: "UnderlyingToken",
          },
          {
            name: "BondTicket",
          },
        ],
      },
    },
  ],
  errors: [
    {
      code: 6000,
      name: "ArithmeticOverflow",
      msg: "overflow occured on checked_add",
    },
    {
      code: 6001,
      name: "ArithmeticUnderflow",
      msg: "underflow occured on checked_sub",
    },
    {
      code: 6002,
      name: "DoesNotOwnTicket",
      msg: "owner does not own the ticket",
    },
    {
      code: 6003,
      name: "DoesNotOwnEventAdapter",
      msg: "signer does not own the event adapter",
    },
    {
      code: 6004,
      name: "EventQueueFull",
      msg: "queue does not have room for another event",
    },
    {
      code: 6005,
      name: "FailedToDeserializeTicket",
      msg: "failed to deserialize the SplitTicket or ClaimTicket",
    },
    {
      code: 6006,
      name: "ImmatureBond",
      msg: "bond is not mature and cannot be claimed",
    },
    {
      code: 6007,
      name: "InsufficientSeeds",
      msg: "not enough seeds were provided for the accounts that need to be initialized",
    },
    {
      code: 6008,
      name: "InvalidEvent",
      msg: "the wrong event type was unwrapped\\nthis condition should be impossible, and does not result from invalid input",
    },
    {
      code: 6009,
      name: "InvokeCreateAccount",
      msg: "failed to invoke account creation",
    },
    {
      code: 6010,
      name: "IoError",
      msg: "failed to properly serialize or deserialize a data structure",
    },
    {
      code: 6011,
      name: "MarketStateNotProgramOwned",
      msg: "this market state account is not owned by the current program",
    },
    {
      code: 6012,
      name: "MissingEventAdapter",
      msg: "tried to access a missing adapter account",
    },
    {
      code: 6013,
      name: "NoEvents",
      msg: "consume_events instruction failed to consume a single event",
    },
    {
      code: 6014,
      name: "OracleError",
      msg: "there was a problem loading the price oracle",
    },
    {
      code: 6015,
      name: "OrderNotFound",
      msg: "id was not found in the user's open orders",
    },
    {
      code: 6016,
      name: "PriceMissing",
      msg: "price could not be accessed from oracle",
    },
    {
      code: 6017,
      name: "TicketNotFromManager",
      msg: "claim ticket is not from this manager",
    },
    {
      code: 6018,
      name: "UnauthorizedCaller",
      msg: "this signer is not authorized to place a permissioned order",
    },
    {
      code: 6019,
      name: "UserDoesNotOwnAccount",
      msg: "this user does not own the user account",
    },
    {
      code: 6020,
      name: "UserDoesNotOwnAdapter",
      msg: "this adapter does not belong to the user",
    },
    {
      code: 6021,
      name: "UserNotInMarket",
      msg: "this user account is not associated with this bond market",
    },
    {
      code: 6022,
      name: "WrongBondManager",
      msg: "adapter does not belong to given bond manager",
    },
    {
      code: 6023,
      name: "WrongCrankAuthority",
      msg: "wrong authority for this crank instruction",
    },
    {
      code: 6024,
      name: "WrongMarketState",
      msg: "this market state is not associated with this market",
    },
    {
      code: 6025,
      name: "WrongTicketManager",
      msg: "wrong TicketManager account provided",
    },
    {
      code: 6026,
      name: "DoesNotOwnMarket",
      msg: "this market owner does not own this market",
    },
    {
      code: 6027,
      name: "WrongClaimAccount",
      msg: "the wrong account was provided for the token account that represents a user's claims",
    },
    {
      code: 6028,
      name: "WrongClaimMint",
      msg: "the wrong account was provided for the claims token mint",
    },
    {
      code: 6029,
      name: "WrongOracle",
      msg: "wrong oracle address was sent to instruction",
    },
    {
      code: 6030,
      name: "WrongOrderbookUser",
      msg: "wrong orderbook user account address was sent to instruction",
    },
    {
      code: 6031,
      name: "WrongProgramAuthority",
      msg: "incorrect authority account",
    },
    {
      code: 6032,
      name: "WrongTicketMint",
      msg: "not the ticket mint for this bond market",
    },
    {
      code: 6033,
      name: "WrongVault",
      msg: "wrong vault address was sent to instruction",
    },
    {
      code: 6034,
      name: "ZeroDivision",
      msg: "attempted to divide with zero",
    },
  ],
};
