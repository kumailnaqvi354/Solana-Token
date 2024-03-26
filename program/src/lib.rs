use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

const TAX_RATE: u64 = 1;

const WHITELIST_ACCOUNT_PUBKEY: [u8; 32] = [0; 32];

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    
    let from_account = next_account_info(accounts_iter)?;

    let to_account = next_account_info(accounts_iter)?;

    let burn_account = next_account_info(accounts_iter)?;

    if !from_account.is_writable || !to_account.is_writable || !burn_account.is_writable {
        return Err(ProgramError::InvalidArgument);
    }

    let from_owner = from_account.owner;

    if from_owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let transfer_amount = from_account.lamports() - from_account.try_data_len()? as u64;

    let is_whitelisted = *from_owner == Pubkey::new(&WHITELIST_ACCOUNT_PUBKEY);

    let tax_amount = if !is_whitelisted {
        (transfer_amount * TAX_RATE) / 100
    } else {
        0 
    };

    let final_amount = transfer_amount - tax_amount;

    let burn_amount = tax_amount / 2;

    let from_data = from_account.try_borrow_data()?;
    let mut to_data = to_account.try_borrow_mut_data()?;
    to_data.copy_from_slice(&from_data);

    from_account
        .try_borrow_mut_lamports()?
        .checked_sub(tax_amount)
        .ok_or(ProgramError::InsufficientFunds)?;

    to_account
        .try_borrow_mut_lamports()?
        .checked_add(final_amount)
        .ok_or(ProgramError::InsufficientFunds)?;

    burn_account
        .try_borrow_mut_lamports()?
        .checked_add(burn_amount)
        .ok_or(ProgramError::InsufficientFunds)?;

    Ok(())
}
