export type JetMargin = {
  version: "1.0.0"
  name: "jet_margin"
  docs: [
    "This crate documents the instructions used in the `margin` program of the",
    "[jet-v2 repo](https://github.com/jet-lab/jet-v2/).",
    "",
    "Handler functions are described for each instruction well as struct parameters",
    "(and their types and descriptions are listed) and any handler function",
    "parameters aside from parameters that exist in every instruction handler function.",
    "",
    "Accounts associated with events emitted for the purposes of data logging are also included."
  ]
  constants: [
    {
      name: "TOKEN_CONFIG_SEED"
      type: "bytes"
      value: "[116, 111, 107, 101, 110, 45, 99, 111, 110, 102, 105, 103]"
    },
    {
      name: "ADAPTER_CONFIG_SEED"
      type: "bytes"
      value: "[97, 100, 97, 112, 116, 101, 114, 45, 99, 111, 110, 102, 105, 103]"
    },
    {
      name: "LIQUIDATOR_CONFIG_SEED"
      type: {
        defined: "&[u8]"
      }
      value: "PERMIT_SEED"
    },
    {
      name: "PERMIT_SEED"
      type: "bytes"
      value: "[112, 101, 114, 109, 105, 116]"
    },
    {
      name: "MAX_ORACLE_CONFIDENCE"
      type: "u16"
      value: "5_00"
    },
    {
      name: "MAX_ORACLE_STALENESS"
      type: "i64"
      value: "30"
    },
    {
      name: "MAX_PRICE_QUOTE_AGE"
      type: "u64"
      value: "30"
    },
    {
      name: "LIQUIDATION_TIMEOUT"
      type: {
        defined: "UnixTimestamp"
      }
      value: "60"
    }
  ]
  instructions: [
    {
      name: "createAccount"
      docs: [
        "Create a new margin account for a user",
        "",
        "# Parameters",
        "",
        "* `seed` - An abritrary integer used to derive the new account address. This allows",
        "a user to own multiple margin accounts, by creating new accounts with different",
        "seed values.",
        "",
        "# [Accounts](jet_margin::accounts::CreateAccount)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `owner` | `signer` | The owner of the new margin account. |",
        "| `payer` | `signer` | The pubkey paying rent for the new margin account opening. |",
        "| `margin_account` | `writable` | The margin account to initialize for the owner. |",
        "| `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AccountCreated`] | Marks the creation of the account. |"
      ]
      accounts: [
        {
          name: "owner"
          isMut: false
          isSigner: true
          docs: ["The owner of the new margin account"]
        },
        {
          name: "permit"
          isMut: false
          isSigner: false
          docs: ["A permission given to a user address that enables them to use resources within an airspace."]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
        },
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account to initialize for the owner"]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "seed"
          type: "u16"
        }
      ]
    },
    {
      name: "closeAccount"
      docs: [
        "Close a user's margin account",
        "",
        "The margin account must have zero positions remaining to be closed.",
        "",
        "# [Accounts](jet_margin::accounts::CloseAccount)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `owner` | `signer` | The owner of the account being closed. |",
        "| `receiver` | `writable` | The account to get any returned rent. |",
        "| `margin_account` | `writable` | The account being closed. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AccountClosed`] | Marks the closure of the account. |"
      ]
      accounts: [
        {
          name: "owner"
          isMut: false
          isSigner: true
          docs: ["The owner of the account being closed"]
        },
        {
          name: "receiver"
          isMut: true
          isSigner: false
          docs: ["The account to get any returned rent"]
        },
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The account being closed"]
        }
      ]
      args: []
    },
    {
      name: "registerPosition"
      docs: [
        "Register a position for deposits of tokens returned by adapter programs (e.g. margin-pool).",
        "",
        "This will create a token account to hold the adapter provided tokens which represent",
        "a user's deposit with that adapter.",
        "",
        "This instruction may fail if the account has reached it's maximum number of positions.",
        "",
        "# [Accounts](jet_margin::accounts::RegisterPosition)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `authority` | `signer` | The authority that can change the margin account. |",
        "| `payer` | `signer` | The address paying for rent. |",
        "| `margin_account` | `writable` |  The margin account to register position type with. |",
        "| `position_token_mint` | `read_only` | The mint for the position token being registered. |",
        "| `metadata` | `read_only` | The metadata account that references the correct oracle for the token. |",
        "| `token_account` | `writable` | The token account to store hold the position assets in the custody of the margin account. |",
        "| `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |",
        "| `rent` | `read_only` | The [rent sysvar](https://docs.solana.com/developing/runtime-facilities/sysvars#rent). The rent to open the account. |",
        "| `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionRegistered`] | Marks the registration of the position. |"
      ]
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that can change the margin account"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["The address paying for rent"]
        },
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account to register position type with"]
        },
        {
          name: "positionTokenMint"
          isMut: false
          isSigner: false
          docs: ["The mint for the position token being registered"]
        },
        {
          name: "config"
          isMut: false
          isSigner: false
          docs: ["The margin config for the token"]
        },
        {
          name: "tokenAccount"
          isMut: true
          isSigner: false
          docs: ["The token account to store hold the position assets in the custody of the", "margin account."]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
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
        }
      ]
      args: []
    },
    {
      name: "updatePositionBalance"
      docs: [
        "Update the balance of a position stored in the margin account to match the actual",
        "stored by the SPL token account.",
        "",
        "When a user deposits tokens directly (without invoking this program), there's no",
        "update within the user's margin account to account for the new token balance. This",
        "instruction allows udating the margin account state to reflect the current available",
        "balance of collateral.",
        "",
        "# [Accounts](jet_margin::accounts::UpdatePositionBalance)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `margin_account` | `writable` | The margin account to update. |",
        "| `token_account` | `read_only` | The token account to update the balance for. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionBalanceUpdated`] | Marks the updating of the position balance. |",
        ""
      ]
      accounts: [
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The account to update"]
        },
        {
          name: "tokenAccount"
          isMut: false
          isSigner: false
          docs: ["The token account to update the balance for"]
        }
      ]
      args: []
    },
    {
      name: "refreshPositionMetadata"
      docs: [
        "Update the metadata for a position stored in the margin account,",
        "in the case where the metadata has changed after the position was",
        "created.",
        "",
        "# [Accounts](jet_margin::accounts::RefreshPositionMetadata)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `margin_account` | `writable` | The margin account with the position to be refreshed. |",
        "| `metadata` | `read_only` | The metadata account for the token, which has been updated. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionMetadataRefreshed`] | Marks the refreshing of position metadata. |",
        ""
      ]
      accounts: [
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account with the position to be refreshed"]
        },
        {
          name: "metadata"
          isMut: false
          isSigner: false
          docs: ["The metadata account for the token, which has been updated"]
        },
        {
          name: "permit"
          isMut: false
          isSigner: false
          docs: ["permit that authorizes the refresher"]
        },
        {
          name: "refresher"
          isMut: false
          isSigner: true
          docs: ["account that is authorized to refresh position metadata"]
        }
      ]
      args: []
    },
    {
      name: "closePosition"
      docs: [
        "Close out a position, removing it from the account.",
        "",
        "Since there is a finite number of positions a single account can maintain it may be",
        "necessary for a user to close out old positions to take new ones.",
        "",
        "# [Accounts](jet_margin::accounts::ClosePosition)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `authority` | `signer` | The authority that can change the margin account. |",
        "| `receiver` | `writable` | The receiver for the rent released. |",
        "| `margin_account` | `writable` | The margin account with the position to close. |",
        "| `position_token_mint` | `read_only` | The mint for the position token being deregistered. |",
        "| `token_account` | `writable` | The token account for the position being closed. |",
        "| `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionClosed`] | Marks the closure of the position. |",
        ""
      ]
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that can change the margin account"]
        },
        {
          name: "receiver"
          isMut: true
          isSigner: false
          docs: ["The receiver for the rent released"]
        },
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account with the position to close"]
        },
        {
          name: "positionTokenMint"
          isMut: false
          isSigner: false
          docs: ["The mint for the position token being deregistered"]
        },
        {
          name: "tokenAccount"
          isMut: true
          isSigner: false
          docs: ["The token account for the position being closed"]
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
      name: "verifyHealthy"
      docs: [
        "Verify that the account is healthy, by validating the collateralization",
        "ration is above the minimum.",
        "",
        "There's no real reason to call this instruction, outside of wanting to simulate",
        "the health check for a margin account.",
        "",
        "",
        "# [Accounts](jet_margin::accounts::VerifyHealthy)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `margin_account` | `read_only` | The account to verify the health of. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::VerifiedHealthy`] | Marks the verification of the position. |",
        ""
      ]
      accounts: [
        {
          name: "marginAccount"
          isMut: false
          isSigner: false
          docs: ["The account verify the health of"]
        }
      ]
      args: []
    },
    {
      name: "adapterInvoke"
      docs: [
        "Perform an action by invoking other programs, allowing them to alter",
        "the balances of the token accounts belonging to this margin account.",
        "",
        "This provides the margin account as a signer to any invoked instruction, and therefore",
        "grants the adapter authority over any tokens held by the margin account.",
        "",
        "This validates the invoked program by expecting an `adapter_metadata` account,",
        "which must exist for the instruction to be considered valid. The configuration",
        "for allowing adapter programs is controlled by protocol governance.",
        "",
        "All extra accounts passed in are used as the input accounts when invoking",
        "the provided adapter porgram.",
        "",
        "# Parameters",
        "",
        "* `data` - The instruction data to pass to the adapter program",
        "",
        "# [Accounts](jet_margin::accounts::AdapterInvoke)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `owner` | `signer` | The authority that owns the margin account. |",
        "| `margin_account` | `writable` | The margin account to proxy an action for. |",
        "| `adapter_program` | `read_only` | The program to be invoked. |",
        "| `adapter_metadata` | `read_only` | The metadata about the proxy program. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AdapterInvokeBegin`] | Marks the start of the adapter invocation (includes the margin account pubkey and the adapter program pubkey). |",
        "| [`events::PositionEvent`] _(Note that each single event represents a different adapter position)_ | The [PositionEvent](events::PositionEvent) marks the change in position. |",
        "| [`events::AdapterInvokeEnd`] | Marks the ending of the adapter invocation (includes no data except for the event itself being emitted). |"
      ]
      accounts: [
        {
          name: "owner"
          isMut: false
          isSigner: true
          docs: ["The authority that owns the margin account"]
        },
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account to proxy an action for"]
        },
        {
          name: "adapterProgram"
          isMut: false
          isSigner: false
          docs: ["The program to be invoked"]
        },
        {
          name: "adapterConfig"
          isMut: false
          isSigner: false
          docs: ["The metadata about the proxy program"]
        }
      ]
      args: [
        {
          name: "data"
          type: "bytes"
        }
      ]
    },
    {
      name: "accountingInvoke"
      docs: [
        "Perform an action by invoking other programs, allowing them only to",
        "refresh the state of the margin account to be consistent with the actual",
        "underlying prices or positions, but not permitting new position changes.",
        "",
        "This is a permissionless way of updating the value of positions on a margin",
        "account which require some adapter to provide the update. Unlike `adapter_invoke`,",
        "this instruction will not provider the margin account as a signer to invoked programs,",
        "and they thefore do not have authority to modify any token balances held by the account.",
        "",
        "All extra accounts passed in are used as the input accounts when invoking",
        "the provided adapter porgram.",
        "",
        "# Parameters",
        "",
        "* `data` - The instruction data to pass to the adapter program",
        "",
        "# [Accounts](jet_margin::accounts::AccountingInvoke)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** |  **Description** |",
        "| `margin_account` | `writable` | The margin account to proxy an action for. |",
        "| `adapter_program` | `read_only` | The program to be invoked. |",
        "| `adapter_metadata` | `read_only` | The metadata about the proxy program. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| [`events::AccountingInvokeBegin`] | Signify that the accounting invocation process has begun. |",
        "| [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | The [PositionEvent](events::PositionEvent) marks the change in position. |",
        "| [`events::AccountingInvokeEnd`] | Signify that the accounting invocation process has ended. |"
      ]
      accounts: [
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account to proxy an action for"]
        },
        {
          name: "adapterProgram"
          isMut: false
          isSigner: false
          docs: ["The program to be invoked"]
        },
        {
          name: "adapterConfig"
          isMut: false
          isSigner: false
          docs: ["The metadata about the proxy program"]
        }
      ]
      args: [
        {
          name: "data"
          type: "bytes"
        }
      ]
    },
    {
      name: "liquidateBegin"
      docs: [
        "Begin liquidating an account",
        "",
        "The account will enter a state preventing the owner from taking any action,",
        "until the liquidator process is complete.",
        "",
        "Requires the `liquidator_metadata` account, which restricts the signer to",
        "those approved by protocol governance.",
        "",
        "# [Accounts](jet_margin::accounts::LiquidateBegin)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `margin_account` | `writable` | The account in need of liquidation. |",
        "| `payer` | `signer` | The address paying rent. |",
        "| `liquidator` | `signer` | The liquidator account performing the liquidation. |",
        "| `liquidator_metadata` | `read_only` | The metadata describing the liquidator. |",
        "| `liquidation` | `writable` | The account to persist the state of liquidation. |",
        "| `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidationBegun`] | Marks the beginning of the liquidation. |"
      ]
      accounts: [
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The account in need of liquidation"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["The address paying rent"]
        },
        {
          name: "liquidator"
          isMut: false
          isSigner: true
          docs: ["The liquidator account performing the liquidation actions"]
        },
        {
          name: "permit"
          isMut: false
          isSigner: false
          docs: ["The permit allowing the liquidator to do this"]
        },
        {
          name: "liquidation"
          isMut: true
          isSigner: false
          docs: ["Account to persist the state of the liquidation"]
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
      name: "liquidateEnd"
      docs: [
        "End the liquidation state for an account",
        "",
        "Normally must be signed by the liquidator that started the liquidation state. Can be",
        "signed by anyone after the [timeout period](jet_margin::LIQUIDATION_TIMEOUT) has elapsed.",
        "",
        "# [Accounts](jet_margin::accounts::LiquidateEnd)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `authority` | `signer` | The pubkey calling the instruction to end liquidation. |",
        "| `margin_account` | `writable` | The account in need of liquidation. |",
        "| `liquidation` | `writable` | The account to persist the state of liquidation. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidationEnded`] | Marks the ending of the liquidation. |"
      ]
      accounts: [
        {
          name: "authority"
          isMut: true
          isSigner: true
          docs: [
            "If the liquidation is timed out, this can be any account",
            "If the liquidation is not timed out, this must be the liquidator, and it must be a signer"
          ]
        },
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The account in need of liquidation"]
        },
        {
          name: "liquidation"
          isMut: true
          isSigner: false
          docs: ["Account to persist the state of the liquidation"]
        }
      ]
      args: []
    },
    {
      name: "liquidatorInvoke"
      docs: [
        "Perform an action by invoking another program, for the purposes of",
        "liquidating a margin account.",
        "",
        "Requires the account already be in the liquidation state, and the signer must",
        "be the same liquidator that started the liquidation state.",
        "",
        "# [Accounts](jet_margin::accounts::LiquidatorInvoke)",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `liquidator` | `signer` | The liquidator processing the margin account. |",
        "| `liquidation` | `writable` | The account to persist the state of liquidation. |",
        "| `margin_account` | `writable` | The margin account to proxy an action for. |",
        "| `adapter_program` | `read_only` | The program to be invoked. |",
        "| `adapter_metadata` | `read_only` | The metadata about the proxy program. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidatorInvokeBegin`] | Marks the beginning of this liquidation event. |",
        "| [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | The [PositionEvent](events::PositionEvent) describing the change in position. |",
        "| [`events::LiquidatorInvokeEnd`] | Marks the ending of this liquidator event. |"
      ]
      accounts: [
        {
          name: "liquidator"
          isMut: false
          isSigner: true
          docs: ["The liquidator processing the margin account"]
        },
        {
          name: "liquidation"
          isMut: true
          isSigner: false
          docs: ["Account to persist the state of the liquidation"]
        },
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account to proxy an action for"]
        },
        {
          name: "adapterProgram"
          isMut: false
          isSigner: false
          docs: ["The program to be invoked"]
        },
        {
          name: "adapterConfig"
          isMut: false
          isSigner: false
          docs: ["The metadata about the proxy program"]
        }
      ]
      args: [
        {
          name: "data"
          type: "bytes"
        }
      ]
    },
    {
      name: "refreshPositionConfig"
      docs: [
        "Update the config for a token position stored in the margin account,",
        "in the case where the token config has changed after the position was",
        "created."
      ]
      accounts: [
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account with the position to be refreshed"]
        },
        {
          name: "config"
          isMut: false
          isSigner: false
          docs: ["The config account for the token, which has been updated"]
        },
        {
          name: "permit"
          isMut: false
          isSigner: false
          docs: ["permit that authorizes the refresher"]
        },
        {
          name: "refresher"
          isMut: false
          isSigner: true
          docs: ["account that is authorized to refresh position metadata"]
        }
      ]
      args: []
    },
    {
      name: "refreshDepositPosition"
      docs: ["Refresh the price/balance for a deposit position"]
      accounts: [
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The account to update"]
        },
        {
          name: "config"
          isMut: false
          isSigner: false
          docs: ["The margin config for the token"]
        },
        {
          name: "priceOracle"
          isMut: false
          isSigner: false
          docs: ["The oracle for the token"]
        }
      ]
      args: []
    },
    {
      name: "createDepositPosition"
      docs: ["Create a new account for holding SPL token deposits directly by a margin account."]
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority that can change the margin account"]
        },
        {
          name: "payer"
          isMut: true
          isSigner: true
          docs: ["The address paying for rent"]
        },
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account to register this deposit account with"]
        },
        {
          name: "mint"
          isMut: false
          isSigner: false
          docs: ["The mint for the token being stored in this account"]
        },
        {
          name: "config"
          isMut: false
          isSigner: false
          docs: ["The margin config for the token"]
        },
        {
          name: "tokenAccount"
          isMut: false
          isSigner: false
          docs: ["The token account to store deposits"]
        },
        {
          name: "associatedTokenProgram"
          isMut: false
          isSigner: false
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
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
        }
      ]
      args: []
    },
    {
      name: "transferDeposit"
      docs: ["Transfer tokens into or out of a token account being used for deposits."]
      accounts: [
        {
          name: "owner"
          isMut: false
          isSigner: true
          docs: ["The authority that owns the margin account"]
        },
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account that the deposit account is associated with"]
        },
        {
          name: "sourceOwner"
          isMut: false
          isSigner: false
          docs: ["The authority for the source account"]
        },
        {
          name: "source"
          isMut: true
          isSigner: false
          docs: ["The source account to transfer tokens from"]
        },
        {
          name: "destination"
          isMut: true
          isSigner: false
          docs: ["The destination account to transfer tokens in"]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
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
      name: "configureToken"
      docs: [
        "Set the configuration for a token, which allows it to be used as a position in a margin",
        "account.",
        "",
        "The configuration for a token only applies for the associated airspace, and changing any",
        "configuration requires the airspace authority to sign.",
        "",
        "The account storing the configuration will be funded if not already. If a `None` is provided as",
        "the updated configuration, then the account will be defunded."
      ]
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority allowed to make changes to configuration"]
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
          docs: ["The payer for any rent costs, if required"]
        },
        {
          name: "mint"
          isMut: false
          isSigner: false
          docs: ["The mint for the token being configured"]
        },
        {
          name: "tokenConfig"
          isMut: true
          isSigner: false
          docs: ["The config account to be modified"]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "update"
          type: {
            option: {
              defined: "TokenConfigUpdate"
            }
          }
        }
      ]
    },
    {
      name: "configureAdapter"
      docs: [
        "Set the configuration for an adapter.",
        "",
        "The configuration for a token only applies for the associated airspace, and changing any",
        "configuration requires the airspace authority to sign.",
        "",
        "The account storing the configuration will be funded if not already. If a `None` is provided as",
        "the updated configuration, then the account will be defunded."
      ]
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority allowed to make changes to configuration"]
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
          docs: ["The payer for any rent costs, if required"]
        },
        {
          name: "adapterProgram"
          isMut: false
          isSigner: false
          docs: ["The adapter being configured"]
        },
        {
          name: "adapterConfig"
          isMut: true
          isSigner: false
          docs: ["The config account to be modified"]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "isAdapter"
          type: "bool"
        }
      ]
    },
    {
      name: "configureLiquidator"
      docs: [
        "Set the configuration for a liquidator.",
        "",
        "The configuration for a token only applies for the associated airspace, and changing any",
        "configuration requires the airspace authority to sign.",
        "",
        "The account storing the configuration will be funded if not already. If a `None` is provided as",
        "the updated configuration, then the account will be defunded."
      ]
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority allowed to make changes to configuration"]
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
          docs: ["The payer for any rent costs, if required"]
        },
        {
          name: "owner"
          isMut: false
          isSigner: false
          docs: ["The owner being configured"]
        },
        {
          name: "permit"
          isMut: true
          isSigner: false
          docs: ["The config account to be modified"]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "isLiquidator"
          type: "bool"
        }
      ]
    },
    {
      name: "configurePositionConfigRefresher"
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The authority allowed to make changes to configuration"]
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
          docs: ["The payer for any rent costs, if required"]
        },
        {
          name: "owner"
          isMut: false
          isSigner: false
          docs: ["The owner being configured"]
        },
        {
          name: "permit"
          isMut: true
          isSigner: false
          docs: ["The config account to be modified"]
        },
        {
          name: "systemProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "mayRefresh"
          type: "bool"
        }
      ]
    },
    {
      name: "adminTransferPosition"
      docs: [
        "Allow governing address to transfer any position from one margin account to another",
        "",
        "This is provided as a mechanism to allow for manually fixing issues that occur in the",
        "protocol due to bad user assets."
      ]
      accounts: [
        {
          name: "authority"
          isMut: false
          isSigner: true
          docs: ["The administrative authority"]
        },
        {
          name: "targetAccount"
          isMut: true
          isSigner: false
          docs: ["The target margin account to move a position into"]
        },
        {
          name: "sourceAccount"
          isMut: true
          isSigner: false
          docs: ["The source account to move a position out of"]
        },
        {
          name: "sourceTokenAccount"
          isMut: true
          isSigner: false
          docs: ["The token account to be moved from"]
        },
        {
          name: "targetTokenAccount"
          isMut: true
          isSigner: false
          docs: ["The token account to be moved into"]
        },
        {
          name: "tokenProgram"
          isMut: false
          isSigner: false
        }
      ]
      args: [
        {
          name: "amount"
          type: "u64"
        }
      ]
    }
  ]
  accounts: [
    {
      name: "marginAccount"
      type: {
        kind: "struct"
        fields: [
          {
            name: "version"
            type: "u8"
          },
          {
            name: "bumpSeed"
            type: {
              array: ["u8", 1]
            }
          },
          {
            name: "userSeed"
            type: {
              array: ["u8", 2]
            }
          },
          {
            name: "invocation"
            docs: [
              "Data an adapter can use to check what the margin program thinks about the current invocation",
              "Must normally be zeroed, except during an invocation."
            ]
            type: {
              defined: "Invocation"
            }
          },
          {
            name: "reserved0"
            type: {
              array: ["u8", 3]
            }
          },
          {
            name: "owner"
            docs: ["The owner of this account, which generally has to sign for any changes to it"]
            type: "publicKey"
          },
          {
            name: "airspace"
            docs: ["The airspace this account belongs to"]
            type: "publicKey"
          },
          {
            name: "liquidator"
            docs: ["The active liquidator for this account"]
            type: "publicKey"
          },
          {
            name: "positions"
            docs: ["The storage for tracking account balances"]
            type: {
              array: ["u8", 7432]
            }
          }
        ]
      }
    },
    {
      name: "liquidationState"
      docs: ["State of an in-progress liquidation"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "liquidator"
            docs: ["The signer responsible for liquidation"]
            type: "publicKey"
          },
          {
            name: "marginAccount"
            docs: ["The margin account being liquidated"]
            type: "publicKey"
          },
          {
            name: "state"
            docs: ["The state object"]
            type: {
              defined: "Liquidation"
            }
          }
        ]
      }
    },
    {
      name: "tokenConfig"
      docs: [
        "The configuration account specifying parameters for a token when used",
        "in a position within a margin account."
      ]
      type: {
        kind: "struct"
        fields: [
          {
            name: "mint"
            docs: ["The mint for the token"]
            type: "publicKey"
          },
          {
            name: "underlyingMint"
            docs: ["The mint for the underlying token represented, if any"]
            type: "publicKey"
          },
          {
            name: "airspace"
            docs: ["The space this config is valid within"]
            type: "publicKey"
          },
          {
            name: "admin"
            docs: [
              "The administrator of this token, which has the authority to provide information",
              "about (e.g. prices) and otherwise modify position states for these tokens."
            ]
            type: {
              defined: "TokenAdmin"
            }
          },
          {
            name: "tokenKind"
            docs: [
              "Description of this token",
              "",
              "This determines the way the margin program values a token as a position in a",
              "margin account."
            ]
            type: {
              defined: "TokenKind"
            }
          },
          {
            name: "valueModifier"
            docs: ["A modifier to adjust the token value, based on the kind of token"]
            type: "u16"
          },
          {
            name: "maxStaleness"
            docs: ["The maximum staleness (seconds) that's acceptable for balances of this token"]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "permit"
      docs: ["Configuration enabling a signer to execute permissioned actions"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "airspace"
            docs: ["Airspace where the permit is valid."]
            type: "publicKey"
          },
          {
            name: "owner"
            docs: ["Address which may sign to perform the permitted actions."]
            type: "publicKey"
          },
          {
            name: "permissions"
            docs: ["Actions which may be performed with the signature of the owner."]
            type: {
              defined: "Permissions"
            }
          }
        ]
      }
    },
    {
      name: "adapterConfig"
      docs: ["Configuration for allowed adapters"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "airspace"
            docs: ["The airspace this adapter can be used in"]
            type: "publicKey"
          },
          {
            name: "adapterProgram"
            docs: ["The program address allowed to be called as an adapter"]
            type: "publicKey"
          }
        ]
      }
    }
  ]
  types: [
    {
      name: "AdapterResult"
      type: {
        kind: "struct"
        fields: [
          {
            name: "positionChanges"
            docs: ["keyed by token mint, same as position"]
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
      name: "PriceChangeInfo"
      type: {
        kind: "struct"
        fields: [
          {
            name: "value"
            docs: ["The current price of the asset"]
            type: "i64"
          },
          {
            name: "confidence"
            docs: ["The current confidence value for the asset price"]
            type: "u64"
          },
          {
            name: "twap"
            docs: ["The recent average price"]
            type: "i64"
          },
          {
            name: "publishTime"
            docs: ["The time that the price was published at"]
            type: "i64"
          },
          {
            name: "exponent"
            docs: ["The exponent for the price values"]
            type: "i32"
          }
        ]
      }
    },
    {
      name: "ValuationSummary"
      type: {
        kind: "struct"
        fields: [
          {
            name: "equity"
            type: "i128"
          },
          {
            name: "liabilities"
            type: "i128"
          },
          {
            name: "requiredCollateral"
            type: "i128"
          },
          {
            name: "weightedCollateral"
            type: "i128"
          },
          {
            name: "effectiveCollateral"
            type: "i128"
          },
          {
            name: "availableCollateral"
            type: "i128"
          },
          {
            name: "pastDue"
            type: "bool"
          }
        ]
      }
    },
    {
      name: "TokenConfigUpdate"
      type: {
        kind: "struct"
        fields: [
          {
            name: "underlyingMint"
            docs: ["The underlying token represented, if any"]
            type: "publicKey"
          },
          {
            name: "admin"
            docs: ["The administration authority for the token"]
            type: {
              defined: "TokenAdmin"
            }
          },
          {
            name: "tokenKind"
            docs: ["Description of this token"]
            type: {
              defined: "TokenKind"
            }
          },
          {
            name: "valueModifier"
            docs: ["A modifier to adjust the token value, based on the kind of token"]
            type: "u16"
          },
          {
            name: "maxStaleness"
            docs: ["The maximum staleness (seconds) that's acceptable for balances of this token"]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "AdapterPositionFlags"
      type: {
        kind: "struct"
        fields: [
          {
            name: "flags"
            type: "u8"
          }
        ]
      }
    },
    {
      name: "PriceInfo"
      type: {
        kind: "struct"
        fields: [
          {
            name: "value"
            docs: ["The current price"]
            type: "i64"
          },
          {
            name: "timestamp"
            docs: ["The timestamp the price was valid at"]
            type: "u64"
          },
          {
            name: "exponent"
            docs: ["The exponent for the price value"]
            type: "i32"
          },
          {
            name: "isValid"
            docs: ["Flag indicating if the price is valid for the position"]
            type: "u8"
          },
          {
            name: "reserved"
            type: {
              array: ["u8", 3]
            }
          }
        ]
      }
    },
    {
      name: "AccountPosition"
      type: {
        kind: "struct"
        fields: [
          {
            name: "token"
            docs: ["The address of the token/mint of the asset"]
            type: "publicKey"
          },
          {
            name: "address"
            docs: ["The address of the account holding the tokens."]
            type: "publicKey"
          },
          {
            name: "adapter"
            docs: ["The address of the adapter managing the asset"]
            type: "publicKey"
          },
          {
            name: "value"
            docs: ["The current value of this position, stored as a `Number128` with fixed precision."]
            type: {
              array: ["u8", 16]
            }
          },
          {
            name: "balance"
            docs: ["The amount of tokens in the account"]
            type: "u64"
          },
          {
            name: "balanceTimestamp"
            docs: ["The timestamp of the last balance update"]
            type: "u64"
          },
          {
            name: "price"
            docs: ["The current price/value of each token"]
            type: {
              defined: "PriceInfo"
            }
          },
          {
            name: "kind"
            docs: ["The kind of balance this position contains"]
            type: "u32"
          },
          {
            name: "exponent"
            docs: ["The exponent for the token value"]
            type: "i16"
          },
          {
            name: "valueModifier"
            docs: ["A weight on the value of this asset when counting collateral"]
            type: "u16"
          },
          {
            name: "maxStaleness"
            docs: ["The max staleness for the account balance (seconds)"]
            type: "u64"
          },
          {
            name: "flags"
            docs: ["Flags that are set by the adapter"]
            type: {
              defined: "AdapterPositionFlags"
            }
          },
          {
            name: "reserved"
            docs: ["Unused"]
            type: {
              array: ["u8", 23]
            }
          }
        ]
      }
    },
    {
      name: "AccountPositionKey"
      type: {
        kind: "struct"
        fields: [
          {
            name: "mint"
            docs: ["The address of the mint for the position token"]
            type: "publicKey"
          },
          {
            name: "index"
            docs: ["The array index where the data for this position is located"]
            type: "u64"
          }
        ]
      }
    },
    {
      name: "AccountPositionList"
      type: {
        kind: "struct"
        fields: [
          {
            name: "length"
            type: "u64"
          },
          {
            name: "map"
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
            name: "positions"
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
      name: "Liquidation"
      type: {
        kind: "struct"
        fields: [
          {
            name: "startTime"
            docs: ["time that liquidate_begin initialized this liquidation"]
            type: "i64"
          },
          {
            name: "equityLoss"
            docs: ["The cumulative amount of equity lost during liquidation so far"]
            type: "i128"
          },
          {
            name: "maxEquityLoss"
            docs: ["The maximum amount of collateral allowed to be lost during all steps"]
            type: "i128"
          }
        ]
      }
    },
    {
      name: "Invocation"
      type: {
        kind: "struct"
        fields: [
          {
            name: "flags"
            type: "u8"
          }
        ]
      }
    },
    {
      name: "PositionChange"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Price"
            fields: [
              {
                defined: "PriceChangeInfo"
              }
            ]
          },
          {
            name: "Flags"
            fields: [
              {
                defined: "AdapterPositionFlags"
              },
              "bool"
            ]
          },
          {
            name: "Register"
            fields: ["publicKey"]
          },
          {
            name: "Close"
            fields: ["publicKey"]
          }
        ]
      }
    },
    {
      name: "Approver"
      type: {
        kind: "enum"
        variants: [
          {
            name: "MarginAccountAuthority"
          },
          {
            name: "Adapter"
            fields: ["publicKey"]
          }
        ]
      }
    },
    {
      name: "TokenKind"
      docs: ["Description of the token's usage"]
      type: {
        kind: "enum"
        variants: [
          {
            name: "Collateral"
          },
          {
            name: "Claim"
          },
          {
            name: "AdapterCollateral"
          }
        ]
      }
    },
    {
      name: "TokenOracle"
      docs: ["Information about where to find the oracle data for a token"]
      type: {
        kind: "enum"
        variants: [
          {
            name: "Pyth"
            fields: [
              {
                name: "price"
                docs: ["The pyth address containing price information for a token."]
                type: "publicKey"
              },
              {
                name: "product"
                docs: ["The pyth address with product information for a token"]
                type: "publicKey"
              }
            ]
          }
        ]
      }
    },
    {
      name: "TokenAdmin"
      docs: ["Description of which program administers a token"]
      type: {
        kind: "enum"
        variants: [
          {
            name: "Margin"
            fields: [
              {
                name: "oracle"
                docs: ["An oracle that can be used to collect price information for a token"]
                type: {
                  defined: "TokenOracle"
                }
              }
            ]
          },
          {
            name: "Adapter"
            fields: ["publicKey"]
          }
        ]
      }
    },
    {
      name: "SyscallProvider"
      type: {
        kind: "enum"
        variants: [
          {
            name: "Mock"
            fields: [
              {
                defined: "dynFn()->T"
              }
            ]
          },
          {
            name: "SolanaRuntime"
          },
          {
            name: "Stub"
          }
        ]
      }
    }
  ]
  events: [
    {
      name: "AccountCreated"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "owner"
          type: "publicKey"
          index: false
        },
        {
          name: "seed"
          type: "u16"
          index: false
        }
      ]
    },
    {
      name: "AccountClosed"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "VerifiedHealthy"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "PositionRegistered"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "authority"
          type: "publicKey"
          index: false
        },
        {
          name: "position"
          type: {
            defined: "AccountPosition"
          }
          index: false
        }
      ]
    },
    {
      name: "PositionClosed"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "authority"
          type: "publicKey"
          index: false
        },
        {
          name: "token"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "PositionMetadataRefreshed"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "position"
          type: {
            defined: "AccountPosition"
          }
          index: false
        }
      ]
    },
    {
      name: "PositionBalanceUpdated"
      fields: [
        {
          name: "position"
          type: {
            defined: "AccountPosition"
          }
          index: false
        }
      ]
    },
    {
      name: "PositionTouched"
      fields: [
        {
          name: "position"
          type: {
            defined: "AccountPosition"
          }
          index: false
        }
      ]
    },
    {
      name: "AccountingInvokeBegin"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "adapterProgram"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "AccountingInvokeEnd"
      fields: []
    },
    {
      name: "AdapterInvokeBegin"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "adapterProgram"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "AdapterInvokeEnd"
      fields: []
    },
    {
      name: "LiquidationBegun"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "liquidator"
          type: "publicKey"
          index: false
        },
        {
          name: "liquidation"
          type: "publicKey"
          index: false
        },
        {
          name: "liquidationData"
          type: {
            defined: "Liquidation"
          }
          index: false
        },
        {
          name: "valuationSummary"
          type: {
            defined: "ValuationSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "LiquidatorInvokeBegin"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "adapterProgram"
          type: "publicKey"
          index: false
        },
        {
          name: "liquidator"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "LiquidatorInvokeEnd"
      fields: [
        {
          name: "liquidationData"
          type: {
            defined: "Liquidation"
          }
          index: false
        },
        {
          name: "valuationSummary"
          type: {
            defined: "ValuationSummary"
          }
          index: false
        }
      ]
    },
    {
      name: "LiquidationEnded"
      fields: [
        {
          name: "marginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "authority"
          type: "publicKey"
          index: false
        },
        {
          name: "timedOut"
          type: "bool"
          index: false
        }
      ]
    },
    {
      name: "TransferPosition"
      fields: [
        {
          name: "sourceMarginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "targetMarginAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "sourceTokenAccount"
          type: "publicKey"
          index: false
        },
        {
          name: "targetTokenAccount"
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
      name: "TokenConfigured"
      fields: [
        {
          name: "airspace"
          type: "publicKey"
          index: false
        },
        {
          name: "update"
          type: {
            option: {
              defined: "TokenConfigUpdate"
            }
          }
          index: false
        },
        {
          name: "mint"
          type: "publicKey"
          index: false
        }
      ]
    },
    {
      name: "AdapterConfigured"
      fields: [
        {
          name: "airspace"
          type: "publicKey"
          index: false
        },
        {
          name: "adapterProgram"
          type: "publicKey"
          index: false
        },
        {
          name: "isAdapter"
          type: "bool"
          index: false
        }
      ]
    },
    {
      name: "PermitConfigured"
      fields: [
        {
          name: "airspace"
          type: "publicKey"
          index: false
        },
        {
          name: "owner"
          type: "publicKey"
          index: false
        },
        {
          name: "permissions"
          type: "u32"
          index: false
        }
      ]
    }
  ]
  errors: [
    {
      code: 141000
      name: "NoAdapterResult"
    },
    {
      code: 141001
      name: "WrongProgramAdapterResult"
      msg: "The program that set the result was not the adapter"
    },
    {
      code: 141002
      name: "UnauthorizedInvocation"
      msg: "this invocation is not authorized by the necessary accounts"
    },
    {
      code: 141003
      name: "IndirectInvocation"
      msg: "the current instruction was not directly invoked by the margin program"
    },
    {
      code: 141010
      name: "MaxPositions"
      msg: "account cannot record any additional positions"
    },
    {
      code: 141011
      name: "UnknownPosition"
      msg: "account has no record of the position"
    },
    {
      code: 141012
      name: "CloseNonZeroPosition"
      msg: "attempting to close a position that has a balance"
    },
    {
      code: 141013
      name: "PositionAlreadyRegistered"
      msg: "attempting to register an existing position"
    },
    {
      code: 141014
      name: "AccountNotEmpty"
      msg: "attempting to close non-empty margin account"
    },
    {
      code: 141015
      name: "PositionNotRegistered"
      msg: "attempting to use unregistered position"
    },
    {
      code: 141016
      name: "CloseRequiredPosition"
      msg: "attempting to close a position that is required by the adapter"
    },
    {
      code: 141017
      name: "InvalidPositionOwner"
      msg: "registered position owner inconsistent with PositionTokenMetadata owner or token_kind"
    },
    {
      code: 141018
      name: "PositionNotRegisterable"
      msg: "dependencies are not satisfied to auto-register a required but unregistered position"
    },
    {
      code: 141020
      name: "InvalidPositionAdapter"
      msg: "wrong adapter to modify the position"
    },
    {
      code: 141021
      name: "OutdatedPrice"
      msg: "a position price is outdated"
    },
    {
      code: 141022
      name: "InvalidPrice"
      msg: "an asset price is currently invalid"
    },
    {
      code: 141023
      name: "OutdatedBalance"
      msg: "a position balance is outdated"
    },
    {
      code: 141030
      name: "Unhealthy"
      msg: "the account is not healthy"
    },
    {
      code: 141031
      name: "Healthy"
      msg: "the account is already healthy"
    },
    {
      code: 141032
      name: "Liquidating"
      msg: "the account is being liquidated"
    },
    {
      code: 141033
      name: "NotLiquidating"
      msg: "the account is not being liquidated"
    },
    {
      code: 141034
      name: "StalePositions"
    },
    {
      code: 141040
      name: "UnauthorizedLiquidator"
      msg: "the liquidator does not have permission to do this"
    },
    {
      code: 141041
      name: "LiquidationLostValue"
      msg: "attempted to extract too much value during liquidation"
    },
    {
      code: 141042
      name: "WrongLiquidationState"
      msg: "liquidationState does not match given margin account"
    },
    {
      code: 141050
      name: "WrongAirspace"
      msg: "attempting to mix entities from different airspaces"
    },
    {
      code: 141051
      name: "InvalidConfig"
      msg: "attempting to use or set invalid configuration"
    },
    {
      code: 141052
      name: "InvalidOracle"
      msg: "attempting to use or set invalid configuration"
    },
    {
      code: 141060
      name: "InsufficientPermissions"
      msg: "the permit does not authorize this action"
    },
    {
      code: 141061
      name: "PermitNotOwned"
      msg: "the permit is not owned by the current user"
    }
  ]
}

export const IDL: JetMargin = {
  version: "1.0.0",
  name: "jet_margin",
  docs: [
    "This crate documents the instructions used in the `margin` program of the",
    "[jet-v2 repo](https://github.com/jet-lab/jet-v2/).",
    "",
    "Handler functions are described for each instruction well as struct parameters",
    "(and their types and descriptions are listed) and any handler function",
    "parameters aside from parameters that exist in every instruction handler function.",
    "",
    "Accounts associated with events emitted for the purposes of data logging are also included."
  ],
  constants: [
    {
      name: "TOKEN_CONFIG_SEED",
      type: "bytes",
      value: "[116, 111, 107, 101, 110, 45, 99, 111, 110, 102, 105, 103]"
    },
    {
      name: "ADAPTER_CONFIG_SEED",
      type: "bytes",
      value: "[97, 100, 97, 112, 116, 101, 114, 45, 99, 111, 110, 102, 105, 103]"
    },
    {
      name: "LIQUIDATOR_CONFIG_SEED",
      type: {
        defined: "&[u8]"
      },
      value: "PERMIT_SEED"
    },
    {
      name: "PERMIT_SEED",
      type: "bytes",
      value: "[112, 101, 114, 109, 105, 116]"
    },
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
      docs: [
        "Create a new margin account for a user",
        "",
        "# Parameters",
        "",
        "* `seed` - An abritrary integer used to derive the new account address. This allows",
        "a user to own multiple margin accounts, by creating new accounts with different",
        "seed values.",
        "",
        "# [Accounts](jet_margin::accounts::CreateAccount)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `owner` | `signer` | The owner of the new margin account. |",
        "| `payer` | `signer` | The pubkey paying rent for the new margin account opening. |",
        "| `margin_account` | `writable` | The margin account to initialize for the owner. |",
        "| `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AccountCreated`] | Marks the creation of the account. |"
      ],
      accounts: [
        {
          name: "owner",
          isMut: false,
          isSigner: true,
          docs: ["The owner of the new margin account"]
        },
        {
          name: "permit",
          isMut: false,
          isSigner: false,
          docs: ["A permission given to a user address that enables them to use resources within an airspace."]
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true
        },
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account to initialize for the owner"]
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
      docs: [
        "Close a user's margin account",
        "",
        "The margin account must have zero positions remaining to be closed.",
        "",
        "# [Accounts](jet_margin::accounts::CloseAccount)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `owner` | `signer` | The owner of the account being closed. |",
        "| `receiver` | `writable` | The account to get any returned rent. |",
        "| `margin_account` | `writable` | The account being closed. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AccountClosed`] | Marks the closure of the account. |"
      ],
      accounts: [
        {
          name: "owner",
          isMut: false,
          isSigner: true,
          docs: ["The owner of the account being closed"]
        },
        {
          name: "receiver",
          isMut: true,
          isSigner: false,
          docs: ["The account to get any returned rent"]
        },
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account being closed"]
        }
      ],
      args: []
    },
    {
      name: "registerPosition",
      docs: [
        "Register a position for deposits of tokens returned by adapter programs (e.g. margin-pool).",
        "",
        "This will create a token account to hold the adapter provided tokens which represent",
        "a user's deposit with that adapter.",
        "",
        "This instruction may fail if the account has reached it's maximum number of positions.",
        "",
        "# [Accounts](jet_margin::accounts::RegisterPosition)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `authority` | `signer` | The authority that can change the margin account. |",
        "| `payer` | `signer` | The address paying for rent. |",
        "| `margin_account` | `writable` |  The margin account to register position type with. |",
        "| `position_token_mint` | `read_only` | The mint for the position token being registered. |",
        "| `metadata` | `read_only` | The metadata account that references the correct oracle for the token. |",
        "| `token_account` | `writable` | The token account to store hold the position assets in the custody of the margin account. |",
        "| `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |",
        "| `rent` | `read_only` | The [rent sysvar](https://docs.solana.com/developing/runtime-facilities/sysvars#rent). The rent to open the account. |",
        "| `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionRegistered`] | Marks the registration of the position. |"
      ],
      accounts: [
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: ["The authority that can change the margin account"]
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The address paying for rent"]
        },
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account to register position type with"]
        },
        {
          name: "positionTokenMint",
          isMut: false,
          isSigner: false,
          docs: ["The mint for the position token being registered"]
        },
        {
          name: "config",
          isMut: false,
          isSigner: false,
          docs: ["The margin config for the token"]
        },
        {
          name: "tokenAccount",
          isMut: true,
          isSigner: false,
          docs: ["The token account to store hold the position assets in the custody of the", "margin account."]
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
      docs: [
        "Update the balance of a position stored in the margin account to match the actual",
        "stored by the SPL token account.",
        "",
        "When a user deposits tokens directly (without invoking this program), there's no",
        "update within the user's margin account to account for the new token balance. This",
        "instruction allows udating the margin account state to reflect the current available",
        "balance of collateral.",
        "",
        "# [Accounts](jet_margin::accounts::UpdatePositionBalance)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `margin_account` | `writable` | The margin account to update. |",
        "| `token_account` | `read_only` | The token account to update the balance for. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionBalanceUpdated`] | Marks the updating of the position balance. |",
        ""
      ],
      accounts: [
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account to update"]
        },
        {
          name: "tokenAccount",
          isMut: false,
          isSigner: false,
          docs: ["The token account to update the balance for"]
        }
      ],
      args: []
    },
    {
      name: "refreshPositionMetadata",
      docs: [
        "Update the metadata for a position stored in the margin account,",
        "in the case where the metadata has changed after the position was",
        "created.",
        "",
        "# [Accounts](jet_margin::accounts::RefreshPositionMetadata)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `margin_account` | `writable` | The margin account with the position to be refreshed. |",
        "| `metadata` | `read_only` | The metadata account for the token, which has been updated. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionMetadataRefreshed`] | Marks the refreshing of position metadata. |",
        ""
      ],
      accounts: [
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account with the position to be refreshed"]
        },
        {
          name: "metadata",
          isMut: false,
          isSigner: false,
          docs: ["The metadata account for the token, which has been updated"]
        },
        {
          name: "permit",
          isMut: false,
          isSigner: false,
          docs: ["permit that authorizes the refresher"]
        },
        {
          name: "refresher",
          isMut: false,
          isSigner: true,
          docs: ["account that is authorized to refresh position metadata"]
        }
      ],
      args: []
    },
    {
      name: "closePosition",
      docs: [
        "Close out a position, removing it from the account.",
        "",
        "Since there is a finite number of positions a single account can maintain it may be",
        "necessary for a user to close out old positions to take new ones.",
        "",
        "# [Accounts](jet_margin::accounts::ClosePosition)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `authority` | `signer` | The authority that can change the margin account. |",
        "| `receiver` | `writable` | The receiver for the rent released. |",
        "| `margin_account` | `writable` | The margin account with the position to close. |",
        "| `position_token_mint` | `read_only` | The mint for the position token being deregistered. |",
        "| `token_account` | `writable` | The token account for the position being closed. |",
        "| `token_program` | `read_only` | The [spl token program](https://spl.solana.com/token). |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionClosed`] | Marks the closure of the position. |",
        ""
      ],
      accounts: [
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: ["The authority that can change the margin account"]
        },
        {
          name: "receiver",
          isMut: true,
          isSigner: false,
          docs: ["The receiver for the rent released"]
        },
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account with the position to close"]
        },
        {
          name: "positionTokenMint",
          isMut: false,
          isSigner: false,
          docs: ["The mint for the position token being deregistered"]
        },
        {
          name: "tokenAccount",
          isMut: true,
          isSigner: false,
          docs: ["The token account for the position being closed"]
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
      docs: [
        "Verify that the account is healthy, by validating the collateralization",
        "ration is above the minimum.",
        "",
        "There's no real reason to call this instruction, outside of wanting to simulate",
        "the health check for a margin account.",
        "",
        "",
        "# [Accounts](jet_margin::accounts::VerifyHealthy)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `margin_account` | `read_only` | The account to verify the health of. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::VerifiedHealthy`] | Marks the verification of the position. |",
        ""
      ],
      accounts: [
        {
          name: "marginAccount",
          isMut: false,
          isSigner: false,
          docs: ["The account verify the health of"]
        }
      ],
      args: []
    },
    {
      name: "adapterInvoke",
      docs: [
        "Perform an action by invoking other programs, allowing them to alter",
        "the balances of the token accounts belonging to this margin account.",
        "",
        "This provides the margin account as a signer to any invoked instruction, and therefore",
        "grants the adapter authority over any tokens held by the margin account.",
        "",
        "This validates the invoked program by expecting an `adapter_metadata` account,",
        "which must exist for the instruction to be considered valid. The configuration",
        "for allowing adapter programs is controlled by protocol governance.",
        "",
        "All extra accounts passed in are used as the input accounts when invoking",
        "the provided adapter porgram.",
        "",
        "# Parameters",
        "",
        "* `data` - The instruction data to pass to the adapter program",
        "",
        "# [Accounts](jet_margin::accounts::AdapterInvoke)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `owner` | `signer` | The authority that owns the margin account. |",
        "| `margin_account` | `writable` | The margin account to proxy an action for. |",
        "| `adapter_program` | `read_only` | The program to be invoked. |",
        "| `adapter_metadata` | `read_only` | The metadata about the proxy program. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AdapterInvokeBegin`] | Marks the start of the adapter invocation (includes the margin account pubkey and the adapter program pubkey). |",
        "| [`events::PositionEvent`] _(Note that each single event represents a different adapter position)_ | The [PositionEvent](events::PositionEvent) marks the change in position. |",
        "| [`events::AdapterInvokeEnd`] | Marks the ending of the adapter invocation (includes no data except for the event itself being emitted). |"
      ],
      accounts: [
        {
          name: "owner",
          isMut: false,
          isSigner: true,
          docs: ["The authority that owns the margin account"]
        },
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account to proxy an action for"]
        },
        {
          name: "adapterProgram",
          isMut: false,
          isSigner: false,
          docs: ["The program to be invoked"]
        },
        {
          name: "adapterConfig",
          isMut: false,
          isSigner: false,
          docs: ["The metadata about the proxy program"]
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
      docs: [
        "Perform an action by invoking other programs, allowing them only to",
        "refresh the state of the margin account to be consistent with the actual",
        "underlying prices or positions, but not permitting new position changes.",
        "",
        "This is a permissionless way of updating the value of positions on a margin",
        "account which require some adapter to provide the update. Unlike `adapter_invoke`,",
        "this instruction will not provider the margin account as a signer to invoked programs,",
        "and they thefore do not have authority to modify any token balances held by the account.",
        "",
        "All extra accounts passed in are used as the input accounts when invoking",
        "the provided adapter porgram.",
        "",
        "# Parameters",
        "",
        "* `data` - The instruction data to pass to the adapter program",
        "",
        "# [Accounts](jet_margin::accounts::AccountingInvoke)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** |  **Description** |",
        "| `margin_account` | `writable` | The margin account to proxy an action for. |",
        "| `adapter_program` | `read_only` | The program to be invoked. |",
        "| `adapter_metadata` | `read_only` | The metadata about the proxy program. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| [`events::AccountingInvokeBegin`] | Signify that the accounting invocation process has begun. |",
        "| [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | The [PositionEvent](events::PositionEvent) marks the change in position. |",
        "| [`events::AccountingInvokeEnd`] | Signify that the accounting invocation process has ended. |"
      ],
      accounts: [
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account to proxy an action for"]
        },
        {
          name: "adapterProgram",
          isMut: false,
          isSigner: false,
          docs: ["The program to be invoked"]
        },
        {
          name: "adapterConfig",
          isMut: false,
          isSigner: false,
          docs: ["The metadata about the proxy program"]
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
      docs: [
        "Begin liquidating an account",
        "",
        "The account will enter a state preventing the owner from taking any action,",
        "until the liquidator process is complete.",
        "",
        "Requires the `liquidator_metadata` account, which restricts the signer to",
        "those approved by protocol governance.",
        "",
        "# [Accounts](jet_margin::accounts::LiquidateBegin)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `margin_account` | `writable` | The account in need of liquidation. |",
        "| `payer` | `signer` | The address paying rent. |",
        "| `liquidator` | `signer` | The liquidator account performing the liquidation. |",
        "| `liquidator_metadata` | `read_only` | The metadata describing the liquidator. |",
        "| `liquidation` | `writable` | The account to persist the state of liquidation. |",
        "| `system_program` | `read_only` | The [system native program](https://docs.solana.com/developing/runtime-facilities/programs#system-program). |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidationBegun`] | Marks the beginning of the liquidation. |"
      ],
      accounts: [
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account in need of liquidation"]
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The address paying rent"]
        },
        {
          name: "liquidator",
          isMut: false,
          isSigner: true,
          docs: ["The liquidator account performing the liquidation actions"]
        },
        {
          name: "permit",
          isMut: false,
          isSigner: false,
          docs: ["The permit allowing the liquidator to do this"]
        },
        {
          name: "liquidation",
          isMut: true,
          isSigner: false,
          docs: ["Account to persist the state of the liquidation"]
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
      docs: [
        "End the liquidation state for an account",
        "",
        "Normally must be signed by the liquidator that started the liquidation state. Can be",
        "signed by anyone after the [timeout period](jet_margin::LIQUIDATION_TIMEOUT) has elapsed.",
        "",
        "# [Accounts](jet_margin::accounts::LiquidateEnd)",
        "",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `authority` | `signer` | The pubkey calling the instruction to end liquidation. |",
        "| `margin_account` | `writable` | The account in need of liquidation. |",
        "| `liquidation` | `writable` | The account to persist the state of liquidation. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidationEnded`] | Marks the ending of the liquidation. |"
      ],
      accounts: [
        {
          name: "authority",
          isMut: true,
          isSigner: true,
          docs: [
            "If the liquidation is timed out, this can be any account",
            "If the liquidation is not timed out, this must be the liquidator, and it must be a signer"
          ]
        },
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account in need of liquidation"]
        },
        {
          name: "liquidation",
          isMut: true,
          isSigner: false,
          docs: ["Account to persist the state of the liquidation"]
        }
      ],
      args: []
    },
    {
      name: "liquidatorInvoke",
      docs: [
        "Perform an action by invoking another program, for the purposes of",
        "liquidating a margin account.",
        "",
        "Requires the account already be in the liquidation state, and the signer must",
        "be the same liquidator that started the liquidation state.",
        "",
        "# [Accounts](jet_margin::accounts::LiquidatorInvoke)",
        "|     |     |     |",
        "| --- | --- | --- |",
        "| **Name** | **Type** | **Description** |",
        "| `liquidator` | `signer` | The liquidator processing the margin account. |",
        "| `liquidation` | `writable` | The account to persist the state of liquidation. |",
        "| `margin_account` | `writable` | The margin account to proxy an action for. |",
        "| `adapter_program` | `read_only` | The program to be invoked. |",
        "| `adapter_metadata` | `read_only` | The metadata about the proxy program. |",
        "",
        "# Events",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidatorInvokeBegin`] | Marks the beginning of this liquidation event. |",
        "| [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | The [PositionEvent](events::PositionEvent) describing the change in position. |",
        "| [`events::LiquidatorInvokeEnd`] | Marks the ending of this liquidator event. |"
      ],
      accounts: [
        {
          name: "liquidator",
          isMut: false,
          isSigner: true,
          docs: ["The liquidator processing the margin account"]
        },
        {
          name: "liquidation",
          isMut: true,
          isSigner: false,
          docs: ["Account to persist the state of the liquidation"]
        },
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account to proxy an action for"]
        },
        {
          name: "adapterProgram",
          isMut: false,
          isSigner: false,
          docs: ["The program to be invoked"]
        },
        {
          name: "adapterConfig",
          isMut: false,
          isSigner: false,
          docs: ["The metadata about the proxy program"]
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
      name: "refreshPositionConfig",
      docs: [
        "Update the config for a token position stored in the margin account,",
        "in the case where the token config has changed after the position was",
        "created."
      ],
      accounts: [
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account with the position to be refreshed"]
        },
        {
          name: "config",
          isMut: false,
          isSigner: false,
          docs: ["The config account for the token, which has been updated"]
        },
        {
          name: "permit",
          isMut: false,
          isSigner: false,
          docs: ["permit that authorizes the refresher"]
        },
        {
          name: "refresher",
          isMut: false,
          isSigner: true,
          docs: ["account that is authorized to refresh position metadata"]
        }
      ],
      args: []
    },
    {
      name: "refreshDepositPosition",
      docs: ["Refresh the price/balance for a deposit position"],
      accounts: [
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The account to update"]
        },
        {
          name: "config",
          isMut: false,
          isSigner: false,
          docs: ["The margin config for the token"]
        },
        {
          name: "priceOracle",
          isMut: false,
          isSigner: false,
          docs: ["The oracle for the token"]
        }
      ],
      args: []
    },
    {
      name: "createDepositPosition",
      docs: ["Create a new account for holding SPL token deposits directly by a margin account."],
      accounts: [
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: ["The authority that can change the margin account"]
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The address paying for rent"]
        },
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account to register this deposit account with"]
        },
        {
          name: "mint",
          isMut: false,
          isSigner: false,
          docs: ["The mint for the token being stored in this account"]
        },
        {
          name: "config",
          isMut: false,
          isSigner: false,
          docs: ["The margin config for the token"]
        },
        {
          name: "tokenAccount",
          isMut: false,
          isSigner: false,
          docs: ["The token account to store deposits"]
        },
        {
          name: "associatedTokenProgram",
          isMut: false,
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
      name: "transferDeposit",
      docs: ["Transfer tokens into or out of a token account being used for deposits."],
      accounts: [
        {
          name: "owner",
          isMut: false,
          isSigner: true,
          docs: ["The authority that owns the margin account"]
        },
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account that the deposit account is associated with"]
        },
        {
          name: "sourceOwner",
          isMut: false,
          isSigner: false,
          docs: ["The authority for the source account"]
        },
        {
          name: "source",
          isMut: true,
          isSigner: false,
          docs: ["The source account to transfer tokens from"]
        },
        {
          name: "destination",
          isMut: true,
          isSigner: false,
          docs: ["The destination account to transfer tokens in"]
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: [
        {
          name: "amount",
          type: "u64"
        }
      ]
    },
    {
      name: "configureToken",
      docs: [
        "Set the configuration for a token, which allows it to be used as a position in a margin",
        "account.",
        "",
        "The configuration for a token only applies for the associated airspace, and changing any",
        "configuration requires the airspace authority to sign.",
        "",
        "The account storing the configuration will be funded if not already. If a `None` is provided as",
        "the updated configuration, then the account will be defunded."
      ],
      accounts: [
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: ["The authority allowed to make changes to configuration"]
        },
        {
          name: "airspace",
          isMut: false,
          isSigner: false,
          docs: ["The airspace being modified"]
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The payer for any rent costs, if required"]
        },
        {
          name: "mint",
          isMut: false,
          isSigner: false,
          docs: ["The mint for the token being configured"]
        },
        {
          name: "tokenConfig",
          isMut: true,
          isSigner: false,
          docs: ["The config account to be modified"]
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: [
        {
          name: "update",
          type: {
            option: {
              defined: "TokenConfigUpdate"
            }
          }
        }
      ]
    },
    {
      name: "configureAdapter",
      docs: [
        "Set the configuration for an adapter.",
        "",
        "The configuration for a token only applies for the associated airspace, and changing any",
        "configuration requires the airspace authority to sign.",
        "",
        "The account storing the configuration will be funded if not already. If a `None` is provided as",
        "the updated configuration, then the account will be defunded."
      ],
      accounts: [
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: ["The authority allowed to make changes to configuration"]
        },
        {
          name: "airspace",
          isMut: false,
          isSigner: false,
          docs: ["The airspace being modified"]
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The payer for any rent costs, if required"]
        },
        {
          name: "adapterProgram",
          isMut: false,
          isSigner: false,
          docs: ["The adapter being configured"]
        },
        {
          name: "adapterConfig",
          isMut: true,
          isSigner: false,
          docs: ["The config account to be modified"]
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: [
        {
          name: "isAdapter",
          type: "bool"
        }
      ]
    },
    {
      name: "configureLiquidator",
      docs: [
        "Set the configuration for a liquidator.",
        "",
        "The configuration for a token only applies for the associated airspace, and changing any",
        "configuration requires the airspace authority to sign.",
        "",
        "The account storing the configuration will be funded if not already. If a `None` is provided as",
        "the updated configuration, then the account will be defunded."
      ],
      accounts: [
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: ["The authority allowed to make changes to configuration"]
        },
        {
          name: "airspace",
          isMut: false,
          isSigner: false,
          docs: ["The airspace being modified"]
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The payer for any rent costs, if required"]
        },
        {
          name: "owner",
          isMut: false,
          isSigner: false,
          docs: ["The owner being configured"]
        },
        {
          name: "permit",
          isMut: true,
          isSigner: false,
          docs: ["The config account to be modified"]
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: [
        {
          name: "isLiquidator",
          type: "bool"
        }
      ]
    },
    {
      name: "configurePositionConfigRefresher",
      accounts: [
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: ["The authority allowed to make changes to configuration"]
        },
        {
          name: "airspace",
          isMut: false,
          isSigner: false,
          docs: ["The airspace being modified"]
        },
        {
          name: "payer",
          isMut: true,
          isSigner: true,
          docs: ["The payer for any rent costs, if required"]
        },
        {
          name: "owner",
          isMut: false,
          isSigner: false,
          docs: ["The owner being configured"]
        },
        {
          name: "permit",
          isMut: true,
          isSigner: false,
          docs: ["The config account to be modified"]
        },
        {
          name: "systemProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: [
        {
          name: "mayRefresh",
          type: "bool"
        }
      ]
    },
    {
      name: "adminTransferPosition",
      docs: [
        "Allow governing address to transfer any position from one margin account to another",
        "",
        "This is provided as a mechanism to allow for manually fixing issues that occur in the",
        "protocol due to bad user assets."
      ],
      accounts: [
        {
          name: "authority",
          isMut: false,
          isSigner: true,
          docs: ["The administrative authority"]
        },
        {
          name: "targetAccount",
          isMut: true,
          isSigner: false,
          docs: ["The target margin account to move a position into"]
        },
        {
          name: "sourceAccount",
          isMut: true,
          isSigner: false,
          docs: ["The source account to move a position out of"]
        },
        {
          name: "sourceTokenAccount",
          isMut: true,
          isSigner: false,
          docs: ["The token account to be moved from"]
        },
        {
          name: "targetTokenAccount",
          isMut: true,
          isSigner: false,
          docs: ["The token account to be moved into"]
        },
        {
          name: "tokenProgram",
          isMut: false,
          isSigner: false
        }
      ],
      args: [
        {
          name: "amount",
          type: "u64"
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
            docs: [
              "Data an adapter can use to check what the margin program thinks about the current invocation",
              "Must normally be zeroed, except during an invocation."
            ],
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
            docs: ["The owner of this account, which generally has to sign for any changes to it"],
            type: "publicKey"
          },
          {
            name: "airspace",
            docs: ["The airspace this account belongs to"],
            type: "publicKey"
          },
          {
            name: "liquidator",
            docs: ["The active liquidator for this account"],
            type: "publicKey"
          },
          {
            name: "positions",
            docs: ["The storage for tracking account balances"],
            type: {
              array: ["u8", 7432]
            }
          }
        ]
      }
    },
    {
      name: "liquidationState",
      docs: ["State of an in-progress liquidation"],
      type: {
        kind: "struct",
        fields: [
          {
            name: "liquidator",
            docs: ["The signer responsible for liquidation"],
            type: "publicKey"
          },
          {
            name: "marginAccount",
            docs: ["The margin account being liquidated"],
            type: "publicKey"
          },
          {
            name: "state",
            docs: ["The state object"],
            type: {
              defined: "Liquidation"
            }
          }
        ]
      }
    },
    {
      name: "tokenConfig",
      docs: [
        "The configuration account specifying parameters for a token when used",
        "in a position within a margin account."
      ],
      type: {
        kind: "struct",
        fields: [
          {
            name: "mint",
            docs: ["The mint for the token"],
            type: "publicKey"
          },
          {
            name: "underlyingMint",
            docs: ["The mint for the underlying token represented, if any"],
            type: "publicKey"
          },
          {
            name: "airspace",
            docs: ["The space this config is valid within"],
            type: "publicKey"
          },
          {
            name: "admin",
            docs: [
              "The administrator of this token, which has the authority to provide information",
              "about (e.g. prices) and otherwise modify position states for these tokens."
            ],
            type: {
              defined: "TokenAdmin"
            }
          },
          {
            name: "tokenKind",
            docs: [
              "Description of this token",
              "",
              "This determines the way the margin program values a token as a position in a",
              "margin account."
            ],
            type: {
              defined: "TokenKind"
            }
          },
          {
            name: "valueModifier",
            docs: ["A modifier to adjust the token value, based on the kind of token"],
            type: "u16"
          },
          {
            name: "maxStaleness",
            docs: ["The maximum staleness (seconds) that's acceptable for balances of this token"],
            type: "u64"
          }
        ]
      }
    },
    {
      name: "permit",
      docs: ["Configuration enabling a signer to execute permissioned actions"],
      type: {
        kind: "struct",
        fields: [
          {
            name: "airspace",
            docs: ["Airspace where the permit is valid."],
            type: "publicKey"
          },
          {
            name: "owner",
            docs: ["Address which may sign to perform the permitted actions."],
            type: "publicKey"
          },
          {
            name: "permissions",
            docs: ["Actions which may be performed with the signature of the owner."],
            type: "u32"
          }
        ]
      }
    },
    {
      name: "adapterConfig",
      docs: ["Configuration for allowed adapters"],
      type: {
        kind: "struct",
        fields: [
          {
            name: "airspace",
            docs: ["The airspace this adapter can be used in"],
            type: "publicKey"
          },
          {
            name: "adapterProgram",
            docs: ["The program address allowed to be called as an adapter"],
            type: "publicKey"
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
            docs: ["keyed by token mint, same as position"],
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
            docs: ["The current price of the asset"],
            type: "i64"
          },
          {
            name: "confidence",
            docs: ["The current confidence value for the asset price"],
            type: "u64"
          },
          {
            name: "twap",
            docs: ["The recent average price"],
            type: "i64"
          },
          {
            name: "publishTime",
            docs: ["The time that the price was published at"],
            type: "i64"
          },
          {
            name: "exponent",
            docs: ["The exponent for the price values"],
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
            name: "equity",
            type: "i128"
          },
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
      name: "TokenConfigUpdate",
      type: {
        kind: "struct",
        fields: [
          {
            name: "underlyingMint",
            docs: ["The underlying token represented, if any"],
            type: "publicKey"
          },
          {
            name: "admin",
            docs: ["The administration authority for the token"],
            type: {
              defined: "TokenAdmin"
            }
          },
          {
            name: "tokenKind",
            docs: ["Description of this token"],
            type: {
              defined: "TokenKind"
            }
          },
          {
            name: "valueModifier",
            docs: ["A modifier to adjust the token value, based on the kind of token"],
            type: "u16"
          },
          {
            name: "maxStaleness",
            docs: ["The maximum staleness (seconds) that's acceptable for balances of this token"],
            type: "u64"
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
      name: "PriceInfo",
      type: {
        kind: "struct",
        fields: [
          {
            name: "value",
            docs: ["The current price"],
            type: "i64"
          },
          {
            name: "timestamp",
            docs: ["The timestamp the price was valid at"],
            type: "u64"
          },
          {
            name: "exponent",
            docs: ["The exponent for the price value"],
            type: "i32"
          },
          {
            name: "isValid",
            docs: ["Flag indicating if the price is valid for the position"],
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
            docs: ["The address of the token/mint of the asset"],
            type: "publicKey"
          },
          {
            name: "address",
            docs: ["The address of the account holding the tokens."],
            type: "publicKey"
          },
          {
            name: "adapter",
            docs: ["The address of the adapter managing the asset"],
            type: "publicKey"
          },
          {
            name: "value",
            docs: ["The current value of this position, stored as a `Number128` with fixed precision."],
            type: {
              array: ["u8", 16]
            }
          },
          {
            name: "balance",
            docs: ["The amount of tokens in the account"],
            type: "u64"
          },
          {
            name: "balanceTimestamp",
            docs: ["The timestamp of the last balance update"],
            type: "u64"
          },
          {
            name: "price",
            docs: ["The current price/value of each token"],
            type: {
              defined: "PriceInfo"
            }
          },
          {
            name: "kind",
            docs: ["The kind of balance this position contains"],
            type: "u32"
          },
          {
            name: "exponent",
            docs: ["The exponent for the token value"],
            type: "i16"
          },
          {
            name: "valueModifier",
            docs: ["A weight on the value of this asset when counting collateral"],
            type: "u16"
          },
          {
            name: "maxStaleness",
            docs: ["The max staleness for the account balance (seconds)"],
            type: "u64"
          },
          {
            name: "flags",
            docs: ["Flags that are set by the adapter"],
            type: {
              defined: "AdapterPositionFlags"
            }
          },
          {
            name: "reserved",
            docs: ["Unused"],
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
            docs: ["The address of the mint for the position token"],
            type: "publicKey"
          },
          {
            name: "index",
            docs: ["The array index where the data for this position is located"],
            type: "u64"
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
            type: "u64"
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
      name: "Liquidation",
      type: {
        kind: "struct",
        fields: [
          {
            name: "startTime",
            docs: ["time that liquidate_begin initialized this liquidation"],
            type: "i64"
          },
          {
            name: "equityLoss",
            docs: ["The cumulative amount of equity lost during liquidation so far"],
            type: "i128"
          },
          {
            name: "maxEquityLoss",
            docs: ["The maximum amount of collateral allowed to be lost during all steps"],
            type: "i128"
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
            name: "flags",
            type: "u8"
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
      name: "TokenKind",
      docs: ["Description of the token's usage"],
      type: {
        kind: "enum",
        variants: [
          {
            name: "Collateral"
          },
          {
            name: "Claim"
          },
          {
            name: "AdapterCollateral"
          }
        ]
      }
    },
    {
      name: "TokenOracle",
      docs: ["Information about where to find the oracle data for a token"],
      type: {
        kind: "enum",
        variants: [
          {
            name: "Pyth",
            fields: [
              {
                name: "price",
                docs: ["The pyth address containing price information for a token."],
                type: "publicKey"
              },
              {
                name: "product",
                docs: ["The pyth address with product information for a token"],
                type: "publicKey"
              }
            ]
          }
        ]
      }
    },
    {
      name: "TokenAdmin",
      docs: ["Description of which program administers a token"],
      type: {
        kind: "enum",
        variants: [
          {
            name: "Margin",
            fields: [
              {
                name: "oracle",
                docs: ["An oracle that can be used to collect price information for a token"],
                type: {
                  defined: "TokenOracle"
                }
              }
            ]
          },
          {
            name: "Adapter",
            fields: ["publicKey"]
          }
        ]
      }
    },
    {
      name: "SyscallProvider",
      type: {
        kind: "enum",
        variants: [
          {
            name: "Mock",
            fields: [
              {
                defined: "dynFn()->T"
              }
            ]
          },
          {
            name: "SolanaRuntime"
          },
          {
            name: "Stub"
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
    },
    {
      name: "TransferPosition",
      fields: [
        {
          name: "sourceMarginAccount",
          type: "publicKey",
          index: false
        },
        {
          name: "targetMarginAccount",
          type: "publicKey",
          index: false
        },
        {
          name: "sourceTokenAccount",
          type: "publicKey",
          index: false
        },
        {
          name: "targetTokenAccount",
          type: "publicKey",
          index: false
        },
        {
          name: "amount",
          type: "u64",
          index: false
        }
      ]
    },
    {
      name: "TokenConfigured",
      fields: [
        {
          name: "airspace",
          type: "publicKey",
          index: false
        },
        {
          name: "update",
          type: {
            option: {
              defined: "TokenConfigUpdate"
            }
          },
          index: false
        },
        {
          name: "mint",
          type: "publicKey",
          index: false
        }
      ]
    },
    {
      name: "AdapterConfigured",
      fields: [
        {
          name: "airspace",
          type: "publicKey",
          index: false
        },
        {
          name: "adapterProgram",
          type: "publicKey",
          index: false
        },
        {
          name: "isAdapter",
          type: "bool",
          index: false
        }
      ]
    },
    {
      name: "PermitConfigured",
      fields: [
        {
          name: "airspace",
          type: "publicKey",
          index: false
        },
        {
          name: "owner",
          type: "publicKey",
          index: false
        },
        {
          name: "permissions",
          type: "u32",
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
    },
    {
      code: 141042,
      name: "WrongLiquidationState",
      msg: "liquidationState does not match given margin account"
    },
    {
      code: 141050,
      name: "WrongAirspace",
      msg: "attempting to mix entities from different airspaces"
    },
    {
      code: 141051,
      name: "InvalidConfig",
      msg: "attempting to use or set invalid configuration"
    },
    {
      code: 141052,
      name: "InvalidOracle",
      msg: "attempting to use or set invalid configuration"
    },
    {
      code: 141060,
      name: "InsufficientPermissions",
      msg: "the permit does not authorize this action"
    },
    {
      code: 141061,
      name: "PermitNotOwned",
      msg: "the permit is not owned by the current user"
    }
  ]
}
