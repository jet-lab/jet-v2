"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.IDL = void 0;
exports.IDL = {
    version: "0.1.0",
    name: "jet_margin",
    constants: [
        {
            name: "MAX_ORACLE_CONFIDENCE",
            type: "u16",
            value: "5_00"
        },
        {
            name: "MAX_ORACLE_STALENESS",
            type: "i64",
            value: "30"
        },
        {
            name: "MAX_PRICE_QUOTE_AGE",
            type: "u64",
            value: "30"
        },
        {
            name: "LIQUIDATION_TIMEOUT",
            type: {
                defined: "UnixTimestamp"
            },
            value: "60"
        }
    ],
    instructions: [
        {
            name: "createAccount",
            accounts: [
                {
                    name: "owner",
                    isMut: false,
                    isSigner: true
                },
                {
                    name: "payer",
                    isMut: true,
                    isSigner: true
                },
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
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
                    type: "u16"
                }
            ]
        },
        {
            name: "closeAccount",
            accounts: [
                {
                    name: "owner",
                    isMut: false,
                    isSigner: true
                },
                {
                    name: "receiver",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                }
            ],
            args: []
        },
        {
            name: "registerPosition",
            accounts: [
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
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "positionTokenMint",
                    isMut: false,
                    isSigner: false
                },
                {
                    name: "metadata",
                    isMut: false,
                    isSigner: false
                },
                {
                    name: "tokenAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "tokenProgram",
                    isMut: false,
                    isSigner: false
                },
                {
                    name: "rent",
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
            name: "updatePositionBalance",
            accounts: [
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "tokenAccount",
                    isMut: false,
                    isSigner: false
                }
            ],
            args: []
        },
        {
            name: "refreshPositionMetadata",
            accounts: [
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "metadata",
                    isMut: false,
                    isSigner: false
                }
            ],
            args: []
        },
        {
            name: "closePosition",
            accounts: [
                {
                    name: "authority",
                    isMut: false,
                    isSigner: true
                },
                {
                    name: "receiver",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "positionTokenMint",
                    isMut: false,
                    isSigner: false
                },
                {
                    name: "tokenAccount",
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
            name: "verifyHealthy",
            accounts: [
                {
                    name: "marginAccount",
                    isMut: false,
                    isSigner: false
                }
            ],
            args: []
        },
        {
            name: "adapterInvoke",
            accounts: [
                {
                    name: "owner",
                    isMut: false,
                    isSigner: true
                },
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "adapterProgram",
                    isMut: false,
                    isSigner: false
                },
                {
                    name: "adapterMetadata",
                    isMut: false,
                    isSigner: false
                }
            ],
            args: [
                {
                    name: "data",
                    type: "bytes"
                }
            ]
        },
        {
            name: "accountingInvoke",
            accounts: [
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "adapterProgram",
                    isMut: false,
                    isSigner: false
                },
                {
                    name: "adapterMetadata",
                    isMut: false,
                    isSigner: false
                }
            ],
            args: [
                {
                    name: "data",
                    type: "bytes"
                }
            ]
        },
        {
            name: "liquidateBegin",
            accounts: [
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "payer",
                    isMut: true,
                    isSigner: true
                },
                {
                    name: "liquidator",
                    isMut: false,
                    isSigner: true
                },
                {
                    name: "liquidatorMetadata",
                    isMut: false,
                    isSigner: false
                },
                {
                    name: "liquidation",
                    isMut: true,
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
            name: "liquidateEnd",
            accounts: [
                {
                    name: "authority",
                    isMut: true,
                    isSigner: true
                },
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "liquidation",
                    isMut: true,
                    isSigner: false
                }
            ],
            args: []
        },
        {
            name: "liquidatorInvoke",
            accounts: [
                {
                    name: "liquidator",
                    isMut: false,
                    isSigner: true
                },
                {
                    name: "liquidation",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "marginAccount",
                    isMut: true,
                    isSigner: false
                },
                {
                    name: "adapterProgram",
                    isMut: false,
                    isSigner: false
                },
                {
                    name: "adapterMetadata",
                    isMut: false,
                    isSigner: false
                }
            ],
            args: [
                {
                    name: "data",
                    type: "bytes"
                }
            ]
        }
    ],
    accounts: [
        {
            name: "marginAccount",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "version",
                        type: "u8"
                    },
                    {
                        name: "bumpSeed",
                        type: {
                            array: ["u8", 1]
                        }
                    },
                    {
                        name: "userSeed",
                        type: {
                            array: ["u8", 2]
                        }
                    },
                    {
                        name: "invocation",
                        type: {
                            defined: "Invocation"
                        }
                    },
                    {
                        name: "reserved0",
                        type: {
                            array: ["u8", 3]
                        }
                    },
                    {
                        name: "owner",
                        type: "publicKey"
                    },
                    {
                        name: "liquidation",
                        type: "publicKey"
                    },
                    {
                        name: "liquidator",
                        type: "publicKey"
                    },
                    {
                        name: "positions",
                        type: {
                            array: ["u8", 7432]
                        }
                    }
                ]
            }
        },
        {
            name: "liquidation",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "startTime",
                        type: "i64"
                    },
                    {
                        name: "valueChange",
                        type: "i128"
                    },
                    {
                        name: "minValueChange",
                        type: "i128"
                    }
                ]
            }
        }
    ],
    types: [
        {
            name: "AdapterResult",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "positionChanges",
                        type: {
                            vec: {
                                defined: "(Pubkey,Vec<PositionChange>)"
                            }
                        }
                    }
                ]
            }
        },
        {
            name: "PriceChangeInfo",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "value",
                        type: "i64"
                    },
                    {
                        name: "confidence",
                        type: "u64"
                    },
                    {
                        name: "twap",
                        type: "i64"
                    },
                    {
                        name: "publishTime",
                        type: "i64"
                    },
                    {
                        name: "exponent",
                        type: "i32"
                    }
                ]
            }
        },
        {
            name: "ValuationSummary",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "liabilities",
                        type: "i128"
                    },
                    {
                        name: "requiredCollateral",
                        type: "i128"
                    },
                    {
                        name: "weightedCollateral",
                        type: "i128"
                    },
                    {
                        name: "effectiveCollateral",
                        type: "i128"
                    },
                    {
                        name: "availableCollateral",
                        type: "i128"
                    },
                    {
                        name: "pastDue",
                        type: "bool"
                    }
                ]
            }
        },
        {
            name: "PriceInfo",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "value",
                        type: "i64"
                    },
                    {
                        name: "timestamp",
                        type: "u64"
                    },
                    {
                        name: "exponent",
                        type: "i32"
                    },
                    {
                        name: "isValid",
                        type: "u8"
                    },
                    {
                        name: "reserved",
                        type: {
                            array: ["u8", 3]
                        }
                    }
                ]
            }
        },
        {
            name: "AccountPosition",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "token",
                        type: "publicKey"
                    },
                    {
                        name: "address",
                        type: "publicKey"
                    },
                    {
                        name: "adapter",
                        type: "publicKey"
                    },
                    {
                        name: "value",
                        type: {
                            array: ["u8", 16]
                        }
                    },
                    {
                        name: "balance",
                        type: "u64"
                    },
                    {
                        name: "balanceTimestamp",
                        type: "u64"
                    },
                    {
                        name: "price",
                        type: {
                            defined: "PriceInfo"
                        }
                    },
                    {
                        name: "kind",
                        type: {
                            defined: "PositionKind"
                        }
                    },
                    {
                        name: "exponent",
                        type: "i16"
                    },
                    {
                        name: "valueModifier",
                        type: "u16"
                    },
                    {
                        name: "maxStaleness",
                        type: "u64"
                    },
                    {
                        name: "flags",
                        type: {
                            defined: "AdapterPositionFlags"
                        }
                    },
                    {
                        name: "reserved",
                        type: {
                            array: ["u8", 23]
                        }
                    }
                ]
            }
        },
        {
            name: "AccountPositionKey",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "mint",
                        type: "publicKey"
                    },
                    {
                        name: "index",
                        type: {
                            defined: "usize"
                        }
                    }
                ]
            }
        },
        {
            name: "AccountPositionList",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "length",
                        type: {
                            defined: "usize"
                        }
                    },
                    {
                        name: "map",
                        type: {
                            array: [
                                {
                                    defined: "AccountPositionKey"
                                },
                                32
                            ]
                        }
                    },
                    {
                        name: "positions",
                        type: {
                            array: [
                                {
                                    defined: "AccountPosition"
                                },
                                32
                            ]
                        }
                    }
                ]
            }
        },
        {
            name: "PositionChange",
            type: {
                kind: "enum",
                variants: [
                    {
                        name: "Price",
                        fields: [
                            {
                                defined: "PriceChangeInfo"
                            }
                        ]
                    },
                    {
                        name: "Flags",
                        fields: [
                            {
                                defined: "AdapterPositionFlags"
                            },
                            "bool"
                        ]
                    },
                    {
                        name: "Register",
                        fields: ["publicKey"]
                    },
                    {
                        name: "Close",
                        fields: ["publicKey"]
                    }
                ]
            }
        },
        {
            name: "PositionKind",
            type: {
                kind: "enum",
                variants: [
                    {
                        name: "NoValue"
                    },
                    {
                        name: "Deposit"
                    },
                    {
                        name: "Claim"
                    }
                ]
            }
        },
        {
            name: "Approver",
            type: {
                kind: "enum",
                variants: [
                    {
                        name: "MarginAccountAuthority"
                    },
                    {
                        name: "Adapter",
                        fields: ["publicKey"]
                    }
                ]
            }
        },
        {
            name: "Invocation",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "callerHeights",
                        type: "u8"
                    }
                ]
            }
        },
        {
            name: "AdapterPositionFlags",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "flags",
                        type: "u8"
                    }
                ]
            }
        },
        {
            name: "Liquidation",
            type: {
                kind: "struct",
                fields: [
                    {
                        name: "startTime",
                        type: "i64"
                    },
                    {
                        name: "valueChange",
                        type: "i128"
                    },
                    {
                        name: "minValueChange",
                        type: "i128"
                    }
                ]
            }
        }
    ],
    events: [
        {
            name: "AccountCreated",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "owner",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "seed",
                    type: "u16",
                    index: false
                }
            ]
        },
        {
            name: "AccountClosed",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                }
            ]
        },
        {
            name: "VerifiedHealthy",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                }
            ]
        },
        {
            name: "PositionRegistered",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "authority",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "position",
                    type: {
                        defined: "AccountPosition"
                    },
                    index: false
                }
            ]
        },
        {
            name: "PositionClosed",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "authority",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "token",
                    type: "publicKey",
                    index: false
                }
            ]
        },
        {
            name: "PositionMetadataRefreshed",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "position",
                    type: {
                        defined: "AccountPosition"
                    },
                    index: false
                }
            ]
        },
        {
            name: "PositionBalanceUpdated",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "position",
                    type: {
                        defined: "AccountPosition"
                    },
                    index: false
                }
            ]
        },
        {
            name: "PositionTouched",
            fields: [
                {
                    name: "position",
                    type: {
                        defined: "AccountPosition"
                    },
                    index: false
                }
            ]
        },
        {
            name: "AccountingInvokeBegin",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "adapterProgram",
                    type: "publicKey",
                    index: false
                }
            ]
        },
        {
            name: "AccountingInvokeEnd",
            fields: []
        },
        {
            name: "AdapterInvokeBegin",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "adapterProgram",
                    type: "publicKey",
                    index: false
                }
            ]
        },
        {
            name: "AdapterInvokeEnd",
            fields: []
        },
        {
            name: "LiquidationBegun",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "liquidator",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "liquidation",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "liquidationData",
                    type: {
                        defined: "Liquidation"
                    },
                    index: false
                },
                {
                    name: "valuationSummary",
                    type: {
                        defined: "ValuationSummary"
                    },
                    index: false
                }
            ]
        },
        {
            name: "LiquidatorInvokeBegin",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "adapterProgram",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "liquidator",
                    type: "publicKey",
                    index: false
                }
            ]
        },
        {
            name: "LiquidatorInvokeEnd",
            fields: [
                {
                    name: "liquidationData",
                    type: {
                        defined: "Liquidation"
                    },
                    index: false
                },
                {
                    name: "valuationSummary",
                    type: {
                        defined: "ValuationSummary"
                    },
                    index: false
                }
            ]
        },
        {
            name: "LiquidationEnded",
            fields: [
                {
                    name: "marginAccount",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "authority",
                    type: "publicKey",
                    index: false
                },
                {
                    name: "timedOut",
                    type: "bool",
                    index: false
                }
            ]
        }
    ],
    errors: [
        {
            code: 141000,
            name: "NoAdapterResult"
        },
        {
            code: 141001,
            name: "WrongProgramAdapterResult",
            msg: "The program that set the result was not the adapter"
        },
        {
            code: 141002,
            name: "UnauthorizedInvocation",
            msg: "this invocation is not authorized by the necessary accounts"
        },
        {
            code: 141003,
            name: "IndirectInvocation",
            msg: "the current instruction was not directly invoked by the margin program"
        },
        {
            code: 141010,
            name: "MaxPositions",
            msg: "account cannot record any additional positions"
        },
        {
            code: 141011,
            name: "UnknownPosition",
            msg: "account has no record of the position"
        },
        {
            code: 141012,
            name: "CloseNonZeroPosition",
            msg: "attempting to close a position that has a balance"
        },
        {
            code: 141013,
            name: "PositionAlreadyRegistered",
            msg: "attempting to register an existing position"
        },
        {
            code: 141014,
            name: "AccountNotEmpty",
            msg: "attempting to close non-empty margin account"
        },
        {
            code: 141015,
            name: "PositionNotRegistered",
            msg: "attempting to use unregistered position"
        },
        {
            code: 141016,
            name: "CloseRequiredPosition",
            msg: "attempting to close a position that is required by the adapter"
        },
        {
            code: 141017,
            name: "InvalidPositionOwner",
            msg: "registered position owner inconsistent with PositionTokenMetadata owner or token_kind"
        },
        {
            code: 141018,
            name: "PositionNotRegisterable",
            msg: "dependencies are not satisfied to auto-register a required but unregistered position"
        },
        {
            code: 141020,
            name: "InvalidPositionAdapter",
            msg: "wrong adapter to modify the position"
        },
        {
            code: 141021,
            name: "OutdatedPrice",
            msg: "a position price is outdated"
        },
        {
            code: 141022,
            name: "InvalidPrice",
            msg: "an asset price is currently invalid"
        },
        {
            code: 141023,
            name: "OutdatedBalance",
            msg: "a position balance is outdated"
        },
        {
            code: 141030,
            name: "Unhealthy",
            msg: "the account is not healthy"
        },
        {
            code: 141031,
            name: "Healthy",
            msg: "the account is already healthy"
        },
        {
            code: 141032,
            name: "Liquidating",
            msg: "the account is being liquidated"
        },
        {
            code: 141033,
            name: "NotLiquidating",
            msg: "the account is not being liquidated"
        },
        {
            code: 141034,
            name: "StalePositions"
        },
        {
            code: 141040,
            name: "UnauthorizedLiquidator",
            msg: "the liquidator does not have permission to do this"
        },
        {
            code: 141041,
            name: "LiquidationLostValue",
            msg: "attempted to extract too much value during liquidation"
        }
    ]
};
//# sourceMappingURL=jetMargin.js.map