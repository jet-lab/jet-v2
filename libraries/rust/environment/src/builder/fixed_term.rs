use solana_sdk::{
    pubkey::Pubkey, rent::Rent, signature::Keypair, signer::Signer, system_instruction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};

use jet_instructions::{
    control::get_control_authority_address,
    fixed_term::{
        derive::{self, market_from_tenor},
        event_queue_len, orderbook_slab_len, FixedTermIxBuilder, InitializeMarketParams, Market,
        OrderbookAddresses, FIXED_TERM_PROGRAM,
    },
    test_service::{
        self, derive_pyth_price, derive_pyth_product, derive_token_info, TokenCreateParams,
    },
};
use jet_margin::{TokenAdmin, TokenConfigUpdate, TokenKind, TokenOracle};
use jet_solana_client::{
    transaction::TransactionBuilder, NetworkUserInterface, NetworkUserInterfaceExt,
};

use crate::config::FixedTermMarketConfig;

use super::{margin::configure_margin_token, Builder, BuilderError, NetworkKind, TokenContext};

const EVENT_QUEUE_CAPACITY: usize = 8192;
const ORDERBOOK_CAPACITY: usize = 16384;

pub(crate) async fn configure_market_for_token<I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    cranks: &[Pubkey],
    token: &TokenContext,
    config: &FixedTermMarketConfig,
) -> Result<(), BuilderError> {
    let payer = builder.payer();

    let market_address = market_from_tenor(&token.airspace, &token.mint, config.borrow_tenor);

    let control_authority = get_control_authority_address();
    let fee_destination = get_associated_token_address(&control_authority, &token.mint);

    if !builder.account_exists(&fee_destination).await? {
        builder.setup([create_associated_token_account_idempotent(
            &payer,
            &control_authority,
            &token.mint,
            &spl_token::ID,
        )]);
    }

    let market = builder
        .interface
        .get_anchor_account::<Market>(&market_address)
        .await?;

    let ix_builder = match market {
        None => create_market_for_token(builder, token, config, fee_destination).await?,
        Some(market) => FixedTermIxBuilder::new(
            payer,
            token.airspace,
            token.mint,
            market_address,
            builder.authority,
            token.pyth_price,
            token.pyth_price,
            Some(fee_destination),
            OrderbookAddresses {
                bids: market.bids,
                asks: market.asks,
                event_queue: market.event_queue,
            },
        ),
    };

    if builder.network != NetworkKind::Mainnet {
        // Register to get an oracle for the ticket token on testing networks
        let ticket_mint = derive::ticket_mint(&market_address);
        let ticket_info = derive_token_info(&ticket_mint);

        log::info!(
            "register market {} ticket mint {}",
            &market_address,
            &ticket_mint
        );

        if !builder.account_exists(&ticket_info).await? {
            builder.setup([test_service::if_not_initialized(
                ticket_info,
                test_service::token_register(
                    &payer,
                    ticket_mint,
                    &TokenCreateParams {
                        authority: builder.authority,
                        oracle_authority: token.oracle_authority,
                        decimals: token.desc.decimals.unwrap(),
                        max_amount: u64::MAX,
                        source_symbol: token.desc.symbol.clone(),
                        price_ratio: config.ticket_price.unwrap_or(1.0),
                        symbol: format!("{}_{}", token.desc.symbol.clone(), config.borrow_tenor),
                        name: format!("{}_{}", token.desc.name.clone(), config.borrow_tenor),
                    },
                ),
            )]);
        }
    }

    // TODO: support pausing

    // Register tokens for this market with the margin system
    configure_margin_for_market(builder, token, &market_address, config).await?;

    // set permissions for cranks
    configure_cranks_for_market(builder, &ix_builder, cranks).await?;

    Ok(())
}

async fn configure_cranks_for_market<I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    ix_builder: &FixedTermIxBuilder,
    cranks: &[Pubkey],
) -> Result<(), BuilderError> {
    for crank in cranks {
        if builder
            .account_exists(&ix_builder.crank_authorization(crank))
            .await?
        {
            continue;
        }

        builder.propose(
            cranks
                .iter()
                .map(|crank| ix_builder.authorize_crank(*crank)),
        );
    }

    if builder.network == NetworkKind::Localnet && cranks.is_empty() {
        let authorization = ix_builder.crank_authorization(&builder.authority);

        if !builder.account_exists(&authorization).await? {
            builder.propose([test_service::if_not_initialized(
                authorization,
                ix_builder.authorize_crank(builder.authority),
            )]);
        }
    }

    Ok(())
}

async fn configure_margin_for_market<I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    token: &TokenContext,
    market_address: &Pubkey,
    config: &FixedTermMarketConfig,
) -> Result<(), BuilderError> {
    let claims_mint = derive::claims_mint(market_address);
    let ticket_collateral_mint = derive::ticket_collateral_mint(market_address);
    let ticket_mint = derive::ticket_mint(market_address);

    let ticket_oracle = match builder.network {
        NetworkKind::Localnet | NetworkKind::Devnet => Some(TokenOracle::Pyth {
            price: derive_pyth_price(&ticket_mint),
            product: derive_pyth_product(&ticket_mint),
        }),
        NetworkKind::Mainnet => match (config.ticket_pyth_price, config.ticket_pyth_product) {
            (Some(price), Some(product)) => Some(TokenOracle::Pyth { price, product }),
            _ => None,
        },
    };

    configure_margin_token(
        builder,
        &token.airspace,
        &ticket_mint,
        Some(TokenConfigUpdate {
            underlying_mint: ticket_mint,
            admin: TokenAdmin::Margin {
                oracle: ticket_oracle.unwrap(),
            },
            token_kind: TokenKind::Collateral,
            value_modifier: config.ticket_collateral_weight,
            max_staleness: 0,
        }),
    )
    .await?;

    configure_margin_token(
        builder,
        &token.airspace,
        &ticket_collateral_mint,
        Some(TokenConfigUpdate {
            underlying_mint: ticket_mint,
            admin: TokenAdmin::Adapter(FIXED_TERM_PROGRAM),
            token_kind: TokenKind::AdapterCollateral,
            value_modifier: config.ticket_collateral_weight,
            max_staleness: 0,
        }),
    )
    .await?;

    configure_margin_token(
        builder,
        &token.airspace,
        &claims_mint,
        Some(TokenConfigUpdate {
            underlying_mint: token.mint,
            admin: TokenAdmin::Adapter(FIXED_TERM_PROGRAM),
            token_kind: TokenKind::Claim,
            value_modifier: token.desc.max_leverage,
            max_staleness: 0,
        }),
    )
    .await?;

    Ok(())
}

async fn create_market_for_token<I: NetworkUserInterface>(
    builder: &mut Builder<I>,
    token: &TokenContext,
    config: &FixedTermMarketConfig,
    fee_destination: Pubkey,
) -> Result<FixedTermIxBuilder, BuilderError> {
    let key_eq = Keypair::new();
    let key_bids = Keypair::new();
    let key_asks = Keypair::new();

    let orderbook = OrderbookAddresses {
        bids: key_bids.pubkey(),
        asks: key_asks.pubkey(),
        event_queue: key_eq.pubkey(),
    };

    let len_eq = event_queue_len(EVENT_QUEUE_CAPACITY);
    let len_orders = orderbook_slab_len(ORDERBOOK_CAPACITY);

    let mut seed = [0u8; 32];
    seed[..8].copy_from_slice(&config.borrow_tenor.to_le_bytes());

    let payer = builder.payer();
    let ix_builder = FixedTermIxBuilder::new_from_seed(
        payer,
        &token.airspace,
        &token.mint,
        seed,
        builder.authority,
        token.pyth_price,
        token.pyth_price,
        Some(fee_destination),
        orderbook,
    );

    log::info!(
        "create fixed-term market {}_{} at {}",
        &token.desc.name,
        config.borrow_tenor,
        ix_builder.market()
    );

    // TODO: should probably read the rent cost from the network
    let rent = Rent::default();

    // Intialize event/orderbook data accounts
    builder.setup([
        TransactionBuilder {
            instructions: vec![system_instruction::create_account(
                &payer,
                &key_eq.pubkey(),
                rent.minimum_balance(len_eq),
                len_eq as u64,
                &jet_fixed_term::ID,
            )],
            signers: vec![key_eq],
        },
        TransactionBuilder {
            instructions: vec![system_instruction::create_account(
                &payer,
                &key_bids.pubkey(),
                rent.minimum_balance(len_orders),
                len_orders as u64,
                &jet_fixed_term::ID,
            )],
            signers: vec![key_bids],
        },
        TransactionBuilder {
            instructions: vec![system_instruction::create_account(
                &payer,
                &key_asks.pubkey(),
                rent.minimum_balance(len_orders),
                len_orders as u64,
                &jet_fixed_term::ID,
            )],
            signers: vec![key_asks],
        },
    ]);

    builder.propose([
        ix_builder.initialize_market(
            builder.proposal_payer(),
            InitializeMarketParams {
                version_tag: 1,
                seed,
                borrow_tenor: config.borrow_tenor,
                lend_tenor: config.lend_tenor,
                origination_fee: config.origination_fee,
            },
        ),
        ix_builder.initialize_orderbook(builder.proposal_payer(), config.min_order_size),
    ]);

    Ok(ix_builder)
}
