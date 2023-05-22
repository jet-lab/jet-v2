type JetTestServiceIDL = {
  version: '0.1.0';
  name: 'jet_test_service';
  constants: [
    {
      name: 'TOKEN_MINT';
      type: {
        defined: '&[u8]';
      };
      value: 'b"token-mint"';
    },
    {
      name: 'TOKEN_INFO';
      type: {
        defined: '&[u8]';
      };
      value: 'b"token-info"';
    },
    {
      name: 'TOKEN_PYTH_PRICE';
      type: {
        defined: '&[u8]';
      };
      value: 'b"token-pyth-price"';
    },
    {
      name: 'TOKEN_PYTH_PRODUCT';
      type: {
        defined: '&[u8]';
      };
      value: 'b"token-pyth-product"';
    }
  ];
  instructions: [
    {
      name: 'tokenCreate';
      docs: [
        'Create a token mint based on some seed',
        '',
        'The created mint has a this program as the authority, any user may request',
        'tokens via the `token_request` instruction up to the limit specified in the',
        '`max_amount` parameter.',
        '',
        'This will also create pyth oracle accounts for the token.'
      ];
      accounts: [
        {
          name: 'payer';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'mint';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'info';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'pythPrice';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'pythProduct';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'rent';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'params';
          type: {
            defined: 'TokenCreateParams';
          };
        }
      ];
    },
    {
      name: 'tokenInitNative';
      docs: [
        'Initialize the token info and oracles for the native token mint',
        '',
        "Since the native mint is a special case that can't be owned by this program,",
        'this instruction allows creating an oracle for it.'
      ];
      accounts: [
        {
          name: 'payer';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'mint';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'info';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'pythPrice';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'pythProduct';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'rent';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'oracleAuthority';
          type: 'publicKey';
        }
      ];
    },
    {
      name: 'tokenRequest';
      docs: ['Request tokens be minted by the faucet.'];
      accounts: [
        {
          name: 'requester';
          isMut: true;
          isSigner: true;
          docs: ['user requesting tokens'];
        },
        {
          name: 'mint';
          isMut: true;
          isSigner: false;
          docs: ['The relevant token mint'];
        },
        {
          name: 'info';
          isMut: false;
          isSigner: false;
          docs: ['The test info for the token'];
        },
        {
          name: 'destination';
          isMut: true;
          isSigner: false;
          docs: ['The destination account for the minted tokens'];
        },
        {
          name: 'tokenProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'amount';
          type: 'u64';
        }
      ];
    },
    {
      name: 'tokenUpdatePythPrice';
      docs: ['Update the pyth oracle price account for a token'];
      accounts: [
        {
          name: 'oracleAuthority';
          isMut: false;
          isSigner: true;
        },
        {
          name: 'info';
          isMut: false;
          isSigner: false;
        },
        {
          name: 'pythPrice';
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'price';
          type: 'i64';
        },
        {
          name: 'conf';
          type: 'i64';
        },
        {
          name: 'expo';
          type: 'i32';
        }
      ];
    }
  ];
  accounts: [
    {
      name: 'tokenInfo';
      docs: ['Information about a token created by this testing service'];
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'bumpSeed';
            type: 'u8';
          },
          {
            name: 'symbol';
            type: 'string';
          },
          {
            name: 'name';
            type: 'string';
          },
          {
            name: 'authority';
            type: 'publicKey';
          },
          {
            name: 'oracleAuthority';
            type: 'publicKey';
          },
          {
            name: 'mint';
            type: 'publicKey';
          },
          {
            name: 'pythPrice';
            type: 'publicKey';
          },
          {
            name: 'pythProduct';
            type: 'publicKey';
          },
          {
            name: 'maxRequestAmount';
            type: 'u64';
          }
        ];
      };
    }
  ];
  types: [
    {
      name: 'TokenCreateParams';
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'symbol';
            docs: ['The symbol string for the token'];
            type: 'string';
          },
          {
            name: 'name';
            docs: ['The name or description of the token', '', 'Used to derive the mint address'];
            type: 'string';
          },
          {
            name: 'decimals';
            docs: ['The decimals for the mint'];
            type: 'u8';
          },
          {
            name: 'authority';
            docs: ['The authority over the token'];
            type: 'publicKey';
          },
          {
            name: 'oracleAuthority';
            docs: ['The authority to set prices'];
            type: 'publicKey';
          },
          {
            name: 'maxAmount';
            docs: ['The maximum amount of the token a user can request to mint in a', 'single instruction.'];
            type: 'u64';
          }
        ];
      };
    }
  ];
  errors: [
    {
      code: 669000;
      name: 'PermissionDenied';
    }
  ];
};
