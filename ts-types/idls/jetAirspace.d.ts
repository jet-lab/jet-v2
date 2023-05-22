type JetAirspaceIDL = {
  version: '0.1.0';
  name: 'jet_airspace';
  constants: [
    {
      name: 'GOVERNOR_ID';
      type: {
        defined: '&[u8]';
      };
      value: 'b"governor-id"';
    },
    {
      name: 'AIRSPACE';
      type: {
        defined: '&[u8]';
      };
      value: 'b"airspace"';
    },
    {
      name: 'AIRSPACE_PERMIT_ISSUER';
      type: {
        defined: '&[u8]';
      };
      value: 'b"airspace-permit-issuer"';
    },
    {
      name: 'AIRSPACE_PERMIT';
      type: {
        defined: '&[u8]';
      };
      value: 'b"airspace-permit"';
    },
    {
      name: 'GOVERNOR_DEFAULT';
      type: 'publicKey';
      value: 'pubkey ! ("7R6FjP2HfXAgKQjURC4tCBrUmRQLCgEUeX2berrfU4ox")';
    }
  ];
  instructions: [
    {
      name: 'createGovernorId';
      docs: [
        'Create the governor identity account',
        '',
        'If this is a testing environment, the signer on the first transaction executing this',
        'instruction becomes the first governor. For mainnet environment the first governor',
        'is set from a hardcoded default.'
      ];
      accounts: [
        {
          name: 'payer';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'governorId';
          isMut: true;
          isSigner: false;
          docs: ['The governer identity account'];
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: 'setGovernor';
      docs: [
        'Set the protocol governor address. Must be signed by the current governor address.',
        '',
        '# Parameters',
        '',
        '* `new_governor` - The new address with governor authority'
      ];
      accounts: [
        {
          name: 'governor';
          isMut: false;
          isSigner: true;
          docs: ['The current governor'];
        },
        {
          name: 'governorId';
          isMut: true;
          isSigner: false;
          docs: ['The governer identity account'];
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'newGovernor';
          type: 'publicKey';
        }
      ];
    },
    {
      name: 'airspaceCreate';
      docs: [
        'Create a new airspace, which serves as an isolation boundary for resources in the protocol',
        '',
        '# Parameters',
        '',
        '* `seed` - An arbitrary string of bytes used to generate the airspace address.',
        '* `is_restricted` - If true, then user access to create an account within the airspace is',
        'restricted, and must be approved by some regulating authority.',
        '* `authority` - The utimate authority with permission to modify things about an airspace.'
      ];
      accounts: [
        {
          name: 'payer';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'airspace';
          isMut: true;
          isSigner: false;
          docs: ['The airspace account to be created'];
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'seed';
          type: 'string';
        },
        {
          name: 'isRestricted';
          type: 'bool';
        },
        {
          name: 'authority';
          type: 'publicKey';
        }
      ];
    },
    {
      name: 'airspaceSetAuthority';
      docs: [
        'Change the authority for an airspace',
        '',
        '# Parameters',
        '',
        '* `new_authority` - The address that the authority is being changed to.'
      ];
      accounts: [
        {
          name: 'authority';
          isMut: false;
          isSigner: true;
          docs: ['The current airspace authority'];
        },
        {
          name: 'airspace';
          isMut: true;
          isSigner: false;
          docs: ['The airspace to have its authority changed'];
        }
      ];
      args: [
        {
          name: 'newAuthority';
          type: 'publicKey';
        }
      ];
    },
    {
      name: 'airspacePermitIssuerCreate';
      docs: [
        'Create a new license for an address to serve as an airspace regulator.',
        '',
        'Addresses with regulator licenses in an airspace are allowed to issue new permits',
        'for other addresses to utilize the airspace.',
        '',
        '# Parameters',
        '',
        '* `issuer` - The address that is being granted the permission to issue permits.'
      ];
      accounts: [
        {
          name: 'payer';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: true;
          docs: ['The airspace authority'];
        },
        {
          name: 'airspace';
          isMut: false;
          isSigner: false;
          docs: ['The airspace the regulator will grant permits for'];
        },
        {
          name: 'issuerId';
          isMut: true;
          isSigner: false;
          docs: ['The license account, which will prove the given regulator has authority to', 'grant new permits.'];
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'issuer';
          type: 'publicKey';
        }
      ];
    },
    {
      name: 'airspacePermitIssuerRevoke';
      docs: [
        'Revoke a previously authorized permit issuer, preventing the permit issuer from issuing any',
        'new permits for the airspace.'
      ];
      accounts: [
        {
          name: 'receiver';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: true;
          docs: ['The airspace authority'];
        },
        {
          name: 'airspace';
          isMut: false;
          isSigner: false;
          docs: ['The airspace the regulator is to be removed from'];
        },
        {
          name: 'issuerId';
          isMut: true;
          isSigner: false;
          docs: ['The license account that will be removed for the regulator'];
        }
      ];
      args: [];
    },
    {
      name: 'airspacePermitCreate';
      docs: [
        'Create a new permit, allowing an address access to resources in an airspace',
        '',
        '# Parameters',
        '',
        '* `owner` - The owner for the new permit, which is the address being allowed to use',
        'the airspace.'
      ];
      accounts: [
        {
          name: 'payer';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: true;
          docs: [
            'The authority allowed to create a permit in the airspace',
            '',
            'If the airspace is restricted, then this must be either the airspace authority or',
            'an authorized regulator.'
          ];
        },
        {
          name: 'airspace';
          isMut: false;
          isSigner: false;
          docs: ['The airspace the new permit is for'];
        },
        {
          name: 'permit';
          isMut: true;
          isSigner: false;
          docs: ['The airspace account to be created'];
        },
        {
          name: 'issuerId';
          isMut: false;
          isSigner: false;
          docs: [
            'The identity account granting issuer permission for the authority.',
            '',
            'This account is not always required to exist, and only required when the airspace',
            'is restricted, and the authority is not the airspace authority.'
          ];
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'owner';
          type: 'publicKey';
        }
      ];
    },
    {
      name: 'airspacePermitRevoke';
      docs: ['Revoke a previously created permit'];
      accounts: [
        {
          name: 'receiver';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'authority';
          isMut: false;
          isSigner: true;
          docs: [
            'The authority allowed to revoke an airspace permit',
            '',
            'The addresses allowed to revoke are:',
            '* the airspace authority, always',
            '* the regulator that issued the permit, always',
            '* any address, if the airspace is restricted and the regulator license',
            'has been revoked',
            'The only addresses that can revoke a permit are either the regulator that',
            'created the permit, or the airspace authority.'
          ];
        },
        {
          name: 'airspace';
          isMut: false;
          isSigner: false;
          docs: ['The airspace the permit is to be revoked from'];
        },
        {
          name: 'issuerId';
          isMut: false;
          isSigner: false;
          docs: ['The identity account for the regulator that issued the permit'];
        },
        {
          name: 'permit';
          isMut: true;
          isSigner: false;
          docs: ['The airspace account to be created'];
        }
      ];
      args: [];
    }
  ];
  accounts: [
    {
      name: 'Airspace';
      docs: ['The isolation boundary for protocol resources'];
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'authority';
            docs: ['The address allowed to make administrative changes to this airspace.'];
            type: 'publicKey';
          },
          {
            name: 'isRestricted';
            docs: [
              'If true, resources within the airspace should be restricted to only users that receive',
              'permission from an authorized regulator. If false, any user may request a permit without',
              'the need for any authorization.'
            ];
            type: 'bool';
          }
        ];
      };
    },
    {
      name: 'AirspacePermitIssuerId';
      docs: [
        'Permission for an address to issue permits to other addresses to interact with resources',
        'in an airspace.'
      ];
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'airspace';
            docs: ['The relevant airspace for this regulator'];
            type: 'publicKey';
          },
          {
            name: 'issuer';
            docs: ['The address authorized to sign permits allowing users to create accounts', 'within the airspace'];
            type: 'publicKey';
          }
        ];
      };
    },
    {
      name: 'AirspacePermit';
      docs: ['A permission given to a user address that enables them to use resources within an airspace.'];
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'airspace';
            docs: ['The address of the `Airspace` this permit applies to'];
            type: 'publicKey';
          },
          {
            name: 'owner';
            docs: [
              'The owner of this permit, which is the address allowed to sign for any interactions',
              'with resources within the airspace (e.g. margin accounts, lending pools, etc)'
            ];
            type: 'publicKey';
          },
          {
            name: 'issuer';
            docs: ['The issuer of this permit'];
            type: 'publicKey';
          }
        ];
      };
    },
    {
      name: 'GovernorId';
      docs: ['A global account specifying the current governing address for the protocol'];
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'governor';
            docs: [
              'The governing address, which as authority to make configuration changes',
              'to the protocol, including all airspaces.'
            ];
            type: 'publicKey';
          }
        ];
      };
    }
  ];
  events: [
    {
      name: 'AirspaceCreated';
      fields: [
        {
          name: 'airspace';
          type: 'publicKey';
          index: false;
        },
        {
          name: 'seed';
          type: 'string';
          index: false;
        },
        {
          name: 'authority';
          type: 'publicKey';
          index: false;
        },
        {
          name: 'isRestricted';
          type: 'bool';
          index: false;
        }
      ];
    },
    {
      name: 'AirspaceAuthoritySet';
      fields: [
        {
          name: 'airspace';
          type: 'publicKey';
          index: false;
        },
        {
          name: 'authority';
          type: 'publicKey';
          index: false;
        }
      ];
    },
    {
      name: 'AirspaceIssuerIdCreated';
      fields: [
        {
          name: 'airspace';
          type: 'publicKey';
          index: false;
        },
        {
          name: 'issuer';
          type: 'publicKey';
          index: false;
        }
      ];
    },
    {
      name: 'AirspaceIssuerIdRevoked';
      fields: [
        {
          name: 'airspace';
          type: 'publicKey';
          index: false;
        },
        {
          name: 'issuer';
          type: 'publicKey';
          index: false;
        }
      ];
    },
    {
      name: 'AirspacePermitCreated';
      fields: [
        {
          name: 'airspace';
          type: 'publicKey';
          index: false;
        },
        {
          name: 'issuer';
          type: 'publicKey';
          index: false;
        },
        {
          name: 'owner';
          type: 'publicKey';
          index: false;
        }
      ];
    },
    {
      name: 'AirspacePermitRevoked';
      fields: [
        {
          name: 'airspace';
          type: 'publicKey';
          index: false;
        },
        {
          name: 'permit';
          type: 'publicKey';
          index: false;
        }
      ];
    }
  ];
  errors: [
    {
      code: 707000;
      name: 'PermissionDenied';
      msg: 'The signer does not have the required permissions to do this';
    }
  ];
  metadata: {
    address: 'JPASMkxARMmbeahk37H8PAAP1UzPNC4wGhvwLnBsfHi';
  };
};
