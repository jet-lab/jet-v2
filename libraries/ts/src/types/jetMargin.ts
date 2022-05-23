export type JetMargin = {
  "version": "0.1.0",
  "name": "jet_margin",
  "constants": [
    {
      "name": "MIN_COLLATERAL_RATIO",
      "type": "u16",
      "value": "125_00"
    },
    {
      "name": "IDEAL_LIQUIDATION_COLLATERAL_RATIO",
      "type": "u16",
      "value": "130_00"
    },
    {
      "name": "MAX_LIQUIDATION_COLLATERAL_RATIO",
      "type": "u16",
      "value": "150_00"
    },
    {
      "name": "MAX_ORACLE_CONFIDENCE",
      "type": "u16",
      "value": "5_00"
    },
    {
      "name": "MAX_PRICE_QUOTE_AGE",
      "type": "u64",
      "value": "10"
    },
    {
      "name": "MAX_LIQUIDATION_VALUE_SLIPPAGE",
      "type": "u16",
      "value": "5_00"
    },
    {
      "name": "MAX_LIQUIDATION_C_RATIO_SLIPPAGE",
      "type": "u16",
      "value": "5_00"
    },
    {
      "name": "LIQUIDATION_TIMEOUT",
      "type": {
        "defined": "UnixTimestamp"
      },
      "value": "60"
    }
  ],
  "instructions": [
    {
      "name": "createAccount",
      "accounts": [
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
          "name": "marginAccount",
          "isMut": true,
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
          "name": "seed",
          "type": "u16"
        }
      ]
    },
    {
      "name": "closeAccount",
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "receiver",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "registerPosition",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "positionTokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "metadata",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
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
        }
      ],
      "args": []
    },
    {
      "name": "updatePositionBalance",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenAccount",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "closePosition",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "receiver",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenAccount",
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
      "name": "verifyHealthy",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "adapterInvoke",
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "adapterProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "adapterMetadata",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "accountMetas",
          "type": {
            "vec": {
              "defined": "CompactAccountMeta"
            }
          }
        },
        {
          "name": "data",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "accountingInvoke",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "adapterProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "adapterMetadata",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "accountMetas",
          "type": {
            "vec": {
              "defined": "CompactAccountMeta"
            }
          }
        },
        {
          "name": "data",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "liquidateBegin",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "liquidator",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "liquidatorMetadata",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidation",
          "isMut": true,
          "isSigner": false
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
      "name": "liquidateEnd",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidation",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "liquidatorInvoke",
      "accounts": [
        {
          "name": "liquidator",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "liquidation",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "adapterProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "adapterMetadata",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "accountMetas",
          "type": {
            "vec": {
              "defined": "CompactAccountMeta"
            }
          }
        },
        {
          "name": "data",
          "type": "bytes"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "marginAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "type": "u8"
          },
          {
            "name": "bumpSeed",
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "userSeed",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "reserved0",
            "type": {
              "array": [
                "u8",
                4
              ]
            }
          },
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "liquidation",
            "type": "publicKey"
          },
          {
            "name": "liquidator",
            "type": "publicKey"
          },
          {
            "name": "positions",
            "type": {
              "array": [
                "u8",
                7432
              ]
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "CompactAccountMeta",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "isSigner",
            "type": "u8"
          },
          {
            "name": "isWritable",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "PriceChangeInfo",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "mint",
            "type": "publicKey"
          },
          {
            "name": "value",
            "type": "i64"
          },
          {
            "name": "confidence",
            "type": "u64"
          },
          {
            "name": "twap",
            "type": "i64"
          },
          {
            "name": "slot",
            "type": "u64"
          },
          {
            "name": "exponent",
            "type": "i32"
          }
        ]
      }
    },
    {
      "name": "PriceInfo",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "value",
            "type": "i64"
          },
          {
            "name": "timestamp",
            "type": "u64"
          },
          {
            "name": "exponent",
            "type": "i32"
          },
          {
            "name": "isValid",
            "type": "u8"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          }
        ]
      }
    },
    {
      "name": "AdapterResult",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NewBalanceChange",
            "fields": [
              {
                "vec": "publicKey"
              }
            ]
          },
          {
            "name": "PriorBalanceChange",
            "fields": [
              {
                "vec": "publicKey"
              }
            ]
          },
          {
            "name": "PriceChange",
            "fields": [
              {
                "vec": {
                  "defined": "PriceChangeInfo"
                }
              }
            ]
          }
        ]
      }
    },
    {
      "name": "ErrorCode",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NoAdapterResult"
          },
          {
            "name": "WrongProgramAdapterResult"
          },
          {
            "name": "UnauthorizedInvocation"
          },
          {
            "name": "MaxPositions"
          },
          {
            "name": "UnknownPosition"
          },
          {
            "name": "CloseNonZeroPosition"
          },
          {
            "name": "PositionAlreadyRegistered"
          },
          {
            "name": "AccountNotEmpty"
          },
          {
            "name": "PositionNotOwned"
          },
          {
            "name": "InvalidPriceAdapter"
          },
          {
            "name": "OutdatedPrice"
          },
          {
            "name": "InvalidPrice"
          },
          {
            "name": "OutdatedBalance"
          },
          {
            "name": "Unhealthy"
          },
          {
            "name": "Healthy"
          },
          {
            "name": "Liquidating"
          },
          {
            "name": "NotLiquidating"
          },
          {
            "name": "StalePositions"
          },
          {
            "name": "UnauthorizedLiquidator"
          },
          {
            "name": "LiquidationLostValue"
          },
          {
            "name": "LiquidationUnhealthy"
          },
          {
            "name": "LiquidationTooHealthy"
          }
        ]
      }
    },
    {
      "name": "PositionKind",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NoValue"
          },
          {
            "name": "Deposit"
          },
          {
            "name": "Claim"
          }
        ]
      }
    }
  ]
};

export const IDL: JetMargin = {
  "version": "0.1.0",
  "name": "jet_margin",
  "constants": [
    {
      "name": "MIN_COLLATERAL_RATIO",
      "type": "u16",
      "value": "125_00"
    },
    {
      "name": "IDEAL_LIQUIDATION_COLLATERAL_RATIO",
      "type": "u16",
      "value": "130_00"
    },
    {
      "name": "MAX_LIQUIDATION_COLLATERAL_RATIO",
      "type": "u16",
      "value": "150_00"
    },
    {
      "name": "MAX_ORACLE_CONFIDENCE",
      "type": "u16",
      "value": "5_00"
    },
    {
      "name": "MAX_PRICE_QUOTE_AGE",
      "type": "u64",
      "value": "10"
    },
    {
      "name": "MAX_LIQUIDATION_VALUE_SLIPPAGE",
      "type": "u16",
      "value": "5_00"
    },
    {
      "name": "MAX_LIQUIDATION_C_RATIO_SLIPPAGE",
      "type": "u16",
      "value": "5_00"
    },
    {
      "name": "LIQUIDATION_TIMEOUT",
      "type": {
        "defined": "UnixTimestamp"
      },
      "value": "60"
    }
  ],
  "instructions": [
    {
      "name": "createAccount",
      "accounts": [
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
          "name": "marginAccount",
          "isMut": true,
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
          "name": "seed",
          "type": "u16"
        }
      ]
    },
    {
      "name": "closeAccount",
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "receiver",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "registerPosition",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "positionTokenMint",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "metadata",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
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
        }
      ],
      "args": []
    },
    {
      "name": "updatePositionBalance",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenAccount",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "closePosition",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "receiver",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenAccount",
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
      "name": "verifyHealthy",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "adapterInvoke",
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "adapterProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "adapterMetadata",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "accountMetas",
          "type": {
            "vec": {
              "defined": "CompactAccountMeta"
            }
          }
        },
        {
          "name": "data",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "accountingInvoke",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "adapterProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "adapterMetadata",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "accountMetas",
          "type": {
            "vec": {
              "defined": "CompactAccountMeta"
            }
          }
        },
        {
          "name": "data",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "liquidateBegin",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "liquidator",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "liquidatorMetadata",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "liquidation",
          "isMut": true,
          "isSigner": false
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
      "name": "liquidateEnd",
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "liquidation",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "liquidatorInvoke",
      "accounts": [
        {
          "name": "liquidator",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "liquidation",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marginAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "adapterProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "adapterMetadata",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "accountMetas",
          "type": {
            "vec": {
              "defined": "CompactAccountMeta"
            }
          }
        },
        {
          "name": "data",
          "type": "bytes"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "marginAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "type": "u8"
          },
          {
            "name": "bumpSeed",
            "type": {
              "array": [
                "u8",
                1
              ]
            }
          },
          {
            "name": "userSeed",
            "type": {
              "array": [
                "u8",
                2
              ]
            }
          },
          {
            "name": "reserved0",
            "type": {
              "array": [
                "u8",
                4
              ]
            }
          },
          {
            "name": "owner",
            "type": "publicKey"
          },
          {
            "name": "liquidation",
            "type": "publicKey"
          },
          {
            "name": "liquidator",
            "type": "publicKey"
          },
          {
            "name": "positions",
            "type": {
              "array": [
                "u8",
                7432
              ]
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "CompactAccountMeta",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "isSigner",
            "type": "u8"
          },
          {
            "name": "isWritable",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "PriceChangeInfo",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "mint",
            "type": "publicKey"
          },
          {
            "name": "value",
            "type": "i64"
          },
          {
            "name": "confidence",
            "type": "u64"
          },
          {
            "name": "twap",
            "type": "i64"
          },
          {
            "name": "slot",
            "type": "u64"
          },
          {
            "name": "exponent",
            "type": "i32"
          }
        ]
      }
    },
    {
      "name": "PriceInfo",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "value",
            "type": "i64"
          },
          {
            "name": "timestamp",
            "type": "u64"
          },
          {
            "name": "exponent",
            "type": "i32"
          },
          {
            "name": "isValid",
            "type": "u8"
          },
          {
            "name": "reserved",
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          }
        ]
      }
    },
    {
      "name": "AdapterResult",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NewBalanceChange",
            "fields": [
              {
                "vec": "publicKey"
              }
            ]
          },
          {
            "name": "PriorBalanceChange",
            "fields": [
              {
                "vec": "publicKey"
              }
            ]
          },
          {
            "name": "PriceChange",
            "fields": [
              {
                "vec": {
                  "defined": "PriceChangeInfo"
                }
              }
            ]
          }
        ]
      }
    },
    {
      "name": "ErrorCode",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NoAdapterResult"
          },
          {
            "name": "WrongProgramAdapterResult"
          },
          {
            "name": "UnauthorizedInvocation"
          },
          {
            "name": "MaxPositions"
          },
          {
            "name": "UnknownPosition"
          },
          {
            "name": "CloseNonZeroPosition"
          },
          {
            "name": "PositionAlreadyRegistered"
          },
          {
            "name": "AccountNotEmpty"
          },
          {
            "name": "PositionNotOwned"
          },
          {
            "name": "InvalidPriceAdapter"
          },
          {
            "name": "OutdatedPrice"
          },
          {
            "name": "InvalidPrice"
          },
          {
            "name": "OutdatedBalance"
          },
          {
            "name": "Unhealthy"
          },
          {
            "name": "Healthy"
          },
          {
            "name": "Liquidating"
          },
          {
            "name": "NotLiquidating"
          },
          {
            "name": "StalePositions"
          },
          {
            "name": "UnauthorizedLiquidator"
          },
          {
            "name": "LiquidationLostValue"
          },
          {
            "name": "LiquidationUnhealthy"
          },
          {
            "name": "LiquidationTooHealthy"
          }
        ]
      }
    },
    {
      "name": "PositionKind",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NoValue"
          },
          {
            "name": "Deposit"
          },
          {
            "name": "Claim"
          }
        ]
      }
    }
  ]
};
