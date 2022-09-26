export declare type JetControl = {
    version: "0.1.0";
    name: "jet_control";
    constants: [
        {
            name: "FEE_DESTINATION";
            type: {
                defined: "&[u8]";
            };
            value: 'b"margin-pool-fee-destination"';
        }
    ];
    instructions: [
        {
            name: "createAuthority";
            accounts: [
                {
                    name: "authority";
                    isMut: true;
                    isSigner: false;
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
            args: [];
        },
        {
            name: "createMarginPool";
            accounts: [
                {
                    name: "requester";
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: "authority";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "marginPool";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "vault";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "depositNoteMint";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "loanNoteMint";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "tokenMint";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "tokenMetadata";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "depositNoteMetadata";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "loanNoteMetadata";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "feeDestination";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "marginPoolProgram";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "metadataProgram";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "tokenProgram";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "systemProgram";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "rent";
                    isMut: false;
                    isSigner: false;
                }
            ];
            args: [];
        },
        {
            name: "registerAdapter";
            accounts: [
                {
                    name: "requester";
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: "authority";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "adapter";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "metadataAccount";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "payer";
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: "metadataProgram";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "systemProgram";
                    isMut: false;
                    isSigner: false;
                }
            ];
            args: [];
        },
        {
            name: "configureMarginPool";
            accounts: [
                {
                    name: "requester";
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: "authority";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "tokenMint";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "marginPool";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "tokenMetadata";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "depositMetadata";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "loanMetadata";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "pythProduct";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "pythPrice";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "marginPoolProgram";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "metadataProgram";
                    isMut: false;
                    isSigner: false;
                }
            ];
            args: [
                {
                    name: "metadata";
                    type: {
                        option: {
                            defined: "TokenMetadataParams";
                        };
                    };
                },
                {
                    name: "poolConfig";
                    type: {
                        option: {
                            defined: "MarginPoolConfig";
                        };
                    };
                }
            ];
        },
        {
            name: "setLiquidator";
            accounts: [
                {
                    name: "requester";
                    isMut: false;
                    isSigner: true;
                },
                {
                    name: "authority";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "liquidator";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "metadataAccount";
                    isMut: true;
                    isSigner: false;
                },
                {
                    name: "payer";
                    isMut: true;
                    isSigner: true;
                },
                {
                    name: "metadataProgram";
                    isMut: false;
                    isSigner: false;
                },
                {
                    name: "systemProgram";
                    isMut: false;
                    isSigner: false;
                }
            ];
            args: [
                {
                    name: "isLiquidator";
                    type: "bool";
                }
            ];
        }
    ];
    accounts: [
        {
            name: "authority";
            type: {
                kind: "struct";
                fields: [
                    {
                        name: "seed";
                        type: {
                            array: ["u8", 1];
                        };
                    }
                ];
            };
        }
    ];
    types: [
        {
            name: "TokenMetadataParams";
            type: {
                kind: "struct";
                fields: [
                    {
                        name: "tokenKind";
                        type: {
                            defined: "TokenKind";
                        };
                    },
                    {
                        name: "collateralWeight";
                        type: "u16";
                    },
                    {
                        name: "maxLeverage";
                        type: "u16";
                    }
                ];
            };
        },
        {
            name: "MarginPoolParams";
            type: {
                kind: "struct";
                fields: [
                    {
                        name: "feeDestination";
                        type: "publicKey";
                    }
                ];
            };
        },
        {
            name: "TokenKind";
            type: {
                kind: "enum";
                variants: [
                    {
                        name: "NonCollateral";
                    },
                    {
                        name: "Collateral";
                    },
                    {
                        name: "Claim";
                    }
                ];
            };
        },
        {
            name: "MarginPoolConfig";
            type: {
                kind: "struct";
                fields: [
                    {
                        name: "flags";
                        type: "u64";
                    },
                    {
                        name: "utilizationRate1";
                        type: "u16";
                    },
                    {
                        name: "utilizationRate2";
                        type: "u16";
                    },
                    {
                        name: "borrowRate0";
                        type: "u16";
                    },
                    {
                        name: "borrowRate1";
                        type: "u16";
                    },
                    {
                        name: "borrowRate2";
                        type: "u16";
                    },
                    {
                        name: "borrowRate3";
                        type: "u16";
                    },
                    {
                        name: "managementFeeRate";
                        type: "u16";
                    },
                    {
                        name: "managementFeeCollectThreshold";
                        type: "u64";
                    }
                ];
            };
        },
        {
            name: "LiquidatorMetadata";
            type: {
                kind: "struct";
                fields: [
                    {
                        name: "liquidator";
                        type: "publicKey";
                    }
                ];
            };
        },
        {
            name: "MarginAdapterMetadata";
            type: {
                kind: "struct";
                fields: [
                    {
                        name: "adapterProgram";
                        type: "publicKey";
                    }
                ];
            };
        },
        {
            name: "TokenMetadata";
            type: {
                kind: "struct";
                fields: [
                    {
                        name: "tokenMint";
                        type: "publicKey";
                    },
                    {
                        name: "pythPrice";
                        type: "publicKey";
                    },
                    {
                        name: "pythProduct";
                        type: "publicKey";
                    }
                ];
            };
        },
        {
            name: "PositionTokenMetadata";
            type: {
                kind: "struct";
                fields: [
                    {
                        name: "positionTokenMint";
                        type: "publicKey";
                    },
                    {
                        name: "underlyingTokenMint";
                        type: "publicKey";
                    },
                    {
                        name: "adapterProgram";
                        type: "publicKey";
                    },
                    {
                        name: "tokenKind";
                        type: {
                            defined: "TokenKind";
                        };
                    },
                    {
                        name: "valueModifer";
                        type: "u16";
                    },
                    {
                        name: "maxStaleness";
                        type: "u64";
                    }
                ];
            };
        }
    ];
    events: [
        {
            name: "AuthorityCreated";
            fields: [
                {
                    name: "authority";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "seed";
                    type: "u8";
                    index: false;
                }
            ];
        },
        {
            name: "LiquidatorSet";
            fields: [
                {
                    name: "requester";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "authority";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "liquidatorMetadata";
                    type: {
                        defined: "LiquidatorMetadata";
                    };
                    index: false;
                },
                {
                    name: "metadataAccount";
                    type: "publicKey";
                    index: false;
                }
            ];
        },
        {
            name: "AdapterRegistered";
            fields: [
                {
                    name: "requester";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "authority";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "adapter";
                    type: {
                        defined: "MarginAdapterMetadata";
                    };
                    index: false;
                },
                {
                    name: "metadataAccount";
                    type: "publicKey";
                    index: false;
                }
            ];
        },
        {
            name: "TokenMetadataConfigured";
            fields: [
                {
                    name: "requester";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "authority";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "metadataAccount";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "metadata";
                    type: {
                        defined: "TokenMetadata";
                    };
                    index: false;
                }
            ];
        },
        {
            name: "PositionTokenMetadataConfigured";
            fields: [
                {
                    name: "requester";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "authority";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "metadataAccount";
                    type: "publicKey";
                    index: false;
                },
                {
                    name: "metadata";
                    type: {
                        defined: "PositionTokenMetadata";
                    };
                    index: false;
                }
            ];
        }
    ];
};
export declare const IDL: JetControl;
//# sourceMappingURL=jetControl.d.ts.map