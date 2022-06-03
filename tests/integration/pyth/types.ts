export type Pyth = {
  "version": "0.1.0",
  "name": "pyth",
  "instructions": [
    {
      "name": "initialize",
      "accounts": [
        {
          "name": "product",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "price",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "price",
          "type": "i64"
        },
        {
          "name": "expo",
          "type": "i32"
        },
        {
          "name": "conf",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updatePrice",
      "accounts": [
        {
          "name": "price",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "price",
          "type": "i64"
        },
        {
          "name": "conf",
          "type": "u64"
        }
      ]
    }
  ],
  "types": [
    {
      "name": "AccountType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Unknown"
          },
          {
            "name": "Mapping"
          },
          {
            "name": "Product"
          },
          {
            "name": "Price"
          }
        ]
      }
    },
    {
      "name": "CorpAction",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NoCorpAct"
          }
        ]
      }
    },
    {
      "name": "PriceStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Unknown"
          },
          {
            "name": "Trading"
          },
          {
            "name": "Halted"
          },
          {
            "name": "Auction"
          }
        ]
      }
    },
    {
      "name": "PriceType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Unknown"
          },
          {
            "name": "Price"
          }
        ]
      }
    }
  ]
};

export const IDL: Pyth = {
  "version": "0.1.0",
  "name": "pyth",
  "instructions": [
    {
      "name": "initialize",
      "accounts": [
        {
          "name": "product",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "price",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "price",
          "type": "i64"
        },
        {
          "name": "expo",
          "type": "i32"
        },
        {
          "name": "conf",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updatePrice",
      "accounts": [
        {
          "name": "price",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "price",
          "type": "i64"
        },
        {
          "name": "conf",
          "type": "u64"
        }
      ]
    }
  ],
  "types": [
    {
      "name": "AccountType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Unknown"
          },
          {
            "name": "Mapping"
          },
          {
            "name": "Product"
          },
          {
            "name": "Price"
          }
        ]
      }
    },
    {
      "name": "CorpAction",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "NoCorpAct"
          }
        ]
      }
    },
    {
      "name": "PriceStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Unknown"
          },
          {
            "name": "Trading"
          },
          {
            "name": "Halted"
          },
          {
            "name": "Auction"
          }
        ]
      }
    },
    {
      "name": "PriceType",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Unknown"
          },
          {
            "name": "Price"
          }
        ]
      }
    }
  ]
};
