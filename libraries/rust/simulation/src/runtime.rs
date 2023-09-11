use std::{cell::RefCell, collections::HashMap, sync::Arc, sync::Mutex};

use async_trait::async_trait;
use jet_solana_client::rpc::{AccountFilter, ClientError, ClientResult, SolanaRpc};
use lazy_static::lazy_static;

use solana_bpf_loader_program::serialization::{
    deserialize_parameters, deserialize_parameters_aligned, serialize_parameters,
};
use solana_program_runtime::{
    compute_budget::ComputeBudget, ic_logger_msg, invoke_context::InvokeContext, stable_log,
    sysvar_cache::SysvarCache, timings::ExecuteTimings,
};
use solana_runtime::{
    accounts_index::ScanConfig,
    bank::{Bank, TransactionLogCollectorFilter},
};
use solana_sdk::{
    account::Account,
    clock::Clock,
    compute_budget,
    entrypoint::SUCCESS,
    feature_set::FeatureSet,
    genesis_config::GenesisConfig,
    hash::Hash,
    instruction::InstructionError,
    packet::PACKET_DATA_SIZE,
    program_error::{ProgramError, UNSUPPORTED_SYSVAR},
    program_pack::Pack,
    program_stubs::SyscallStubs,
    pubkey::Pubkey,
    signature::Signature,
    slot_hashes::SlotHashes,
    sysvar::Sysvar,
    transaction::{
        MessageHash, SanitizedTransaction, Transaction, TransactionError, VersionedTransaction,
    },
};
use solana_transaction_status::{TransactionConfirmationStatus, TransactionStatus};
use spl_token::state::Account as TokenAccount;

use crate::log::declare_logging;

#[doc(hidden)]
pub use solana_sdk::entrypoint::ProcessInstruction;

pub type Entrypoint = fn(*mut u8) -> u64;

/// Utility for testing programs with the Solana runtime in-memory
#[derive(Clone)]
pub struct TestRuntime {
    bank: Arc<Bank>,
}

lazy_static! {
    static ref GLOBAL_PROGRAM_MAP: GlobalProgramMap = GlobalProgramMap::new();
}

thread_local! {
    static LOCAL_CONTEXTS: RefCell<Vec<LocalContext>> = RefCell::new(Vec::new());
}

declare_logging! {
    logging = "jet_simulation" {
        program         = "program";
        account_loader  = "acctldr";
        account_realloc = "realloc";
        program_data    = "pgmdata";
        program_return  = "pgmrtrn";
        instruction     = "ixhndlr";
        transaction     = "txhndlr";
        rpc             = "rpc";
        bank            = "bank";
        custom          = "_custom"; // for logs that are specific to the
                                     // simulated runtime, that you wouldn't
                                     // normally expect from a real validator.
    }
}
// intentionally shadows the crate to force proper logging
use logging as log;

#[derive(Clone, Copy)]
struct LocalContext {
    invoke_context: *mut (),
    param_memory: (*mut u8, usize),
    account_lens: (*const usize, usize),
}

impl LocalContext {
    fn invoke_context<'a>(&self) -> &'a mut InvokeContext<'a> {
        if self.invoke_context.is_null() {
            panic!("local context not set");
        }

        unsafe { &mut *(self.invoke_context as *mut InvokeContext) }
    }

    fn param_memory<'a>(&self) -> &'a mut [u8] {
        let (data, len) = self.param_memory;
        unsafe { std::slice::from_raw_parts_mut(data, len) }
    }

    fn accounts_lens<'a>(&self) -> &'a [usize] {
        let (data, len) = self.account_lens;
        unsafe { std::slice::from_raw_parts(data, len) }
    }
}

impl TestRuntime {
    pub fn new(
        native_programs: impl IntoIterator<Item = (Pubkey, ProcessInstruction)>,
        _sbf_programs: impl IntoIterator<Item = (Pubkey, Vec<u8>)>,
    ) -> Self {
        let mut bank = Bank::new_for_tests(&GenesisConfig::new(&[], &[]));
        let features = Arc::make_mut(&mut bank.feature_set);

        let programs = native_programs.into_iter().collect::<Vec<_>>();
        let program_ids = programs.iter().map(|(k, _)| *k).collect::<Vec<_>>();

        for (program, _) in &programs {
            log::bank::info!("loading program {program}");
        }

        GLOBAL_PROGRAM_MAP.insert(features, HashMap::from_iter(programs));

        bank.add_builtin("compute_budget", &compute_budget::ID, noop_handler);
        #[cfg(feature = "test-runtime")]
        bank.add_builtin(
            "address_lookup_table",
            &solana_address_lookup_table_program::ID,
            solana_address_lookup_table_program::processor::process_instruction,
        );

        bank.set_sysvar_for_tests(&SlotHashes::new(&[
            (0, Hash::new_unique()),
            (1, Hash::new_unique()),
        ]));

        bank.set_compute_budget(Some(ComputeBudget::default()));
        bank.set_rent_burn_percentage(100);
        bank.set_capitalization();

        let mut features = FeatureSet::clone(&bank.feature_set);
        features.activate(
            &solana_sdk::feature_set::versioned_tx_message_enabled::id(),
            0,
        );
        bank.feature_set = Arc::new(features);

        for program_id in program_ids {
            let ix_processor_name = format!("test-runtime:{program_id}");
            bank.add_builtin(&ix_processor_name, &program_id, global_instruction_handler);
        }

        bank.transaction_log_collector_config
            .write()
            .unwrap()
            .filter = TransactionLogCollectorFilter::All;

        solana_sdk::program_stubs::set_syscall_stubs(Box::new(LocalRuntimeSyscallStub));

        Self {
            bank: Arc::new(Bank::new_from_parent(
                &Arc::new(bank),
                &Pubkey::new_unique(),
                1,
            )),
        }
    }

    /// Set the state for an account
    pub fn set_account(&self, address: &Pubkey, account: &Account) {
        self.bank.store_account(address, account)
    }

    pub fn rpc(&self) -> TestRuntimeRpcClient {
        TestRuntimeRpcClient {
            manager: Arc::new(BankManager::new(self.bank.clone())),
        }
    }
}

/// Map of program handlers for each test context
struct GlobalProgramMap(Mutex<Vec<(Pubkey, HashMap<Pubkey, ProcessInstruction>)>>);

impl GlobalProgramMap {
    fn new() -> Self {
        Self(Mutex::new(Vec::new()))
    }

    fn insert(&self, feature: &mut FeatureSet, map: HashMap<Pubkey, ProcessInstruction>) {
        let mut bank_list = self.0.lock().unwrap();
        let id = Pubkey::new_unique();

        assert!(!bank_list.iter().any(|(k, _)| *k == id));
        bank_list.push((id, map));

        feature.activate(&id, 0);
    }

    fn get_programs(&self, feature: &FeatureSet) -> Option<HashMap<Pubkey, ProcessInstruction>> {
        let bank_list = self.0.lock().unwrap();

        for (bank_id, programs) in bank_list.iter() {
            if feature.is_active(bank_id) {
                return Some(programs.clone());
            }
        }

        None
    }
}

fn push_local_context(context: &InvokeContext, param_memory: &mut [u8], account_lens: &[usize]) {
    LOCAL_CONTEXTS.with(|local_contexts| unsafe {
        local_contexts.borrow_mut().push(LocalContext {
            invoke_context: std::mem::transmute(context),
            param_memory: (param_memory.as_mut_ptr(), param_memory.len()),
            account_lens: (account_lens.as_ptr(), account_lens.len()),
        });
    });
}

fn pop_local_context() {
    LOCAL_CONTEXTS.with(|local_contexts| local_contexts.borrow_mut().pop().unwrap());
}

fn get_local_context() -> LocalContext {
    LOCAL_CONTEXTS.with(|local_contexts| local_contexts.borrow().last().copied().unwrap())
}

fn get_local_context_height() -> usize {
    LOCAL_CONTEXTS.with(|local_contexts| local_contexts.borrow().len())
}

fn global_instruction_handler(
    _: usize,
    context: &mut InvokeContext,
) -> Result<(), InstructionError> {
    let programs = GLOBAL_PROGRAM_MAP
        .get_programs(&context.feature_set)
        .unwrap();

    let tx_context = &context.transaction_context;
    let ix_context = tx_context.get_current_instruction_context()?;

    let (mut memory, acc_lengths) = serialize_parameters(tx_context, ix_context, true)?;

    push_local_context(context, memory.as_slice_mut(), &acc_lengths);

    let program_id = ix_context.get_last_program_key(tx_context)?;
    let program_entrypoint = programs
        .get(program_id)
        .ok_or(InstructionError::IncorrectProgramId)?;

    log::instruction::debug!(
        "Program {} invoke [{}]",
        program_id,
        get_local_context_height()
    );

    let (program_id, accounts, instruction_data) =
        unsafe { solana_sdk::entrypoint::deserialize(&mut memory.as_slice_mut()[0]) };

    for account in &accounts {
        let signer_text = account.is_signer.then_some("SIGNER").unwrap_or_default();
        let mut_text = account.is_writable.then_some("MUTABLE").unwrap_or_default();

        log::account_loader::debug!(
            "Loaded Account {}: {} lamports, {} bytes, {} {}",
            account.key,
            account.lamports(),
            account.data_len(),
            mut_text,
            signer_text
        );
    }

    let result = program_entrypoint(program_id, &accounts, instruction_data);
    match result {
        Ok(()) => {
            log::instruction::debug!("Program {} success", program_id);
            stable_log::program_success(&context.get_log_collector(), program_id)
        }
        Err(err) => {
            let new_err = u64::from(err).into();

            log::instruction::debug!("Program {} failed: {}", program_id, &new_err);
            stable_log::program_failure(&context.get_log_collector(), program_id, &new_err);
            return Err(new_err);
        }
    };

    pop_local_context();

    let ix_context = tx_context.get_current_instruction_context()?;

    // commit changes
    deserialize_parameters_aligned(tx_context, ix_context, memory.as_slice(), &acc_lengths)
        .unwrap();

    Ok(())
}

fn noop_handler(_: usize, _: &mut InvokeContext) -> Result<(), InstructionError> {
    Ok(())
}

struct LocalRuntimeSyscallStub;

impl SyscallStubs for LocalRuntimeSyscallStub {
    fn sol_log(&self, message: &str) {
        log::program::debug!("Program log: {}", message);
        ic_logger_msg!(
            get_local_context().invoke_context().get_log_collector(),
            "Program log: {}",
            message
        );
    }

    fn sol_log_data(&self, fields: &[&[u8]]) {
        for field in fields {
            let data = base64::encode(field);
            log::program_data::debug!("Program data: {}", data);
            ic_logger_msg!(
                get_local_context().invoke_context().get_log_collector(),
                "Program data: {}",
                data
            );
        }
    }

    fn sol_invoke_signed(
        &self,
        instruction: &solana_sdk::instruction::Instruction,
        account_infos: &[solana_sdk::account_info::AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> solana_sdk::entrypoint::ProgramResult {
        let context = get_local_context().invoke_context();
        let ix_context = context
            .transaction_context
            .get_current_instruction_context()
            .unwrap();

        // commit changes from current program
        let buffer = get_local_context().param_memory();
        let account_lengths = get_local_context().accounts_lens();
        deserialize_parameters(
            context.transaction_context,
            ix_context,
            buffer,
            account_lengths,
        )
        .unwrap();

        let caller = ix_context
            .get_last_program_key(context.transaction_context)
            .unwrap();
        let signers = signers_seeds
            .iter()
            .map(|s| Pubkey::create_program_address(s, caller).unwrap())
            .collect::<Vec<_>>();
        let (ix_accounts, program_idx) =
            context.prepare_instruction(instruction, &signers).unwrap();

        let mut compute_consumed = 0;
        context
            .process_instruction(
                &instruction.data,
                &ix_accounts,
                &program_idx,
                &mut compute_consumed,
                &mut ExecuteTimings::default(),
            )
            .map_err(|e| ProgramError::try_from(e).unwrap())?;

        // copy changes to current instruction's loaded accounts
        for ix_account in ix_accounts {
            let tx_context = &context.transaction_context;
            let ix_context = tx_context.get_current_instruction_context().unwrap();
            let key = tx_context
                .get_key_of_account_at_index(ix_account.index_in_transaction)
                .unwrap();

            let account_info = account_infos.iter().find(|info| info.key == key).unwrap();

            let tx_account = ix_context
                .try_borrow_instruction_account(tx_context, ix_account.index_in_caller)
                .unwrap();

            if !account_info.is_writable {
                continue;
            }

            **account_info.try_borrow_mut_lamports().unwrap() = tx_account.get_lamports();
            account_info.assign(tx_account.get_owner());

            let acc_data = tx_account.get_data();

            if account_info.data_len() != acc_data.len()
                && tx_account.can_data_be_resized(acc_data.len()).is_ok()
            {
                log::account_realloc::debug!(
                    "acc realloc {}: {} -> {}",
                    account_info.key,
                    account_info.data_len(),
                    acc_data.len()
                );

                account_info.realloc(acc_data.len(), false).unwrap();
            }

            if tx_account.can_data_be_changed().is_ok() {
                unsafe {
                    (*account_info.data.as_ptr()).clone_from_slice(tx_account.get_data());
                }
            }
        }

        Ok(())
    }

    fn sol_get_clock_sysvar(&self, var_addr: *mut u8) -> u64 {
        read_sysvar(var_addr, |cache| cache.get_clock().ok())
    }

    fn sol_get_rent_sysvar(&self, var_addr: *mut u8) -> u64 {
        read_sysvar(var_addr, |cache| cache.get_rent().ok())
    }

    fn sol_get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
        let (program_id, data) = get_local_context()
            .invoke_context()
            .transaction_context
            .get_return_data();

        if data.is_empty() {
            return None;
        }

        Some((*program_id, data.to_vec()))
    }

    fn sol_set_return_data(&self, data: &[u8]) {
        let context = get_local_context().invoke_context();
        let tx_context = &mut context.transaction_context;
        let caller = tx_context
            .get_current_instruction_context()
            .unwrap()
            .get_last_program_key(tx_context)
            .unwrap();

        if data.is_empty() {
            let encoded = base64::encode(data);
            log::program_return::debug!("Program return: {}", encoded);

            ic_logger_msg!(
                get_local_context().invoke_context().get_log_collector(),
                "Program return: {}",
                encoded
            );
        }

        tx_context.set_return_data(*caller, data.to_vec()).unwrap();
    }

    fn sol_get_stack_height(&self) -> u64 {
        let context = get_local_context().invoke_context();
        context.get_stack_height() as u64
    }
}

fn read_sysvar<F, T>(target: *mut u8, get_fn: F) -> u64
where
    T: Sysvar + Clone,
    F: Fn(&SysvarCache) -> Option<Arc<T>>,
{
    let context = get_local_context();
    match get_fn(context.invoke_context().get_sysvar_cache()) {
        None => UNSUPPORTED_SYSVAR,
        Some(val) => {
            unsafe {
                *(target as *mut _ as *mut T) = T::clone(&val);
            }

            SUCCESS
        }
    }
}

fn send_legacy_transaction(
    bank: &Arc<Bank>,
    transaction: &Transaction,
) -> Result<Signature, TransactionError> {
    let serialized_len = transaction.message.serialize().len();
    assert!(
        serialized_len < PACKET_DATA_SIZE,
        "tx size too large: {} (limit {})",
        serialized_len,
        PACKET_DATA_SIZE
    );

    let signature = transaction.signatures[0];
    let tx = SanitizedTransaction::from_transaction_for_tests(transaction.clone());

    log::transaction::info!("processing transaction {}", transaction.signatures[0]);
    transaction.verify()?;

    let sim_result = bank.simulate_transaction_unchecked(tx);

    match sim_result.result {
        Ok(()) => {
            bank.process_transaction_with_logs(transaction).unwrap();
            bank.register_recent_blockhash(&Hash::new_unique());

            Ok(signature)
        }

        Err(e) => {
            log::transaction::error!("tx error {signature}: {e:?}");
            log::transaction::error!("{:#?}", sim_result.logs);

            Err(e)
        }
    }
}

fn send_transaction(
    bank: &Arc<Bank>,
    transaction: &VersionedTransaction,
) -> Result<Signature, TransactionError> {
    let serialized_len = transaction.message.serialize().len();
    assert!(
        serialized_len < PACKET_DATA_SIZE,
        "tx size too large: {} (limit {})",
        serialized_len,
        PACKET_DATA_SIZE
    );

    let signature = transaction.signatures[0];
    let tx = SanitizedTransaction::try_create(
        transaction.clone(),
        MessageHash::Compute,
        None,
        &**bank,
        true,
    )?;

    log::transaction::info!("processing transaction {}", transaction.signatures[0]);
    transaction.verify_and_hash_message()?;

    let sim_result = bank.simulate_transaction_unchecked(tx);

    match sim_result.result {
        Ok(()) => {
            bank.process_entry_transactions(vec![transaction.clone()])
                .pop()
                .unwrap()?;
            bank.register_recent_blockhash(&Hash::new_unique());

            Ok(signature)
        }

        Err(e) => {
            log::transaction::error!("tx error {signature}: {e:?}");
            log::transaction::error!("{:#?}", sim_result.logs);

            Err(e)
        }
    }
}

struct BankManager {
    bank: Mutex<Arc<Bank>>,
}

impl BankManager {
    fn new(bank: Arc<Bank>) -> Self {
        Self {
            bank: Mutex::new(bank),
        }
    }

    fn complete_block(&self) {
        let mut bank = self.bank.lock().unwrap();

        bank.register_tick(&Hash::new_unique());

        *bank = Arc::new(Bank::new_from_parent(
            &bank,
            &Pubkey::new_unique(),
            bank.slot() + 1,
        ));

        log::bank::info!("new bank at slot {}", bank.slot());
    }
}

#[derive(Clone)]
pub struct TestRuntimeRpcClient {
    manager: Arc<BankManager>,
}

impl TestRuntimeRpcClient {
    fn bank(&self) -> Arc<Bank> {
        self.manager.bank.lock().unwrap().clone()
    }

    pub fn set_clock(&self, new_clock: &Clock) {
        self.bank().set_sysvar_for_tests(new_clock);
    }

    pub fn next_block(&self) {
        self.manager.complete_block();
    }
}

#[async_trait]
impl SolanaRpc for TestRuntimeRpcClient {
    async fn get_genesis_hash(&self) -> ClientResult<Hash> {
        Ok(self.bank().parent_hash())
    }

    async fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        Ok(self.bank().last_blockhash())
    }

    async fn get_slot(&self) -> ClientResult<u64> {
        Ok(self.bank().slot())
    }

    async fn get_block_time(&self, slot: u64) -> ClientResult<i64> {
        let timestamp = match slot {
            n if n == self.bank().slot() => {
                self.bank()
                    .get_sysvar_cache_for_tests()
                    .get_clock()
                    .unwrap()
                    .unix_timestamp
            }
            // FIXME: use correct bank timestamp
            _ => self.bank().unix_timestamp_from_genesis(),
        };

        log::rpc::trace!("get_block_time({}) = {}", slot, timestamp);

        Ok(timestamp)
    }

    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> ClientResult<Vec<Option<Account>>> {
        Ok(pubkeys
            .iter()
            .map(|pubkey| {
                let account_data = self.bank().get_account(pubkey).map(Account::from);

                match &account_data {
                    None => log::rpc::trace!("get_account({pubkey}) = None"),
                    Some(account) => log::rpc::trace!(
                        "get_account({pubkey}) = {{ lamports: {}, data: [{} bytes] }}",
                        account.lamports,
                        account.data.len()
                    ),
                }

                account_data
            })
            .collect())
    }

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> ClientResult<Vec<Option<TransactionStatus>>> {
        Ok(signatures
            .iter()
            .map(|sig| {
                self.bank()
                    .get_signature_status(sig)
                    .map(|status| TransactionStatus {
                        slot: self.bank().slot(),
                        confirmations: Some(1),
                        err: status.clone().err(),
                        status,
                        confirmation_status: Some(TransactionConfirmationStatus::Processed),
                    })
            })
            .collect())
    }

    async fn airdrop(&self, account: &Pubkey, lamports: u64) -> ClientResult<()> {
        match self.bank().deposit(account, lamports) {
            Err(e) => return Err(ClientError::Other(format!("airdrop failed: {:?}", e))),
            Ok(_) => Ok(()),
        }
    }

    async fn send_transaction_legacy(&self, transaction: &Transaction) -> ClientResult<Signature> {
        self.next_block();
        Ok(send_legacy_transaction(&self.bank(), transaction)?)
    }

    async fn send_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> ClientResult<Signature> {
        self.next_block();
        Ok(send_transaction(&self.bank(), transaction)?)
    }

    async fn get_program_accounts(
        &self,
        program: &Pubkey,
        filters: &[AccountFilter],
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        let accounts = self
            .bank()
            .get_program_accounts(program, &ScanConfig::default())
            .unwrap()
            .into_iter()
            .filter_map(|(address, account)| {
                let account = account.into();

                if filters.iter().all(|filter| filter.matches(&account)) {
                    Some((address, account))
                } else {
                    None
                }
            })
            .collect();

        log::rpc::trace!(
            "get_program_accounts({}, {:?}) = {:#?}",
            program,
            filters,
            accounts
        );
        Ok(accounts)
    }

    async fn get_token_accounts_by_owner(
        &self,
        owner: &Pubkey,
    ) -> Result<Vec<(Pubkey, TokenAccount)>, ClientError> {
        let accounts = self
            .bank()
            .get_program_accounts(&spl_token::id(), &ScanConfig::default())
            .unwrap()
            .into_iter()
            .filter_map(|(address, account)| {
                let account: Account = account.into();

                if let Ok(token_account) = TokenAccount::unpack(&account.data) {
                    if token_account.owner == *owner {
                        return Some((address, token_account));
                    }
                }

                None
            })
            .collect();

        log::rpc::trace!("get_token_accounts_by_owner({}) = {:#?}", owner, accounts);
        Ok(accounts)
    }

    async fn wait_for_slot(&self, slot: u64) -> ClientResult<()> {
        while self.bank().slot() < slot {
            self.next_block();
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! create_test_runtime {
    [$($program:tt),+$(,)?] => {{
        let mut programs = vec![];
        $(programs.push($crate::program!($program));)+
        $crate::TestRuntime::new(programs, [])
    }}
}

#[macro_export]
macro_rules! program {
    ($krate:ident) => {{
        (
            $krate::id(),
            $krate::entry as $crate::runtime::ProcessInstruction,
        )
    }};
    (($id:expr, $processor:path)) => {{
        ($id, $processor)
    }};
}

#[cfg(test)]
mod test {
    use solana_sdk::{
        native_token::LAMPORTS_PER_SOL,
        signature::{Keypair, Signer},
        system_transaction,
    };

    use super::*;

    #[tokio::test]
    async fn can_simulate_simple_transfer() {
        let rt = TestRuntime::new([], []);
        let rpc = rt.rpc();

        let source_wallet = Keypair::new();
        let dest_wallet = Keypair::new();

        SolanaRpc::airdrop(&rpc, &source_wallet.pubkey(), 421 * LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let recent_blockhash = SolanaRpc::get_latest_blockhash(&rpc).await.unwrap();
        let transfer_tx = system_transaction::transfer(
            &source_wallet,
            &dest_wallet.pubkey(),
            420 * LAMPORTS_PER_SOL,
            recent_blockhash,
        );

        SolanaRpc::send_transaction_legacy(&rpc, &transfer_tx)
            .await
            .unwrap();

        let dest_balance = SolanaRpc::get_account(&rpc, &dest_wallet.pubkey())
            .await
            .unwrap()
            .unwrap()
            .lamports;

        assert_eq!(420 * LAMPORTS_PER_SOL, dest_balance);
    }
}
