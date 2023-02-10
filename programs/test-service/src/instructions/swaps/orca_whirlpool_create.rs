// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::seeds::{POSITION, SWAP_POOL_INFO};
use crate::state::{TokenInfo, WhirlpoolSwapInfo};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use orca_whirlpool::cpi::accounts::{
    InitializeConfig, InitializeFeeTier, InitializePool, InitializeReward, InitializeTickArray,
    ModifyLiquidity, OpenPosition,
};
use orca_whirlpool::state::{OpenPositionBumps, Whirlpool, WhirlpoolBumps, WhirlpoolsConfig};

// todo list
// ix:
// 1. init whirlpool config
// 2. init fee tier
// 3. init pool
// 4. init reward
// 5. init tick arrays
// 6. open position // may need position metadata ix
// 7. increase liquidity

// notes:
// a. load all accounts and bumps
// b. hard code defaults/preset
// what are the account infos & signers needed for invoke sign

// what are the config variables?
// 1. set to 300: default_protocol_fee_rate: u16,
// 2. set to 8 for stables: tick_spacing: u16
// 3. set to 3000: default_fee_rate: u16
// 4. set to 5: initSqrtPrice = defaultInitSqrtPrice, anchor bn 5
// 5. set to 5: price new decimal 5
// 6. set to 0: reward index
#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone, Eq, PartialEq)]
pub struct WhirlpoolCreateParams {
    pub liquidity_level: u8,
    pub price_threshold: u16,
    pub default_protocol_fee_rate: u16,
    pub tick_spacing: u16,
    pub fee_rate: u16,
    pub init_sqrt_price: u128,
    pub reward_index: u8,
}

#[derive(Accounts)]
pub struct WhirlpoolCreate<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(mut)]
    mint_a: Box<Account<'info, Mint>>,

    #[account(mut)]
    mint_b: Box<Account<'info, Mint>>,

    #[account(constraint = info_a.mint == mint_a.key(),
              constraint = info_a.pyth_price == pyth_price_a.key()
    )]
    info_a: Box<Account<'info, TokenInfo>>,

    #[account(constraint = info_b.mint == mint_b.key(),
              constraint = info_b.pyth_price == pyth_price_b.key()
    )]
    info_b: Box<Account<'info, TokenInfo>>,

    pyth_price_a: AccountInfo<'info>,
    pyth_price_b: AccountInfo<'info>,

    #[account(has_one = whirlpool,
        seeds = [
          SWAP_POOL_INFO,
          mint_a.key().as_ref(),
          mint_b.key().as_ref()
        ],
        bump,
    )]
    pool_info: Box<Account<'info, WhirlpoolSwapInfo>>,

    #[account(mut,
        token::mint = mint_a,
        token::authority = payer)]
    token_owner_a: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        token::mint = mint_b,
        token::authority = payer)]
    token_owner_b: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        token::mint = mint_a,
        token::authority = whirlpool)]
    token_vault_a: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        token::mint = mint_b,
        token::authority = whirlpool)]
    token_vault_b: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    whirlpool_config: Account<'info, WhirlpoolsConfig>,

    #[account(mut)]
    fee_tier: AccountInfo<'info>,

    #[account(mut)]
    whirlpool: Account<'info, Whirlpool>,

    #[account(mut)]
    reward_mint: AccountInfo<'info>,

    #[account(mut,
        token::mint = reward_mint)]
    reward_vault: Box<Account<'info, TokenAccount>>,

    // #[account(mut)]
    // reward_vault: AccountInfo<'info>,
    #[account(mut)]
    tick_array: AccountInfo<'info>,

    #[account(mut)]
    tick_array_lower: AccountInfo<'info>,

    #[account(mut)]
    tick_array_upper: AccountInfo<'info>,

    // todo - double check if AccountInfo is needed for position, position mint and position token account
    // position: positionInfo.positionPda.publicKey,
    // positionTokenAccount: positionInfo.positionTokenAccount,
    #[account(mut)]
    position: AccountInfo<'info>,

    #[account(mut)]
    position_mint: AccountInfo<'info>,

    #[account(mut,
        token::mint = position_mint)]
    position_token_account: Box<Account<'info, TokenAccount>>,

    // todo - double check if authority is okay same as payer
    // #[account(constraint = pool_authority.key() == whirlpool_config.fee_authority.key())]
    // pool_authority: AccountInfo<'info>,
    swap_program: AccountInfo<'info>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> WhirlpoolCreate<'info> {
    fn init_config(&self) -> Result<()> {
        orca_whirlpool::cpi::initialize_config(
            CpiContext::new(
                self.swap_program.to_account_info(),
                InitializeConfig {
                    config: self.whirlpool_config.to_account_info(),
                    funder: self.payer.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
            ),
            self.payer.key(),
            self.payer.key(),
            self.payer.key(),
            3000,
        )
    }

    fn init_fee_tier(&self) -> Result<()> {
        orca_whirlpool::cpi::initialize_fee_tier(
            CpiContext::new(
                self.swap_program.to_account_info(),
                InitializeFeeTier {
                    config: self.whirlpool_config.to_account_info(),
                    fee_tier: self.fee_tier.to_account_info(),
                    funder: self.payer.to_account_info(),
                    fee_authority: self.payer.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
            ),
            8, // tick_spacing: 8 for stable, 128 for standard
            300,
        )
    }

    fn init_pool(&self) -> Result<()> {
        orca_whirlpool::cpi::initialize_pool(
            CpiContext::new_with_signer(
                self.swap_program.to_account_info(),
                InitializePool {
                    whirlpools_config: self.whirlpool_config.to_account_info(),
                    token_mint_a: self.mint_a.to_account_info(),
                    token_mint_b: self.mint_b.to_account_info(),
                    whirlpool: self.whirlpool.to_account_info(),
                    token_vault_a: self.token_vault_a.to_account_info(),
                    token_vault_b: self.token_vault_b.to_account_info(),
                    fee_tier: self.fee_tier.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                    funder: self.payer.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
                &[&self.whirlpool.seeds()],
            ),
            WhirlpoolBumps { whirlpool_bump: 0 },
            8, // tick_spacing: 8 for stable, 128 for standard
            5, // anchor bn 5
        )
    }

    fn init_reward(&self) -> Result<()> {
        orca_whirlpool::cpi::initialize_reward(
            CpiContext::new(
                self.swap_program.to_account_info(),
                InitializeReward {
                    reward_authority: self.payer.to_account_info(),
                    funder: self.payer.to_account_info(),
                    whirlpool: self.whirlpool.to_account_info(),
                    reward_mint: self.reward_mint.to_account_info(),
                    reward_vault: self.reward_vault.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                },
            ),
            0,
        )
    }

    // todo - fixme, modify for more than one tick array.
    // need
    // 1. tick spacing: standard - 128, stable - 8
    // 2. direction: a_to_b ? -1 : 1
    // 3. result: PDA[]
    fn init_tick_array(&self) -> Result<()> {
        orca_whirlpool::cpi::initialize_tick_array(
            CpiContext::new(
                self.swap_program.to_account_info(),
                InitializeTickArray {
                    whirlpool: self.whirlpool.to_account_info(),
                    funder: self.payer.to_account_info(),
                    tick_array: self.tick_array.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
            ),
            22528, // to 33792
        )
    }

    fn open_position(&self) -> Result<()> {
        orca_whirlpool::cpi::open_position(
            CpiContext::new_with_signer(
                self.swap_program.to_account_info(),
                OpenPosition {
                    funder: self.payer.to_account_info(),
                    owner: self.payer.to_account_info(),
                    position: self.position.to_account_info(),
                    position_mint: self.position_mint.to_account_info(),
                    position_token_account: self.position_token_account.to_account_info(),
                    whirlpool: self.whirlpool.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    rent: self.rent.to_account_info(),
                    associated_token_program: self.associated_token_program.to_account_info(),
                },
                &[&[POSITION.as_ref(), self.position_mint.key().as_ref()]],
            ),
            OpenPositionBumps { position_bump: 0 },
            29440,
            33536,
        )
    }

    // todo - future: calculate token max for a & b
    // orca util for getting token max a & b
    //     public static getTokenAmountsFromLiquidity(
    //     liquidity: BN,
    //     currentSqrtPrice: BN,
    //     lowerSqrtPrice: BN,
    //     upperSqrtPrice: BN,
    //     round_up: boolean
    //   ): TokenAmounts {
    //     const _liquidity = new Decimal(liquidity.toString());
    //     const _currentPrice = new Decimal(currentSqrtPrice.toString());
    //     const _lowerPrice = new Decimal(lowerSqrtPrice.toString());
    //     const _upperPrice = new Decimal(upperSqrtPrice.toString());
    //     let tokenA, tokenB;
    //     if (currentSqrtPrice.lt(lowerSqrtPrice)) {
    // x = L * (pb - pa) / (pa * pb)
    //       tokenA = MathUtil.toX64_Decimal(_liquidity)
    //         .mul(_upperPrice.sub(_lowerPrice))
    //         .div(_lowerPrice.mul(_upperPrice));
    //       tokenB = new Decimal(0);
    //     } else if (currentSqrtPrice.lt(upperSqrtPrice)) {
    // x = L * (pb - p) / (p * pb)
    // y = L * (p - pa)
    //       tokenA = MathUtil.toX64_Decimal(_liquidity)
    //         .mul(_upperPrice.sub(_currentPrice))
    //         .div(_currentPrice.mul(_upperPrice));
    //       tokenB = MathUtil.fromX64_Decimal(_liquidity.mul(_currentPrice.sub(_lowerPrice)));
    //     } else {
    // y = L * (pb - pa)
    //       tokenA = new Decimal(0);
    //       tokenB = MathUtil.fromX64_Decimal(_liquidity.mul(_upperPrice.sub(_lowerPrice)));
    //     }

    //     if (round_up) {
    //       return {
    //         tokenA: new u64(tokenA.ceil().toString()),
    //         tokenB: new u64(tokenB.ceil().toString()),
    //       };
    //     } else {
    //       return {
    //         tokenA: new u64(tokenA.floor().toString()),
    //         tokenB: new u64(tokenB.floor().toString()),
    //       };
    //     }
    //   }

    fn increase_liquidity(&self) -> Result<()> {
        orca_whirlpool::cpi::increase_liquidity(
            CpiContext::new(
                self.swap_program.to_account_info(),
                ModifyLiquidity {
                    whirlpool: self.whirlpool.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    position_authority: self.payer.to_account_info(),
                    position: self.position.to_account_info(),
                    position_token_account: self.position_token_account.to_account_info(),
                    token_owner_account_a: self.token_owner_a.to_account_info(),
                    token_owner_account_b: self.token_owner_b.to_account_info(),
                    token_vault_a: self.token_vault_a.to_account_info(),
                    token_vault_b: self.token_vault_b.to_account_info(),
                    tick_array_lower: self.tick_array_lower.to_account_info(),
                    tick_array_upper: self.tick_array_upper.to_account_info(),
                },
            ),
            100_000,
            100_000,
            100_000,
        )
    }
}

pub fn whirlpool_create_handler(
    ctx: Context<WhirlpoolCreate>,
    params: WhirlpoolCreateParams,
) -> Result<()> {
    let pool_info = &mut ctx.accounts.pool_info;
    pool_info.whirlpool = ctx.accounts.whirlpool.key();
    pool_info.liquidity_level = params.liquidity_level;
    pool_info.price_threshold = params.price_threshold;

    // todo - test: ix - init whirlpool config
    // needs:
    // 1. keygen: whirlpool config account - config state, context wallet: funder, sys
    // 2. keygen: fee authority - pubkey,
    // 3. keygen: collect_protocol_fees_authority: Pubkey,
    // 4. keygen: reward_emissions_super_authority: Pubkey,
    // 5. set to 300: default_protocol_fee_rate: u16,

    // const configKeypairs: TestWhirlpoolsConfigKeypairs = {
    //     feeAuthorityKeypair: Keypair.generate(),
    //     collectProtocolFeesAuthorityKeypair: Keypair.generate(),
    //     rewardEmissionsSuperAuthorityKeypair: Keypair.generate(),
    //   };
    //   const configInitInfo = {
    //     whirlpoolsConfigKeypair: Keypair.generate(),
    //     feeAuthority: configKeypairs.feeAuthorityKeypair.publicKey,
    //     collectProtocolFeesAuthority: configKeypairs.collectProtocolFeesAuthorityKeypair.publicKey,
    //     rewardEmissionsSuperAuthority: configKeypairs.rewardEmissionsSuperAuthorityKeypair.publicKey,
    //     defaultProtocolFeeRate: 300,
    //     funder: funder || context.wallet.publicKey,
    //   };

    // orca_whirlpool::instructions::initialize_config(
    //     ctx,
    //     fee_authority,
    //     collect_protocol_fees_authority,
    //     reward_emissions_super_authority,
    //     default_protocol_fee_rate,
    // );

    // signers: [params.whirlpoolsConfigKeypair],

    // todo - test: ix - init fee tier
    // const testTickSpacing = TickSpacing.Stable: number 8 | Standard: number 128
    // need>
    // config info keypair
    // fee authority keypair
    // test tick spacing : 8
    // default fee rate: 3000

    // signers: [],

    // todo - test: ix - init pool
    // need>
    // initSqrtPrice = defaultInitSqrtPrice, anchor bn 5
    // price new decimal 5
    // whirlpool pda:
    // public static getWhirlpool(
    //     programId: PublicKey,
    //     whirlpoolsConfigKey: PublicKey,
    //     tokenMintAKey: PublicKey,
    //     tokenMintBKey: PublicKey,
    //     tickSpacing: number
    //   ) {
    //     return AddressUtil.findProgramAddress(
    //       [
    //         Buffer.from(PDA_WHIRLPOOL_SEED),
    //         whirlpoolsConfigKey.toBuffer(),
    //         tokenMintAKey.toBuffer(),
    //         tokenMintBKey.toBuffer(),
    //         new BN(tickSpacing).toArrayLike(Buffer, "le", 2),
    //       ],
    //       programId
    //     );
    // whirlpool data:
    // pub struct Whirlpool {
    //     pub whirlpools_config: Pubkey, // 32
    //     pub whirlpool_bump: [u8; 1],   // 1

    //     pub tick_spacing: u16,          // 2
    //     pub tick_spacing_seed: [u8; 2], // 2

    // Stored as hundredths of a basis point
    // u16::MAX corresponds to ~6.5%
    //     pub fee_rate: u16, // 2

    // Portion of fee rate taken stored as basis points
    //     pub protocol_fee_rate: u16, // 2

    // Maximum amount that can be held by Solana account
    //     pub liquidity: u128, // 16

    // MAX/MIN at Q32.64, but using Q64.64 for rounder bytes
    // Q64.64
    //     pub sqrt_price: u128,        // 16
    //     pub tick_current_index: i32, // 4

    //     pub protocol_fee_owed_a: u64, // 8
    //     pub protocol_fee_owed_b: u64, // 8

    //     pub token_mint_a: Pubkey,  // 32
    //     pub token_vault_a: Pubkey, // 32

    // Q64.64
    //     pub fee_growth_global_a: u128, // 16

    //     pub token_mint_b: Pubkey,  // 32
    //     pub token_vault_b: Pubkey, // 32

    // Q64.64
    //     pub fee_growth_global_b: u128, // 16

    //     pub reward_last_updated_timestamp: u64, // 8

    //     pub reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS], // 384
    // }

    // keygen: tokenVaultAKeypair
    // keygen: tokenVaultBKeypair

    // init pool params:
    // 1. initSqrtPrice = defaultInitSqrtPrice, anchor bn 5
    // 2. whirlpoolsConfig: configKey,
    // 3. tokenMintA: tokenAMintPubKey,
    // 4. tokenMintB: tokenBMintPubKey,
    // 5. whirlpoolPda,
    // 6. tokenVaultAKeypair,
    // 7. tokenVaultBKeypair,
    // 8. feeTierKey,
    // 9. tickSpacing,
    // 10. funder: funder || context.wallet.publicKey,

    // const whirlpoolBumps: WhirlpoolBumpsData = {
    //     whirlpoolBump: whirlpoolPda.bump,
    //   };

    //   ix needs >
    //   whirlpoolBump
    //   accounts: {
    //     whirlpoolsConfig,
    //     tokenMintA,
    //     tokenMintB,
    //     funder,
    //     whirlpool: whirlpoolPda.publicKey,
    //     tokenVaultA: tokenVaultAKeypair.publicKey,
    //     tokenVaultB: tokenVaultBKeypair.publicKey,
    //     feeTier: feeTierKey,
    //     tokenProgram: TOKEN_PROGRAM_ID,
    //     systemProgram: SystemProgram.programId,
    //     rent: SYSVAR_RENT_PUBKEY,
    // }

    // signers: [tokenVaultAKeypair, tokenVaultBKeypair],

    // todo - test: init reward
    // const { params } = await initializeReward(
    //     ctx,
    //     configKeypairs.rewardEmissionsSuperAuthorityKeypair,
    //     poolInitInfo.whirlpoolPda.publicKey,
    //     0
    //   );

    // need >
    // rewardAuthorityKeypair: whirlpool config reward emission super authority
    // whirlpool: whirlpool pda
    // rewardIndex: 0 | 1

    // whirlpool config keygen
    // feeAuthorityKeypair
    // collectProtocolFeesAuthorityKeypair
    // rewardEmissionsSuperAuthorityKeypair

    // reward param keygen
    // rewardMint = await createMint(provider);
    // export async function createMintInstructions(
    //     provider: AnchorProvider,
    //     authority: web3.PublicKey,
    //     mint: web3.PublicKey
    // ) {
    //     let instructions = [
    //     web3.SystemProgram.createAccount({
    //         fromPubkey: provider.wallet.publicKey,
    //         newAccountPubkey: mint,
    //         space: 82,
    //         lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
    //         programId: TEST_TOKEN_PROGRAM_ID,
    //     }),
    //     Token.createInitMintInstruction(TEST_TOKEN_PROGRAM_ID, mint, 0, authority, null),
    //     ];
    //     return instructions;
    // rewardVaultKeypair

    // ix - init reward need >
    // rewardAuthority: rewardAuthorityKeypair.publicKey,
    // funder: funder?.publicKey || ctx.wallet.publicKey,
    // whirlpool,
    // rewardMint,
    // rewardVaultKeypair,
    // rewardIndex,

    // todo - test: liquidity - init pool with tokens
    // const tickArrays = await initTickArrayRange(
    //     ctx,
    //     whirlpoolPda.publicKey,
    //     22528, // to 33792
    //     3,
    //     TickSpacing.Standard,
    //     false
    //   );

    //   const fundParams: FundedPositionParams[] = [
    //     {
    //       liquidityAmount: new anchor.BN(100_000),
    //       tickLowerIndex: 27904,
    //       tickUpperIndex: 33408,
    //     },
    //   ];

    //   const positionInfos = await fundPositions(
    //     ctx,
    //     poolInitInfo,
    //     tokenAccountA,
    //     tokenAccountB,
    //     fundParams
    //   );

    // todo - test: init tick array range PDA[]
    // ix - init one or more tick arrays
    // todo - test: fund position with fund param
    // 1. ix - open position
    // get pdas tick array lower and upper
    // 2. ix - increase liquidity

    // ctx.accounts.request_token_a()?;
    // ctx.accounts.request_token_b()?;

    // let pool_info = &mut ctx.accounts.whirlpool_config;
    // pool_info.pool_state = ctx.accounts.whirlpool.key();
    // pool_info.liquidity_level = params.liquidity_level;
    // pool_info.price_threshold = params.price_threshold;

    // let bump = *ctx.bumps.get("pool_state").unwrap();

    // let ix_init_config = orca_whirlpool::cpi::initialize_config(
    //     Context::new(
    //         &orca_whirlpool::id(),
    //         &mut InitializeConfig {
    //             config: ctx.accounts.whirlpool_config.clone(),
    //             funder: ctx.accounts.payer.clone(),
    //             system_program: ctx.accounts.system_program.clone(),
    //         },
    //         &[],
    //         BTreeMap::new(),
    //     ),
    //     ctx.accounts.payer,
    //     ctx.accounts.payer,
    //     ctx.accounts.payer,
    //     3000,
    // );

    // let ix_init_fee_tier = orca_whirlpool::cpi::initialize_fee_tier(
    //     Context::new(
    //         &orca_whirlpool::id(),
    //         &mut InitializeFeeTier {
    //             config: todo!(),
    //             fee_tier: todo!(),
    //             funder: todo!(),
    //             fee_authority: todo!(),
    //             system_program: todo!(),
    //         },
    //         &[],
    //         BTreeMap::new(),
    //     ),
    //     8 // tick_spacing: 8 for stable, 128 for standard
    //     300,
    // );

    // let ix_init_pool = orca_whirlpool::cpi::initialize_pool(
    //     CpiContext::new(
    //         &orca_whirlpool::id(),
    //         &mut InitializePool {
    //             whirlpools_config: todo!(),
    //             token_mint_a: todo!(),
    //             token_mint_b: todo!(),
    //             whirlpool: todo!(),
    //             token_vault_a: todo!(),
    //             token_vault_b: todo!(),
    //             fee_tier: todo!(),
    //             token_program: todo!(),
    //             rent: todo!(),
    //         },
    //         &[],
    //         BTreeMap::new(),
    //     ),
    //     bumps,
    //     tick_spacing,
    //     initial_sqrt_price,
    // );

    // let ix_init_reward = orca_whirlpool::cpi::initialize_reward(
    //     CpiContext::new(
    //         &orca_whirlpool::id(),
    //         &mut InitializeReward {
    //             reward_authority: todo!(),
    //             funder: todo!(),
    //             whirlpool: todo!(),
    //             reward_mint: todo!(),
    //             reward_vault: todo!(),
    //             token_program: todo!(),
    //             system_program: todo!(),
    //             rent: todo!(),
    //         },
    //         &[],
    //         BTreeMap::new(),
    //     ),
    //     0,
    // );

    // let ix_init_tick_array = orca_whirlpool::cpi::initialize_tick_array(
    //     Context::new(
    //         &orca_whirlpool::id(),
    //         &mut InitializeTickArray {
    //             whirlpool: todo!(),
    //             funder: todo!(),
    //             tick_array: todo!(),
    //             system_program: todo!(),
    //         },
    //         &[],
    //         BTreeMap::new(),
    //     ),
    //     start_tick_index,
    // );

    // let ix_init_tick_array = orca_whirlpool::cpi::initialize_tick_array(ctx, start_tick_index);
    // let ix_init_tick_array = orca_whirlpool::cpi::initialize_tick_array(ctx, start_tick_index);

    // let ix_open_position = orca_whirlpool::cpi::open_position(
    //     CpiContext::new(
    //         &orca_whirlpool::id(),
    //         &mut OpenPosition {
    //             funder: todo!(),
    //             owner: todo!(),
    //             position: todo!(),
    //             position_mint: todo!(),
    //             position_token_account: todo!(),
    //             whirlpool: todo!(),
    //             token_program: todo!(),
    //             system_program: todo!(),
    //             rent: todo!(),
    //             associated_token_program: todo!(),
    //         },
    //         &[],
    //         BTreeMap::new(),
    //     ),
    //     bumps,
    //     tick_lower_index,
    //     tick_upper_index,
    // );

    // let ix_increase_liquidity = orca_whirlpool::cpi::increase_liquidity(
    //     Context::new(
    //         &orca_whirlpool::id(),
    //         &mut ModifyLiquidity {
    //             whirlpool: todo!(),
    //             token_program: todo!(),
    //             position_authority: todo!(),
    //             position: todo!(),
    //             position_token_account: todo!(),
    //             token_owner_account_a: todo!(),
    //             token_owner_account_b: todo!(),
    //             token_vault_a: todo!(),
    //             token_vault_b: todo!(),
    //             tick_array_lower: todo!(),
    //             tick_array_upper: todo!(),
    //         },
    //         &[],
    //         BTreeMap::new(),
    //     ),
    //     100_000,
    //     token_max_a,
    //     token_max_b,
    // );

    // get seeds
    // invoke_signed

    ctx.accounts.init_config()?;
    ctx.accounts.init_fee_tier()?;
    ctx.accounts.init_pool()?;
    ctx.accounts.init_reward()?;
    ctx.accounts.init_tick_array()?; // todo - question: init three times?
    ctx.accounts.open_position()?;
    ctx.accounts.increase_liquidity()?;

    Ok(())
}
