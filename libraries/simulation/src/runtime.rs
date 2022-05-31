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

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use solana_sdk::account::Account as StoredAccount;
use solana_sdk::account_info::AccountInfo;
use solana_sdk::clock::Clock;
use solana_sdk::entrypoint::SUCCESS;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::msg;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::packet::PACKET_DATA_SIZE;
use solana_sdk::program_error::ProgramError;
use solana_sdk::program_stubs::{set_syscall_stubs, SyscallStubs};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction::{SystemInstruction, MAX_PERMITTED_DATA_LENGTH};
use solana_sdk::system_program::{self, ID as SYSTEM_PROGRAM_ID};
use solana_sdk::transaction::Transaction;

use jet_solana_rpc_api::SolanaRpcClient;
use solana_transaction_status::{TransactionConfirmationStatus, TransactionStatus};

const ACCOUNT_TABLE_SIZE: usize = 10_240;

pub type EntryFn =
    Box<dyn Fn(&Pubkey, &[AccountInfo], &[u8]) -> Result<(), ProgramError> + Send + Sync>;

/// A mock of the Solana runtime, which can be used to simulate programs in
/// an in-memory testing environment.
pub struct TestRuntime(Arc<RuntimeContext>);

impl TestRuntime {
    /// Initialize a new testing runtime with the given set of program entrypoints.
    pub fn new(programs: impl IntoIterator<Item = (Pubkey, EntryFn)>) -> Self {
        let mut programs = programs.into_iter().collect::<HashMap<_, _>>();

        programs
            .entry(spl_token::ID)
            .or_insert_with(|| Box::new(spl_token::processor::Processor::process));

        programs
            .entry(spl_token_swap::ID)
            .or_insert_with(|| Box::new(spl_token_swap::processor::Processor::process));

        programs
            .entry(spl_associated_token_account::ID)
            .or_insert_with(|| {
                Box::new(spl_associated_token_account::processor::process_instruction)
            });

        let accounts = Mutex::new(HashMap::with_capacity(ACCOUNT_TABLE_SIZE));
        let signatures = Mutex::new(HashMap::new());
        let call_stack = parking_lot::ReentrantMutex::new(RefCell::new(vec![]));
        let clock = Mutex::new(Clock::default());
        let return_data = Mutex::new(None);
        let ctx = Arc::new(RuntimeContext {
            programs,
            accounts,
            signatures,
            call_stack,
            clock,
            return_data,
        });

        ctx.create_account(
            solana_sdk::sysvar::rent::ID,
            SYSTEM_PROGRAM_ID,
            vec![
                0x98, 0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x40, 0x64,
            ],
            0,
        );

        let runtime = Box::new(RuntimeStubs(ctx.clone()));

        set_syscall_stubs(runtime);

        Self(ctx)
    }

    /// Get the minimum lamport balance needed to remain rent-exempt
    ///
    /// This mock runtime doesn't enforce or check for rent in any way,
    /// but this is provided to satisfy the programs that may check for the
    /// rent balance themselves.
    pub fn minimum_rent_balance(&self, size: usize) -> u64 {
        solana_sdk::rent::Rent::default().minimum_balance(size)
    }

    /// Create a new account within this runtime context
    ///
    /// This can be used to intialize an account with any aribtrary data,
    /// without having to use any program instructions.
    pub fn create_account(&self, address: Pubkey, owner: Pubkey, data: Vec<u8>, lamports: u64) {
        self.0.create_account(address, owner, data, lamports)
    }

    /// Create a new account within this runtime context, with enough lamports
    /// to be considered 'rent-exempt'
    pub fn create_account_rent_exempt(&self, address: Pubkey, owner: Pubkey, data: Vec<u8>) {
        let lamports = self.minimum_rent_balance(data.len());

        self.0.create_account(address, owner, data, lamports)
    }

    /// Simulate the execution of a transaction within this runtime context
    pub fn execute_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<Signature, (usize, ProgramError)> {
        let serialized = transaction.message.serialize();
        let message = &transaction.message;

        let serialized_len = serialized.len();
        assert!(
            serialized_len <= PACKET_DATA_SIZE,
            "Transaction too big, packet size is {}, transaction is {}",
            PACKET_DATA_SIZE,
            serialized_len
        );

        for (i, ci) in message.instructions.iter().enumerate() {
            let accounts = ci
                .accounts
                .iter()
                .map(|a| {
                    let key = *a as usize;
                    AccountMeta {
                        pubkey: message.account_keys[key],
                        is_signer: message.is_signer(key),
                        is_writable: message.is_writable(key),
                    }
                })
                .collect::<Vec<_>>();

            let instruction = Instruction {
                accounts,
                data: ci.data.clone(),
                program_id: message.account_keys[ci.program_id_index as usize],
            };

            self.invoke(&instruction).map_err(|e| (i, e))?;
        }

        Ok(self.0.create_signature(true))
    }

    /// Get the reference to the stored account data for a given address
    ///
    /// This can be used to directly read/modify account data.
    pub fn get_account_info<'a>(&'a self, address: &'a Pubkey) -> AccountInfo<'a> {
        let mut accounts = self.0.accounts.lock().unwrap();

        if !accounts.contains_key(address) {
            accounts.insert(
                *address,
                RefCell::new(StoredAccount {
                    data: vec![],
                    owner: SYSTEM_PROGRAM_ID,
                    lamports: 0,
                    executable: false,
                    rent_epoch: 0,
                }),
            );
        }

        unsafe {
            let account_ref = accounts.get(address).unwrap();
            let account = &mut *account_ref.as_ptr();

            println!(
                "LOAD ACCOUNT {}: {} bytes, {} lamports",
                address,
                account.data.len(),
                account.lamports
            );

            AccountInfo {
                key: address,
                owner: &account.owner,
                data: Rc::new(RefCell::new(&mut account.data)),
                lamports: Rc::new(RefCell::new(&mut account.lamports)),
                executable: self.0.is_executable(address),
                is_writable: false,
                is_signer: false,
                rent_epoch: account.rent_epoch,
            }
        }
    }

    /// Get the current clock set in the mocked runtime
    pub fn get_clock(&self) -> Clock {
        self.0.clock.lock().unwrap().clone()
    }

    /// Set the current clock to be returned by the mocked runtime
    pub fn set_clock(&self, new_clock: Clock) {
        *self.0.clock.lock().unwrap() = new_clock;
    }

    /// Confirm a previous transaction was actually executed
    pub fn confirm_transaction(&self, signature: &Signature) -> Option<bool> {
        self.0.confirm_signature(signature)
    }

    /// Get status of a previously executed transaction
    pub fn get_signature_status(&self, signature: &Signature) -> Option<TransactionStatus> {
        self.0.get_signature_status(signature)
    }

    /// Get the accounts owned by a program
    pub fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        size: Option<usize>,
    ) -> Vec<(Pubkey, StoredAccount)> {
        self.0
            .accounts
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(address, cell)| {
                let account = cell.borrow();
                let is_program_account = account.owner == *program_id;
                let is_requested_size = size.map(|s| s == account.data.len()).unwrap_or(true);

                (is_program_account && is_requested_size).then(|| (*address, account.clone()))
            })
            .collect()
    }

    fn invoke(&self, instruction: &Instruction) -> Result<(), ProgramError> {
        let account_infos = instruction
            .accounts
            .iter()
            .map(|meta| {
                let mut info = self.get_account_info(&meta.pubkey);

                info.is_signer = meta.is_signer;
                info.is_writable = meta.is_writable;

                info
            })
            .collect::<Vec<_>>();

        RuntimeStubs(self.0.clone()).sol_invoke_signed(instruction, &account_infos, &[])
    }
}

struct RuntimeContext {
    programs: HashMap<Pubkey, EntryFn>,
    accounts: Mutex<HashMap<Pubkey, RefCell<StoredAccount>>>,
    signatures: Mutex<HashMap<Signature, bool>>,
    call_stack: parking_lot::ReentrantMutex<RefCell<Vec<Pubkey>>>,
    clock: Mutex<Clock>,
    return_data: Mutex<Option<(Pubkey, Vec<u8>)>>,
}

impl RuntimeContext {
    fn create_signature(&self, success: bool) -> Signature {
        let signature = Signature::new_unique();
        let mut signatures = self.signatures.lock().unwrap();

        signatures.insert(signature, success);

        signature
    }

    fn confirm_signature(&self, signature: &Signature) -> Option<bool> {
        self.signatures.lock().unwrap().get(signature).cloned()
    }

    pub fn get_signature_status(&self, signature: &Signature) -> Option<TransactionStatus> {
        self.signatures
            .lock()
            .unwrap()
            .get(signature)
            .map(|_| TransactionStatus {
                status: Ok(()),
                err: None,
                slot: 0,
                confirmations: Some(1),
                confirmation_status: Some(TransactionConfirmationStatus::Confirmed),
            })
    }

    fn create_account(&self, address: Pubkey, owner: Pubkey, data: Vec<u8>, lamports: u64) {
        let mut accounts = self.accounts.lock().unwrap();

        match accounts.insert(
            address,
            RefCell::new(StoredAccount {
                data,
                owner,
                lamports,
                executable: false,
                rent_epoch: 0,
            }),
        ) {
            None => (),
            Some(a) if a.borrow().lamports == 0 => (),
            Some(_) => panic!("account already created"),
        }
    }

    fn handle_system_instruction(
        &self,
        instr: &Instruction,
        account_infos: &[AccountInfo],
    ) -> Result<(), ProgramError> {
        let parameter: SystemInstruction = bincode::deserialize(&instr.data).unwrap();

        match parameter {
            SystemInstruction::CreateAccount {
                owner,
                space,
                lamports,
            } => self.system_create_account(account_infos, owner, space, lamports),

            SystemInstruction::Transfer { lamports } => {
                self.system_transfer(account_infos, lamports)
            }

            SystemInstruction::Assign { owner } => self.system_assign(account_infos, owner),
            SystemInstruction::Allocate { space } => self.system_allocate(account_infos, space),

            _ => panic!("not supported"),
        }
    }

    fn system_create_account(
        &self,
        accounts: &[AccountInfo],
        owner: Pubkey,
        space: u64,
        lamports: u64,
    ) -> Result<(), ProgramError> {
        self.system_allocate(&accounts[1..], space)?;
        self.system_assign(&accounts[1..], owner)?;
        self.system_transfer(accounts, lamports)?;

        Ok(())
    }

    fn system_allocate(&self, accounts: &[AccountInfo], space: u64) -> Result<(), ProgramError> {
        if !accounts[0].is_signer {
            msg!("allocate: account {:?} must sign", accounts[0].key);
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !accounts[0].data_is_empty() || !system_program::check_id(accounts[0].owner) {
            msg!("allocate: account {:?} in use", accounts[0].key);
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        if space > MAX_PERMITTED_DATA_LENGTH {
            msg!("allocate: data too large");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut accounts_db = self.accounts.lock().unwrap();

        if !accounts_db.contains_key(accounts[0].owner) {
            accounts_db.insert(
                *accounts[0].key,
                RefCell::new(StoredAccount {
                    data: vec![0u8; space as usize],
                    owner: SYSTEM_PROGRAM_ID,
                    lamports: 0,
                    executable: false,
                    rent_epoch: 0,
                }),
            );
        }

        unsafe {
            // make sure the accounts map isn't reallocated
            assert!(accounts_db.len() <= ACCOUNT_TABLE_SIZE);

            let stored_account = accounts_db.get(accounts[0].key).unwrap();
            let stored_account_ref = &mut *stored_account.as_ptr();

            stored_account_ref.data.resize(space as usize, 0u8);

            accounts[0].data.replace(&mut stored_account_ref.data);
            accounts[0]
                .lamports
                .replace(&mut stored_account_ref.lamports);
        }

        Ok(())
    }

    fn system_transfer(&self, accounts: &[AccountInfo], lamports: u64) -> Result<(), ProgramError> {
        let source = &accounts[0];
        let dest = &accounts[1];

        **source.lamports.borrow_mut() = source.lamports().checked_sub(lamports).unwrap();
        **dest.lamports.borrow_mut() = dest.lamports().checked_add(lamports).unwrap();

        Ok(())
    }

    fn system_assign(
        &self,
        accounts: &[AccountInfo],
        new_owner: Pubkey,
    ) -> Result<(), ProgramError> {
        accounts[0].assign(&new_owner);
        Ok(())
    }

    fn is_executable(&self, address: &Pubkey) -> bool {
        self.programs.contains_key(address) || (*address == SYSTEM_PROGRAM_ID)
    }
}

struct RuntimeStubs(Arc<RuntimeContext>);

impl SyscallStubs for RuntimeStubs {
    fn sol_get_rent_sysvar(&self, var_addr: *mut u8) -> u64 {
        use solana_sdk::rent::Rent;

        unsafe {
            *(var_addr as *mut _ as *mut Rent) = Rent::default();
        }

        SUCCESS
    }

    fn sol_get_clock_sysvar(&self, var_addr: *mut u8) -> u64 {
        unsafe {
            *(var_addr as *mut _ as *mut Clock) = self.0.clock.lock().unwrap().clone();
        }

        SUCCESS
    }

    fn sol_log(&self, message: &str) {
        let stack_len = self.0.call_stack.lock().borrow().len();
        let stack_prefix = format!("[{}]", stack_len);

        println!("{} {}", stack_prefix, message);
    }

    fn sol_invoke_signed(
        &self,
        instruction: &Instruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> Result<(), ProgramError> {
        let mut new_account_infos = vec![];
        let call_stack = self.0.call_stack.lock();

        for meta in instruction.accounts.iter() {
            let info = accounts.iter().find(|i| *i.key == meta.pubkey).unwrap();
            let mut new_account_info = info.clone();

            // Mark accounts as having a signer if the seeds have been provided
            for seeds in signers_seeds {
                let signed_address =
                    Pubkey::create_program_address(seeds, call_stack.borrow().last().unwrap())?;

                if signed_address == *info.key {
                    msg!("signed {}", signed_address);
                    new_account_info.is_signer = true;
                }
            }

            new_account_infos.push(new_account_info);
        }

        assert!(call_stack.borrow().len() < 4);
        call_stack.borrow_mut().push(instruction.program_id);

        msg!(
            "Program {} invoke [{}]",
            instruction.program_id,
            call_stack.borrow().len()
        );

        let result = match self.0.programs.get(&instruction.program_id) {
            None if instruction.program_id == Pubkey::default() => self
                .0
                .handle_system_instruction(instruction, &new_account_infos),

            None => Err(ProgramError::IncorrectProgramId),
            Some(program) => program(
                &instruction.program_id,
                &new_account_infos,
                &instruction.data,
            ),
        };

        let result_text = match &result {
            &Ok(_) => "success",
            _ => "failed",
        };

        msg!(
            "Program {} {} [{}]",
            instruction.program_id,
            result_text,
            call_stack.borrow().len()
        );
        call_stack.borrow_mut().pop();

        result
    }

    fn sol_set_return_data(&self, data: &[u8]) {
        let call_stack = self.0.call_stack.lock();
        let mut storage = self.0.return_data.lock().unwrap();

        assert!(data.len() <= 1024);

        let program_id = *call_stack.borrow().last().unwrap();
        *storage = Some((program_id, data.to_vec()));
    }

    fn sol_get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
        let storage = self.0.return_data.lock().unwrap();
        storage.clone()
    }
}

#[macro_export]
macro_rules! create_test_runtime {
    [$( $krate:ident ),+] => {{

        let mut programs = vec![];

        $({
            let entry_fn: $crate::EntryFn = Box::new($krate::entry);
            programs.push(($krate::id(), entry_fn));
        })+

        TestRuntime::new(programs)
    }}
}

#[async_trait]
impl SolanaRpcClient for crate::TestRuntime {
    async fn get_account(&self, address: &Pubkey) -> anyhow::Result<Option<StoredAccount>> {
        let info = self.get_account_info(address);

        if info.data_is_empty() {
            return Ok(None);
        }

        let lamports = **info.lamports.borrow();
        let data = info.data.borrow().to_vec();

        Ok(Some(StoredAccount {
            data,
            lamports,
            owner: *info.owner,
            executable: info.executable,
            rent_epoch: info.rent_epoch,
        }))
    }

    async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> anyhow::Result<Vec<Option<StoredAccount>>> {
        let infos = pubkeys
            .iter()
            .map(|address| {
                let info = self.get_account_info(address);
                if info.data_is_empty() {
                    None
                } else {
                    let lamports = **info.lamports.borrow();
                    let data = info.data.borrow().to_vec();

                    Some(StoredAccount {
                        data,
                        lamports,
                        owner: *info.owner,
                        executable: info.executable,
                        rent_epoch: info.rent_epoch,
                    })
                }
            })
            .collect();

        Ok(infos)
    }

    async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        size: Option<usize>,
    ) -> anyhow::Result<Vec<(Pubkey, StoredAccount)>> {
        Ok(self.get_program_accounts(program_id, size))
    }

    async fn get_latest_blockhash(&self) -> anyhow::Result<Hash> {
        Ok(Hash::new_unique())
    }

    async fn get_minimum_balance_for_rent_exemption(&self, length: usize) -> anyhow::Result<u64> {
        Ok(self.minimum_rent_balance(length))
    }

    async fn send_transaction(&self, transaction: &Transaction) -> anyhow::Result<Signature> {
        Ok(self.execute_transaction(transaction).map_err(|(_, e)| e)?)
    }

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> anyhow::Result<Vec<Option<TransactionStatus>>> {
        Ok(signatures
            .iter()
            .map(|s| self.get_signature_status(s))
            .collect())
    }

    async fn confirm_transactions(&self, signatures: &[Signature]) -> anyhow::Result<Vec<bool>> {
        Ok(signatures
            .iter()
            .map(|s| self.confirm_transaction(s).unwrap())
            .collect())
    }

    fn payer(&self) -> &Keypair {
        use std::mem::MaybeUninit;
        use std::sync::Once;

        unsafe {
            static mut DEFAULT_PAYER: MaybeUninit<Keypair> = MaybeUninit::uninit();
            static DEFAULT_PAYER_INIT: Once = Once::new();

            DEFAULT_PAYER_INIT.call_once(|| {
                let keypair = Keypair::new();

                self.create_account(
                    keypair.pubkey(),
                    system_program::ID,
                    vec![],
                    1_000_000_000 * LAMPORTS_PER_SOL,
                );

                DEFAULT_PAYER = MaybeUninit::new(keypair);
            });

            DEFAULT_PAYER.assume_init_ref()
        }
    }

    fn get_clock(&self) -> Option<Clock> {
        Some(self.get_clock())
    }

    fn set_clock(&self, new_clock: Clock) {
        self.set_clock(new_clock)
    }
}
