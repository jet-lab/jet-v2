export type JetMarginSerum = {
  "version": "0.1.0",
  "name": "jet_margin_serum",
  "instructions": [
    {
      "name": "cancelOrderByClientIdV2",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "clientOrderId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "cancelOrderV2",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": "u8"
        },
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "closeOpenOrders",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "destination",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "consumeEvents",
      "accounts": [
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinFeeReceivableAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcFeeReceivableAccount",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "matchOrders",
      "accounts": [
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "requestQueue",
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
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinFeeReceivableAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcFeeReceivableAccount",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "newOrderV3",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "requestQueue",
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
          "name": "payer",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "splTokenProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rentSysvarId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": "u8"
        },
        {
          "name": "limitPrice",
          "type": "u64"
        },
        {
          "name": "maxCoinQty",
          "type": "u64"
        },
        {
          "name": "maxNativePcQtyIncludingFees",
          "type": "u64"
        },
        {
          "name": "selfTradeBehavior",
          "type": "u8"
        },
        {
          "name": "orderType",
          "type": "u8"
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "settleFunds",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "splTokenProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultSigner",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    }
  ],
  "types": [
    {
      "name": "ErrorCode",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "InvalidOrderType"
          },
          {
            "name": "InvalidSelfTradeBehavior"
          },
          {
            "name": "InvalidSide"
          }
        ]
      }
    }
  ]
};

export const IDL: JetMarginSerum = {
  "version": "0.1.0",
  "name": "jet_margin_serum",
  "instructions": [
    {
      "name": "cancelOrderByClientIdV2",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "clientOrderId",
          "type": "u64"
        }
      ]
    },
    {
      "name": "cancelOrderV2",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketBids",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "marketAsks",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": "u8"
        },
        {
          "name": "orderId",
          "type": "u128"
        }
      ]
    },
    {
      "name": "closeOpenOrders",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "openOrders",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "destination",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "consumeEvents",
      "accounts": [
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinFeeReceivableAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcFeeReceivableAccount",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "matchOrders",
      "accounts": [
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "requestQueue",
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
          "name": "eventQueue",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinFeeReceivableAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcFeeReceivableAccount",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "newOrderV3",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "requestQueue",
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
          "name": "payer",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "baseVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "quoteVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "splTokenProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rentSysvarId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "side",
          "type": "u8"
        },
        {
          "name": "limitPrice",
          "type": "u64"
        },
        {
          "name": "maxCoinQty",
          "type": "u64"
        },
        {
          "name": "maxNativePcQtyIncludingFees",
          "type": "u64"
        },
        {
          "name": "selfTradeBehavior",
          "type": "u8"
        },
        {
          "name": "orderType",
          "type": "u8"
        },
        {
          "name": "clientOrderId",
          "type": "u64"
        },
        {
          "name": "limit",
          "type": "u16"
        }
      ]
    },
    {
      "name": "settleFunds",
      "accounts": [
        {
          "name": "marginAccount",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "serumProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "market",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "splTokenProgramId",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "openOrdersAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "coinWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcVault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "pcWallet",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vaultSigner",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    }
  ],
  "types": [
    {
      "name": "ErrorCode",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "InvalidOrderType"
          },
          {
            "name": "InvalidSelfTradeBehavior"
          },
          {
            "name": "InvalidSide"
          }
        ]
      }
    }
  ]
};
