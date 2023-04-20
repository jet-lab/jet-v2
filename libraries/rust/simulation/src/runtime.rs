use std::{cell::RefCell, collections::HashMap, sync::Arc, sync::Mutex};

use anyhow::{anyhow, bail};
use async_trait::async_trait;
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
    account::{Account, ReadableAccount},
    clock::Clock,
    commitment_config::CommitmentConfig,
    compute_budget,
    entrypoint::SUCCESS,
    feature_set::FeatureSet,
    genesis_config::GenesisConfig,
    hash::Hash,
    instruction::InstructionError,
    program_error::{ProgramError, UNSUPPORTED_SYSVAR},
    program_stubs::SyscallStubs,
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signature},
    slot_history::Slot,
    sysvar::Sysvar,
    transaction::{SanitizedTransaction, Transaction, TransactionError},
};
use solana_transaction_status::{TransactionConfirmationStatus, TransactionStatus};

use crate::{log::declare_logging, solana_rpc_api::SolanaRpcClient};

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
        let mut bank = Bank::new_no_wallclock_throttle_for_tests(&GenesisConfig::new(&[], &[]));
        let features = Arc::make_mut(&mut bank.feature_set);

        let programs = native_programs.into_iter().collect::<Vec<_>>();
        let program_ids = programs.iter().map(|(k, _)| *k).collect::<Vec<_>>();

        GLOBAL_PROGRAM_MAP.insert(features, HashMap::from_iter(programs.into_iter()));

        bank.add_builtin("compute_budget", &compute_budget::ID, noop_handler);
        #[cfg(feature = "test-runtime")]
        bank.add_builtin(
            "address_lookup_table",
            &solana_address_lookup_table_program::ID,
            solana_address_lookup_table_program::processor::process_instruction,
        );

        bank.set_compute_budget(Some(ComputeBudget::default()));
        bank.set_capitalization();

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
            bank: Arc::new(bank),
        }
    }

    /// Set the state for an account
    pub fn set_account(&self, address: &Pubkey, account: &Account) {
        self.bank.store_account(address, account)
    }

    pub fn rpc(&self, payer: Keypair) -> TestRuntimeRpcClient {
        TestRuntimeRpcClient {
            bank: self.bank.clone(),
            payer,
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

pub struct TestRuntimeRpcClient {
    bank: Arc<Bank>,
    payer: Keypair,
}

impl TestRuntimeRpcClient {
    pub fn clone_with_payer(&self, payer: Keypair) -> Self {
        Self {
            bank: self.bank.clone(),
            payer,
        }
    }
}

#[async_trait]
impl SolanaRpcClient for TestRuntimeRpcClient {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn clone_with_payer(&self, payer: Keypair) -> Box<dyn SolanaRpcClient> {
        Box::new(Self::clone_with_payer(self, payer))
    }

    async fn get_account(&self, address: &Pubkey) -> anyhow::Result<Option<Account>> {
        Ok(self.bank.get_account(address).map(|a| a.into()))
    }

    async fn get_multiple_accounts(
        &self,
        addresses: &[Pubkey],
    ) -> anyhow::Result<Vec<Option<Account>>> {
        Ok(addresses
            .iter()
            .map(|addr| self.bank.get_account(addr).map(|a| a.into()))
            .collect())
    }

    async fn get_genesis_hash(&self) -> anyhow::Result<Hash> {
        Ok(self.bank.last_blockhash())
    }

    async fn get_latest_blockhash(&self) -> anyhow::Result<Hash> {
        self.bank.register_recent_blockhash(&Hash::new_unique());

        Ok(self.bank.last_blockhash())
    }

    async fn get_minimum_balance_for_rent_exemption(&self, length: usize) -> anyhow::Result<u64> {
        Ok(Rent::default().minimum_balance(length))
    }

    async fn send_transaction(
        &self,
        transaction: &Transaction,
    ) -> anyhow::Result<solana_sdk::signature::Signature> {
        let signature = transaction.signatures[0];
        let tx = SanitizedTransaction::from_transaction_for_tests(transaction.clone());

        log::transaction::info!("processing transaction {}", transaction.signatures[0]);
        transaction.verify()?;

        let sim_result = self.bank.simulate_transaction_unchecked(tx);

        match sim_result.result {
            Ok(()) => self
                .bank
                .process_transaction_with_logs(transaction)
                .unwrap(),

            Err(e) => {
                log::transaction::error!("tx error {signature}: {e:?}");
                log::transaction::error!("{:#?}", sim_result.logs);

                if let TransactionError::InstructionError(_, error) = &e {
                    if let Ok(error) = ProgramError::try_from(error.clone()) {
                        return Err(anyhow!("program error: {error}").context(error));
                    }
                }

                bail!("failed: {e}");
            }
        }

        Ok(signature)
    }

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> anyhow::Result<Vec<Option<TransactionStatus>>> {
        Ok(signatures
            .iter()
            .map(|_| {
                Some(TransactionStatus {
                    slot: self.bank.slot(),
                    err: None,
                    confirmations: Some(1),
                    confirmation_status: Some(TransactionConfirmationStatus::Processed),
                    status: Ok(()),
                })
            })
            .collect())
    }

    async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        size: Option<usize>,
    ) -> anyhow::Result<Vec<(Pubkey, Account)>> {
        Ok(self
            .bank
            .get_program_accounts(program_id, &ScanConfig::default())?
            .into_iter()
            .filter_map(|(address, account)| match (size, account.data().len()) {
                (Some(target), length) if target != length => None,
                _ => Some((address, account.into())),
            })
            .collect())
    }

    async fn airdrop(&self, account: &Pubkey, amount: u64) -> anyhow::Result<()> {
        self.bank.deposit(account, amount)?;
        Ok(())
    }

    async fn get_clock(&self) -> anyhow::Result<Clock> {
        let sysvar = self
            .bank
            .get_account(&solana_sdk::sysvar::clock::ID)
            .unwrap();

        let clock = bincode::deserialize(sysvar.data())?;

        log::custom::debug!("time is {:?}", &clock);

        Ok(clock)
    }

    async fn set_clock(&self, new_clock: Clock) -> anyhow::Result<()> {
        self.bank.set_sysvar_for_tests(&new_clock);
        Ok(())
    }

    async fn get_slot(&self, _commitment_config: Option<CommitmentConfig>) -> anyhow::Result<Slot> {
        // just return the slot from the latest updated clock
        Ok(self.get_clock().await?.slot)
    }

    fn payer(&self) -> &Keypair {
        &self.payer
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
        let payer = Keypair::new();
        let rpc = rt.rpc(payer);

        let source_wallet = Keypair::new();
        let dest_wallet = Keypair::new();

        rpc.airdrop(&source_wallet.pubkey(), 421 * LAMPORTS_PER_SOL)
            .await
            .unwrap();

        let recent_blockhash = rpc.get_latest_blockhash().await.unwrap();
        let transfer_tx = system_transaction::transfer(
            &source_wallet,
            &dest_wallet.pubkey(),
            420 * LAMPORTS_PER_SOL,
            recent_blockhash,
        );

        rpc.send_transaction(&transfer_tx).await.unwrap();

        let dest_balance = rpc
            .get_account(&dest_wallet.pubkey())
            .await
            .unwrap()
            .unwrap()
            .lamports;

        assert_eq!(420 * LAMPORTS_PER_SOL, dest_balance);
    }
}
