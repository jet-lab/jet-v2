use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};

#[derive(Accounts)]
pub struct IfNotInitialized<'info> {
    pub program: AccountInfo<'info>,
    pub account_to_check: AccountInfo<'info>,
}

pub fn if_not_initialized_handler(
    ctx: Context<IfNotInitialized>,
    instruction: Vec<u8>,
) -> Result<()> {
    let acc = &ctx.accounts.account_to_check;
    if **acc.lamports.borrow() == 0 && acc.data.borrow().len() == 0 {
        invoke(
            &Instruction {
                program_id: ctx.accounts.program.key(),
                accounts: ctx.remaining_accounts.to_vec().to_account_metas(None),
                data: instruction,
            },
            ctx.remaining_accounts,
        )?;
    }

    Ok(())
}
