export type JetMargin = {
  version: "1.0.0"
  name: "jet_margin"
  docs: [
    "This crate documents the instructions used in the `margin` program of the",
    "[jet-v2 repo](https://github.com/jet-lab/jet-v2/).",
    "Handler functions are described for each instruction well as struct parameters",
    "(and their types and descriptions are listed) and any handler function",
    "parameters aside from parameters that exist in every instruction handler function.",
    "Parameters of events emitted for the purposes of data logging are also included."
  ]
  constants: [
    {
      name: "TOKEN_CONFIG_SEED"
      type: {
        defined: "&[u8]"
      }
      value: 'b"token-config"'
    },
    {
      name: "ADAPTER_CONFIG_SEED"
      type: {
        defined: "&[u8]"
      }
      value: 'b"adapter-config"'
    },
    {
      name: "LIQUIDATOR_CONFIG_SEED"
      type: {
        defined: "&[u8]"
      }
      value: 'b"liquidator-config"'
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
        "This instruction does the following:",
        "",
        "1.  Let `account` be a mutable reference to the margin account.",
        "",
        "2.  Initialize the margin account by setting the margin account version, owner,",
        "bump seed, user seed, and setting liquidator pubkey field to the default",
        "(if an account is being liquidated, the liquidator pubkey will be set here).",
        "",
        "3.  Emit the `AccountCreated` event for data logging (see table below):",
        "",
        "4.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of create\\_account.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `owner` | The owner of the new margin account. |",
        "| `payer` | The pubkey paying rent for the new margin account opening. |",
        "| `margin_account` | The margin account to initialize for the owner. |",
        "| `system_program` | The system program. |",
        "",
        "**Events emitted by create\\_account.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AccountCreated`] | The created account (includes the margin account pubkey, the owner of margin account’s the pubkey, and the seed). |"
      ]
      accounts: [
        {
          name: "owner"
          isMut: false
          isSigner: true
          docs: ["The owner of the new margin account"]
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
        "This instruction does the following:",
        "",
        "1.  Let `account`be a reference to the margin account being closed.",
        "",
        "2.  Check if the loaded margin account has any open positions.",
        "",
        "a.  If open positions exist, then return `ErrorCode::AccountNotEmpty`.",
        "",
        "3.  Emit the `AccountClosed` event for data logging (see table below).",
        "",
        "4.  Load the margin account.",
        "",
        "5.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of close\\_account.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `owner` | The owner of the account being closed. |",
        "| `receiver` | The account to get any returned rent. |",
        "| `margin_account` | The account being closed. |",
        "",
        "**Events emitted by close\\_account.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AccountClosed`] | The closed account (includes the margin account pubkey). |"
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
        "Register a position for some token that will be custodied by margin.",
        "Currently this applies to anything other than a claim.",
        "",
        "This instruction does the following:",
        "",
        "1.  Register a new position that belongs to the individual margin account, allocate account space for it, and set the parameters for that asset type.",
        "",
        "2.  Emit the `PositionRegistered` event for data logging (see table below).",
        "",
        "3.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of register\\_position.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `authority` | The authority that can change the margin account. |",
        "| `payer` | The address paying for rent. |",
        "| `margin_account` | The margin account to register position type with. |",
        "| `position_token_mint` | The mint for the position token being registered. |",
        "| `metadata` | The metadata account that references the correct oracle for the token. |",
        "| `token_account` | The token account to store hold the position assets in the custody of the margin account. |",
        "| `token_program` | The token program of the token accounts to store for this margin account. |",
        "| `rent` | The rent to open the account. |",
        "| `system_program` | The system program. |",
        "",
        "**Events emitted by register\\_position.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionRegistered`] | The position registered (includes the margin account pubkey, the authority pubkey of that margin account, and the position itself). |"
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
          name: "metadata"
          isMut: false
          isSigner: false
          docs: ["The metadata account that references the correct oracle for the token"]
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
        "Update the balance of a position stored in the margin account to",
        "match the actual balance stored by the SPL token acount.",
        "",
        "This instruction does the following:",
        "",
        "1.  Let `margin_account` be a mutable reference to the margin account.",
        "",
        "2.  Let `token_account` be a reference to the token account.",
        "",
        "3.  Load a margin account position and update it with `token_account`, `account`, and `balance`.",
        "",
        "4.  Emit the `PositionBalanceUpdated` event for data logging (see table below).",
        "",
        "5.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of update\\_position\\_balance.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The margin account to update. |",
        "| `token_account` | The token account to update the balance for. |",
        "",
        "**Events emitted by update\\_position\\_balance.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionBalanceUpdated`] | The updated position (includes the token account, margin account pubkey, and token balance). |",
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
        "This instruction does the following:",
        "",
        "1.  Read account token metadata.",
        "",
        "2.  Load the margin account.",
        "",
        "3.  Update the position with refreshed metadata.",
        "",
        "4.  Emit the `PositionMetadataRefreshed` event for data logging (see table below).",
        "",
        "5.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of refresh\\_position\\_metadata.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The margin account with the position to be refreshed. |",
        "| `metadata` | The metadata account for the token, which has been updated. |",
        "",
        "**Events emitted by refresh\\_position\\_metadata.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionMetadataRefreshed`] | The position of which metadata was refreshed (including the margin account pubkey and the `position` itself). |"
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
        }
      ]
      args: []
    },
    {
      name: "closePosition"
      docs: [
        "Close out a position, freeing up space in the account.",
        "",
        "This instruction does the following:",
        "",
        "1.  Let `account` be a mutable reference to the margin account.",
        "",
        "2.  Verify the authority of `account`.",
        "",
        "3.  Record unregistering (closing) the position in question of `account`, which involves passing the token mint account, token account, and margin account authority.",
        "",
        "4.  If the token account authority of the account is the same as the authority.",
        "",
        "a.  Return the token account.",
        "",
        "5.  Emit the `PositionClosed` event for data logging (see table below):",
        "",
        "6.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of close\\_position.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `authority` | The authority that can change the margin account. |",
        "| `receiver` | The receiver for the rent released. |",
        "| `margin_account` | The margin account with the position to close. |",
        "| `position_token_mint` | The mint for the position token being deregistered. |",
        "| `token_account` | The token account for the position being closed. |",
        "| `token_program` | The token program for the position being closed. |",
        "",
        "**Events emitted by close\\_position.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionClosed`] | The closed position (includes the margin account authority’s pubkey and the relevant token pool’s note mint pubkey). |"
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
        "This instruction does the following:",
        "",
        "1.  Let `account` be the loaded margin account.",
        "",
        "2.  Check if all positions for that margin account are healthy.",
        "",
        "a.  If there are unhealthy positions exist for this margin account, return `False`.",
        "",
        "3.  Emit the `VerifiedHealthy` event for data logging (see table below).",
        "",
        "4.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of verify\\_healthy.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The account to verify the health of. |",
        "",
        "**Events emitted by verify\\_healthy.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "|[`events::VerifiedHealthy`] | The margin account pubkeys of verified healthy accounts. |"
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
        "This instruction does the following:",
        "",
        "1.  If a read account has the `liquidation` parameter set to a pubkey:",
        "",
        "a.  This means that that margin account is already under liquidation by the liquidator at that pubkey.",
        "",
        "b.  Return `ErrorCode::Liquidating`.",
        "",
        "2.  Emit the `AdapterInvokeBegin` event for data logging (see table below).",
        "",
        "3.  Check if any positions that have changed via adapters.",
        "",
        "a.  For each changed position, emit each existing adapter position as an `event` (see table below).",
        "",
        "4.  Emit the `AdapterInvokeEnd` event for data logging (see table below).",
        "",
        "5.  Verify that margin accounts positions via adapter are healthy.",
        "",
        "6.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of adapter\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `owner` | The authority that owns the margin account. |",
        "| `margin_account` | The margin account to proxy an action for. |",
        "| `adapter_program` | The program to be invoked. |",
        "| `adapter_metadata` | The metadata about the proxy program. |",
        "",
        "**Events emitted by adapter\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AdapterInvokeBegin`] | Marks the start of the adapter invocation (includes the margin account pubkey and the adapter program pubkey). |",
        "| [`events::PositionEvent`] _(Note that each single event represents a different adapter position)_ | Each adapter position is emitted as an event (includes the margin account, the adapter program, the accounts, and a value of `true` for the field `signed`. |",
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
        }
      ]
      args: [
        {
          "name": "instructions",
          "type": {
            "vec": {
              "defined": "IxData"
            }
          }
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
        "This instruction does the following:",
        "",
        "1.  Emit `AccountingInvokeBegin` events for data logging (see table below).",
        "",
        "2.  Check if any positions that have changed via adapters.",
        "",
        "a.  For each changed position, emit each existing adapter position as an `event` (see table below).",
        "",
        "3.  Emit `AccountingInvokeEnd` event for data logging (see table below).",
        "",
        "4.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of accounting\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The margin account to proxy an action for. |",
        "| `adapter_program` | The program to be invoked. |",
        "| `adapter_metadata` | The metadata about the proxy program. |",
        "",
        "**Events emitted by accounting\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| [`events::AccountingInvokeBegin`] | Signify that the accounting invocation process has begun. |",
        "| [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | Each adapter position is emitted as an event (includes the margin account, the adapter program, the remaining accounts, and a value of `false` for the field `signed`. |",
        "| [`events::AccountingInvokeEnd`] | The margin account to proxy an action for. |"
      ]
      accounts: [
        {
          name: "marginAccount"
          isMut: true
          isSigner: false
          docs: ["The margin account to proxy an action for"]
        }
      ]
      args: [
        {
          "name": "instructions",
          "type": {
            "vec": {
              "defined": "IxData"
            }
          }
        }
      ]
    },
    {
      name: "liquidateBegin"
      docs: [
        "Begin liquidating an account",
        "",
        "This instruction does the following:",
        "",
        "1.  Read `liquidation` and `liquidator` from the account.",
        "",
        "2.  Let `account` be a mutable reference to the margin account.",
        "",
        "3.  Verify that the account is subject to liquidation, return `False` if not.",
        "",
        "4.  Verify that the account is not already being liquidated.",
        "",
        "a.  If the liquidator is already assigned to this margin account, do nothing.",
        "",
        "b.  Else if there is no liquidator assigned to the unhealthy account, the liquidator can claim this unhealthy account and begin the process of liquidation.",
        "",
        "c.  Otherwise return `ErrorCode::Liquidating` because it is already claimed by some other liquidator.",
        "",
        "5.  Record the valuation of the account.",
        "",
        "6.  Record the minimum valuation change of the account.",
        "",
        "7.  Emit the `LiquidationBegun` event for data logging (see table below).",
        "",
        "8.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of liquidate\\_begin.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The account in need of liquidation. |",
        "| `payer` | The address paying rent. |",
        "| `liquidator` | The liquidator account performing the liquidation. |",
        "| `liquidator_metadata` | The metadata describing the liquidator. |",
        "| `liquidation` | The account to persist the state of liquidation. |",
        "| `system_program` | The system program. |",
        "",
        "**Events emitted by liquidate\\_begin.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidationBegun`] | The event marking the beginning of liquidation (Includes the margin account pubkey, the liquidator pubkey, the liquidation pubkey, the liquidation data, and the valuation of the margin account to be liquidated). |"
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
          name: "liquidatorMetadata"
          isMut: false
          isSigner: false
          docs: ["The metadata describing the liquidator"]
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
        "Stop liquidating an account",
        "",
        "This instruction does the following:",
        "",
        "1.  Let `account` be a mutable reference to the margin account.",
        "",
        "2.  Let `start_time` be the time that the liquidation on this margin account began, if it exists",
        "",
        "3.  Let `timed_out` be the boolean representing the type of account:",
        "",
        "a.  If the liquidation is timed out, then this can be any account.",
        "",
        "b.  If the liquidation is not timed out, then this must be the liquidator, and it must be a signer.",
        "",
        "4.  Check if the entity trying to end the liquidation is not the liquidator.",
        "",
        "a.  If not, return `ErrorCode::UnauthorizedLiquidator`.",
        "",
        "5.  Record the end of the liquidation.",
        "",
        "6.  Emit the `LiquidationEnded` event for data logging (see table below).",
        "",
        "7.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of liquidate\\_end.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `authority` | The pubkey calling the instruction to end liquidation. |",
        "| `margin_account` | The account in need of liquidation. |",
        "| `liquidation` | The account to persist the state of liquidation. |",
        "",
        "**Events emitted by liquidate\\_end.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidationEnded`] | The event marking the end of liquidation (Includes the margin account pubkey, the authority of the margin account pubkey, and the timed\\_out boolean that is true if the liquidation has timed out). |"
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
        "This instruction does the following:",
        "",
        "1.  Load the margin account.",
        "",
        "2.  Let `start_value` be the valuation of the margin account before invoking the liquidator.",
        "",
        "3.  Emit the `LiquidatorInvokeBegin` event for data logging (see table below).",
        "",
        "4.  Loop through adapter and store positions, getting and storing as `margin_account`, `adapter_program`, `accounts` and `signed`.",
        "",
        "5.  Emit each adapter position as an `event` (see table below).",
        "",
        "6.  Let`liquidation` be a mutable copy of the liquidated account.",
        "",
        "7.  Let `end_value` be the valuation of the margin account after the liquidation attempt, after verifying that a liquidation did occur.",
        "",
        "8.  Emit the `LiquidatorInvokeEnd` event for data logging (see table below).",
        "",
        "9.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of liquidator\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `liquidator` | The liquidator processing the margin account. |",
        "| `liquidation` | The account to persist the state of liquidation. |",
        "| `margin_account` | The margin account to proxy an action for. |",
        "| `adapter_program` | The program to be invoked. |",
        "| `adapter_metadata` | The metadata about the proxy program. |",
        "",
        "**Events emitted by liquidator\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidatorInvokeBegin`] | Marks the beginning of this liquidation event (includes the margin account pubkey, the adapter program pubkey, and the liquidator pubkey that is liquidating that margin account or adapter position). |",
        "| [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | Each adapter position is emitted as an event (includes the margin account, the adapter program, the accounts, and a value of `true` for the `signed` field. |",
        "| [`events::LiquidatorInvokeEnd`] | Marks the ending of this liquidator event (includes the liquidation data and the valuation of the account after liquidation has been performed). |"
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
        }
      ]
      args: [
        {
          "name": "instructions",
          "type": {
            "vec": {
              "defined": "IxData"
            }
          }
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
          isSigner: true
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
          name: "liquidator"
          isMut: false
          isSigner: false
          docs: ["The liquidator being configured"]
        },
        {
          name: "liquidatorConfig"
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
            name: "liquidation"
            docs: ["The state of an active liquidation for this account"]
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
            name: "adapterProgram"
            docs: [
              "The adapter program in control of positions of this token",
              "",
              "If this is `None`, then the margin program is in control of this asset, and",
              "thus determining its price. The `oracle` field must be set to allow the margin",
              "program to price the asset."
            ]
            type: {
              option: "publicKey"
            }
          },
          {
            name: "oracle"
            docs: [
              "The oracle for the token",
              "",
              "This only has effect in the margin program when the price for the token is not",
              "being managed by an adapter."
            ]
            type: {
              option: {
                defined: "TokenOracle"
              }
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
      name: "liquidatorConfig"
      docs: ["Configuration for allowed liquidators"]
      type: {
        kind: "struct"
        fields: [
          {
            name: "airspace"
            docs: ["The airspace this liquidator is being configured to act within"]
            type: "publicKey"
          },
          {
            name: "liquidator"
            docs: ["The address of the liquidator allowed to act"]
            type: "publicKey"
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
      "name": "IxData",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "numAccounts",
            "type": "u8"
          },
          {
            "name": "data",
            "type": "bytes"
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
            name: "adapterProgram"
            docs: ["The adapter program in control of positions of this token"]
            type: {
              option: "publicKey"
            }
          },
          {
            name: "oracle"
            docs: ["The oracle for the token"]
            type: {
              option: {
                defined: "TokenOracle"
              }
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
            type: {
              defined: "usize"
            }
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
            type: {
              defined: "usize"
            }
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
            name: "equityChange"
            docs: [
              "cumulative change in equity caused by invocations during the liquidation so far",
              "negative if equity is lost"
            ]
            type: "i128"
          },
          {
            name: "minEquityChange"
            docs: [
              "lowest amount of equity change that is allowed during invoke steps",
              "typically negative or zero",
              "if equity_change goes lower than this number, liquidate_invoke should fail"
            ]
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
      name: "PositionKind"
      type: {
        kind: "enum"
        variants: [
          {
            name: "NoValue"
          },
          {
            name: "Deposit"
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
            name: "NoValue"
          },
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
    }
  ]
}

export const IDL: JetMargin = {
  version: "1.0.0",
  name: "jet_margin",
  docs: [
    "This crate documents the instructions used in the `margin` program of the",
    "[jet-v2 repo](https://github.com/jet-lab/jet-v2/).",
    "Handler functions are described for each instruction well as struct parameters",
    "(and their types and descriptions are listed) and any handler function",
    "parameters aside from parameters that exist in every instruction handler function.",
    "Parameters of events emitted for the purposes of data logging are also included."
  ],
  constants: [
    {
      name: "TOKEN_CONFIG_SEED",
      type: {
        defined: "&[u8]"
      },
      value: 'b"token-config"'
    },
    {
      name: "ADAPTER_CONFIG_SEED",
      type: {
        defined: "&[u8]"
      },
      value: 'b"adapter-config"'
    },
    {
      name: "LIQUIDATOR_CONFIG_SEED",
      type: {
        defined: "&[u8]"
      },
      value: 'b"liquidator-config"'
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
        "This instruction does the following:",
        "",
        "1.  Let `account` be a mutable reference to the margin account.",
        "",
        "2.  Initialize the margin account by setting the margin account version, owner,",
        "bump seed, user seed, and setting liquidator pubkey field to the default",
        "(if an account is being liquidated, the liquidator pubkey will be set here).",
        "",
        "3.  Emit the `AccountCreated` event for data logging (see table below):",
        "",
        "4.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of create\\_account.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `owner` | The owner of the new margin account. |",
        "| `payer` | The pubkey paying rent for the new margin account opening. |",
        "| `margin_account` | The margin account to initialize for the owner. |",
        "| `system_program` | The system program. |",
        "",
        "**Events emitted by create\\_account.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AccountCreated`] | The created account (includes the margin account pubkey, the owner of margin account’s the pubkey, and the seed). |"
      ],
      accounts: [
        {
          name: "owner",
          isMut: false,
          isSigner: true,
          docs: ["The owner of the new margin account"]
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
        "This instruction does the following:",
        "",
        "1.  Let `account`be a reference to the margin account being closed.",
        "",
        "2.  Check if the loaded margin account has any open positions.",
        "",
        "a.  If open positions exist, then return `ErrorCode::AccountNotEmpty`.",
        "",
        "3.  Emit the `AccountClosed` event for data logging (see table below).",
        "",
        "4.  Load the margin account.",
        "",
        "5.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of close\\_account.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `owner` | The owner of the account being closed. |",
        "| `receiver` | The account to get any returned rent. |",
        "| `margin_account` | The account being closed. |",
        "",
        "**Events emitted by close\\_account.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AccountClosed`] | The closed account (includes the margin account pubkey). |"
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
        "Register a position for some token that will be custodied by margin.",
        "Currently this applies to anything other than a claim.",
        "",
        "This instruction does the following:",
        "",
        "1.  Register a new position that belongs to the individual margin account, allocate account space for it, and set the parameters for that asset type.",
        "",
        "2.  Emit the `PositionRegistered` event for data logging (see table below).",
        "",
        "3.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of register\\_position.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `authority` | The authority that can change the margin account. |",
        "| `payer` | The address paying for rent. |",
        "| `margin_account` | The margin account to register position type with. |",
        "| `position_token_mint` | The mint for the position token being registered. |",
        "| `metadata` | The metadata account that references the correct oracle for the token. |",
        "| `token_account` | The token account to store hold the position assets in the custody of the margin account. |",
        "| `token_program` | The token program of the token accounts to store for this margin account. |",
        "| `rent` | The rent to open the account. |",
        "| `system_program` | The system program. |",
        "",
        "**Events emitted by register\\_position.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionRegistered`] | The position registered (includes the margin account pubkey, the authority pubkey of that margin account, and the position itself). |"
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
          name: "metadata",
          isMut: false,
          isSigner: false,
          docs: ["The metadata account that references the correct oracle for the token"]
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
        "Update the balance of a position stored in the margin account to",
        "match the actual balance stored by the SPL token acount.",
        "",
        "This instruction does the following:",
        "",
        "1.  Let `margin_account` be a mutable reference to the margin account.",
        "",
        "2.  Let `token_account` be a reference to the token account.",
        "",
        "3.  Load a margin account position and update it with `token_account`, `account`, and `balance`.",
        "",
        "4.  Emit the `PositionBalanceUpdated` event for data logging (see table below).",
        "",
        "5.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of update\\_position\\_balance.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The margin account to update. |",
        "| `token_account` | The token account to update the balance for. |",
        "",
        "**Events emitted by update\\_position\\_balance.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionBalanceUpdated`] | The updated position (includes the token account, margin account pubkey, and token balance). |",
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
        "This instruction does the following:",
        "",
        "1.  Read account token metadata.",
        "",
        "2.  Load the margin account.",
        "",
        "3.  Update the position with refreshed metadata.",
        "",
        "4.  Emit the `PositionMetadataRefreshed` event for data logging (see table below).",
        "",
        "5.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of refresh\\_position\\_metadata.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The margin account with the position to be refreshed. |",
        "| `metadata` | The metadata account for the token, which has been updated. |",
        "",
        "**Events emitted by refresh\\_position\\_metadata.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionMetadataRefreshed`] | The position of which metadata was refreshed (including the margin account pubkey and the `position` itself). |"
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
        }
      ],
      args: []
    },
    {
      name: "closePosition",
      docs: [
        "Close out a position, freeing up space in the account.",
        "",
        "This instruction does the following:",
        "",
        "1.  Let `account` be a mutable reference to the margin account.",
        "",
        "2.  Verify the authority of `account`.",
        "",
        "3.  Record unregistering (closing) the position in question of `account`, which involves passing the token mint account, token account, and margin account authority.",
        "",
        "4.  If the token account authority of the account is the same as the authority.",
        "",
        "a.  Return the token account.",
        "",
        "5.  Emit the `PositionClosed` event for data logging (see table below):",
        "",
        "6.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of close\\_position.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `authority` | The authority that can change the margin account. |",
        "| `receiver` | The receiver for the rent released. |",
        "| `margin_account` | The margin account with the position to close. |",
        "| `position_token_mint` | The mint for the position token being deregistered. |",
        "| `token_account` | The token account for the position being closed. |",
        "| `token_program` | The token program for the position being closed. |",
        "",
        "**Events emitted by close\\_position.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::PositionClosed`] | The closed position (includes the margin account authority’s pubkey and the relevant token pool’s note mint pubkey). |"
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
        "This instruction does the following:",
        "",
        "1.  Let `account` be the loaded margin account.",
        "",
        "2.  Check if all positions for that margin account are healthy.",
        "",
        "a.  If there are unhealthy positions exist for this margin account, return `False`.",
        "",
        "3.  Emit the `VerifiedHealthy` event for data logging (see table below).",
        "",
        "4.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of verify\\_healthy.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The account to verify the health of. |",
        "",
        "**Events emitted by verify\\_healthy.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "|[`events::VerifiedHealthy`] | The margin account pubkeys of verified healthy accounts. |"
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
        "This instruction does the following:",
        "",
        "1.  If a read account has the `liquidation` parameter set to a pubkey:",
        "",
        "a.  This means that that margin account is already under liquidation by the liquidator at that pubkey.",
        "",
        "b.  Return `ErrorCode::Liquidating`.",
        "",
        "2.  Emit the `AdapterInvokeBegin` event for data logging (see table below).",
        "",
        "3.  Check if any positions that have changed via adapters.",
        "",
        "a.  For each changed position, emit each existing adapter position as an `event` (see table below).",
        "",
        "4.  Emit the `AdapterInvokeEnd` event for data logging (see table below).",
        "",
        "5.  Verify that margin accounts positions via adapter are healthy.",
        "",
        "6.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of adapter\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `owner` | The authority that owns the margin account. |",
        "| `margin_account` | The margin account to proxy an action for. |",
        "| `adapter_program` | The program to be invoked. |",
        "| `adapter_metadata` | The metadata about the proxy program. |",
        "",
        "**Events emitted by adapter\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::AdapterInvokeBegin`] | Marks the start of the adapter invocation (includes the margin account pubkey and the adapter program pubkey). |",
        "| [`events::PositionEvent`] _(Note that each single event represents a different adapter position)_ | Each adapter position is emitted as an event (includes the margin account, the adapter program, the accounts, and a value of `true` for the field `signed`. |",
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
        }
      ],
      args: [
        {
          "name": "instructions",
          "type": {
            "vec": {
              "defined": "IxData"
            }
          }
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
        "This instruction does the following:",
        "",
        "1.  Emit `AccountingInvokeBegin` events for data logging (see table below).",
        "",
        "2.  Check if any positions that have changed via adapters.",
        "",
        "a.  For each changed position, emit each existing adapter position as an `event` (see table below).",
        "",
        "3.  Emit `AccountingInvokeEnd` event for data logging (see table below).",
        "",
        "4.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of accounting\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The margin account to proxy an action for. |",
        "| `adapter_program` | The program to be invoked. |",
        "| `adapter_metadata` | The metadata about the proxy program. |",
        "",
        "**Events emitted by accounting\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| [`events::AccountingInvokeBegin`] | Signify that the accounting invocation process has begun. |",
        "| [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | Each adapter position is emitted as an event (includes the margin account, the adapter program, the remaining accounts, and a value of `false` for the field `signed`. |",
        "| [`events::AccountingInvokeEnd`] | The margin account to proxy an action for. |"
      ],
      accounts: [
        {
          name: "marginAccount",
          isMut: true,
          isSigner: false,
          docs: ["The margin account to proxy an action for"]
        }
      ],
      args: [
        {
          "name": "instructions",
          "type": {
            "vec": {
              "defined": "IxData"
            }
          }
        }
      ]
    },
    {
      name: "liquidateBegin",
      docs: [
        "Begin liquidating an account",
        "",
        "This instruction does the following:",
        "",
        "1.  Read `liquidation` and `liquidator` from the account.",
        "",
        "2.  Let `account` be a mutable reference to the margin account.",
        "",
        "3.  Verify that the account is subject to liquidation, return `False` if not.",
        "",
        "4.  Verify that the account is not already being liquidated.",
        "",
        "a.  If the liquidator is already assigned to this margin account, do nothing.",
        "",
        "b.  Else if there is no liquidator assigned to the unhealthy account, the liquidator can claim this unhealthy account and begin the process of liquidation.",
        "",
        "c.  Otherwise return `ErrorCode::Liquidating` because it is already claimed by some other liquidator.",
        "",
        "5.  Record the valuation of the account.",
        "",
        "6.  Record the minimum valuation change of the account.",
        "",
        "7.  Emit the `LiquidationBegun` event for data logging (see table below).",
        "",
        "8.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of liquidate\\_begin.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `margin_account` | The account in need of liquidation. |",
        "| `payer` | The address paying rent. |",
        "| `liquidator` | The liquidator account performing the liquidation. |",
        "| `liquidator_metadata` | The metadata describing the liquidator. |",
        "| `liquidation` | The account to persist the state of liquidation. |",
        "| `system_program` | The system program. |",
        "",
        "**Events emitted by liquidate\\_begin.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidationBegun`] | The event marking the beginning of liquidation (Includes the margin account pubkey, the liquidator pubkey, the liquidation pubkey, the liquidation data, and the valuation of the margin account to be liquidated). |"
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
          name: "liquidatorMetadata",
          isMut: false,
          isSigner: false,
          docs: ["The metadata describing the liquidator"]
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
        "Stop liquidating an account",
        "",
        "This instruction does the following:",
        "",
        "1.  Let `account` be a mutable reference to the margin account.",
        "",
        "2.  Let `start_time` be the time that the liquidation on this margin account began, if it exists",
        "",
        "3.  Let `timed_out` be the boolean representing the type of account:",
        "",
        "a.  If the liquidation is timed out, then this can be any account.",
        "",
        "b.  If the liquidation is not timed out, then this must be the liquidator, and it must be a signer.",
        "",
        "4.  Check if the entity trying to end the liquidation is not the liquidator.",
        "",
        "a.  If not, return `ErrorCode::UnauthorizedLiquidator`.",
        "",
        "5.  Record the end of the liquidation.",
        "",
        "6.  Emit the `LiquidationEnded` event for data logging (see table below).",
        "",
        "7.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of liquidate\\_end.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `authority` | The pubkey calling the instruction to end liquidation. |",
        "| `margin_account` | The account in need of liquidation. |",
        "| `liquidation` | The account to persist the state of liquidation. |",
        "",
        "**Events emitted by liquidate\\_end.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidationEnded`] | The event marking the end of liquidation (Includes the margin account pubkey, the authority of the margin account pubkey, and the timed\\_out boolean that is true if the liquidation has timed out). |"
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
        "This instruction does the following:",
        "",
        "1.  Load the margin account.",
        "",
        "2.  Let `start_value` be the valuation of the margin account before invoking the liquidator.",
        "",
        "3.  Emit the `LiquidatorInvokeBegin` event for data logging (see table below).",
        "",
        "4.  Loop through adapter and store positions, getting and storing as `margin_account`, `adapter_program`, `accounts` and `signed`.",
        "",
        "5.  Emit each adapter position as an `event` (see table below).",
        "",
        "6.  Let`liquidation` be a mutable copy of the liquidated account.",
        "",
        "7.  Let `end_value` be the valuation of the margin account after the liquidation attempt, after verifying that a liquidation did occur.",
        "",
        "8.  Emit the `LiquidatorInvokeEnd` event for data logging (see table below).",
        "",
        "9.  Return `Ok(())`.",
        "",
        "",
        "**Parameters of liquidator\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Name** | **Description** |",
        "| `liquidator` | The liquidator processing the margin account. |",
        "| `liquidation` | The account to persist the state of liquidation. |",
        "| `margin_account` | The margin account to proxy an action for. |",
        "| `adapter_program` | The program to be invoked. |",
        "| `adapter_metadata` | The metadata about the proxy program. |",
        "",
        "**Events emitted by liquidator\\_invoke.rs:**",
        "",
        "|     |     |",
        "| --- | --- |",
        "| **Event Name** | **Description** |",
        "| [`events::LiquidatorInvokeBegin`] | Marks the beginning of this liquidation event (includes the margin account pubkey, the adapter program pubkey, and the liquidator pubkey that is liquidating that margin account or adapter position). |",
        "| [`events::PositionEvent`] _(Note that each single event represents an different adapter position)_ | Each adapter position is emitted as an event (includes the margin account, the adapter program, the accounts, and a value of `true` for the `signed` field. |",
        "| [`events::LiquidatorInvokeEnd`] | Marks the ending of this liquidator event (includes the liquidation data and the valuation of the account after liquidation has been performed). |"
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
        }
      ],
      args: [
        {
          "name": "instructions",
          "type": {
            "vec": {
              "defined": "IxData"
            }
          }
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
          isSigner: true,
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
          name: "liquidator",
          isMut: false,
          isSigner: false,
          docs: ["The liquidator being configured"]
        },
        {
          name: "liquidatorConfig",
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
            name: "liquidation",
            docs: ["The state of an active liquidation for this account"],
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
            name: "adapterProgram",
            docs: [
              "The adapter program in control of positions of this token",
              "",
              "If this is `None`, then the margin program is in control of this asset, and",
              "thus determining its price. The `oracle` field must be set to allow the margin",
              "program to price the asset."
            ],
            type: {
              option: "publicKey"
            }
          },
          {
            name: "oracle",
            docs: [
              "The oracle for the token",
              "",
              "This only has effect in the margin program when the price for the token is not",
              "being managed by an adapter."
            ],
            type: {
              option: {
                defined: "TokenOracle"
              }
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
      name: "liquidatorConfig",
      docs: ["Configuration for allowed liquidators"],
      type: {
        kind: "struct",
        fields: [
          {
            name: "airspace",
            docs: ["The airspace this liquidator is being configured to act within"],
            type: "publicKey"
          },
          {
            name: "liquidator",
            docs: ["The address of the liquidator allowed to act"],
            type: "publicKey"
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
      "name": "IxData",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "numAccounts",
            "type": "u8"
          },
          {
            "name": "data",
            "type": "bytes"
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
            name: "adapterProgram",
            docs: ["The adapter program in control of positions of this token"],
            type: {
              option: "publicKey"
            }
          },
          {
            name: "oracle",
            docs: ["The oracle for the token"],
            type: {
              option: {
                defined: "TokenOracle"
              }
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
            name: "equityChange",
            docs: [
              "cumulative change in equity caused by invocations during the liquidation so far",
              "negative if equity is lost"
            ],
            type: "i128"
          },
          {
            name: "minEquityChange",
            docs: [
              "lowest amount of equity change that is allowed during invoke steps",
              "typically negative or zero",
              "if equity_change goes lower than this number, liquidate_invoke should fail"
            ],
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
          },
          {
            name: "AdapterCollateral"
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
            name: "NoValue"
          },
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
    }
  ]
}
