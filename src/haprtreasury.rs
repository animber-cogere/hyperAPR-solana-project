use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::rent::Rent,
    sysvar::Sysvar,
};
use spl_token::instruction::initialize_mint;

pub const SEED: &[u8] = b"mintthissuperhyperAPRtoken"; // Seed for PDA
pub const TREASURY_AUTHORITY_SEED: &[u8] = b"treasurythissuperhyperAPRtoken";
const MINT_AUTHORITY_SEED: &[u8] = b"mint-authority-seed"; // Unique seed for mint authority

const TREASURY_ACCOUNT_SIZE: usize = 41; // Initialization flag (1) + Admin Pubkey (32) + Balance (8)
const MINT_ACCOUNT_SIZE: usize = 82; // Fixed size for SPL Token Mint

pub fn initialize_treasury(
    accounts: &[AccountInfo],
    admin: &Pubkey,
    program_id: &Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let treasury_account = next_account_info(accounts_iter)?;
    let admin_account = next_account_info(accounts_iter)?;
    let mint_account_info = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let sysvar_rent = next_account_info(accounts_iter)?;
    //let treasury_token_account = next_account_info(accounts_iter)?; // Treasury Token Account

    // Check if the treasury is already initialized
    if treasury_account.try_borrow_data()?.len() >= TREASURY_ACCOUNT_SIZE {
        let treasury_data = treasury_account.try_borrow_data()?;
        if treasury_data[0] == 1 {
            msg!("Treasury is already initialized.");
            return Err(ProgramError::AccountAlreadyInitialized);
        }
    }

    // Create treasury account if it has insufficient space
    if treasury_account.data_len() < TREASURY_ACCOUNT_SIZE {
        let rent = Rent::from_account_info(sysvar_rent)?;
        let lamports_required = rent.minimum_balance(TREASURY_ACCOUNT_SIZE);

        let (pda, bump_seed) = Pubkey::find_program_address(&[TREASURY_AUTHORITY_SEED], program_id);
        if *treasury_account.key != pda {
            msg!("Error: Treasury PDA does not match derived PDA.");
            return Err(ProgramError::InvalidArgument);
        }

        let seeds = &[TREASURY_AUTHORITY_SEED, &[bump_seed]];

        invoke_signed(
            &system_instruction::create_account(
                admin,
                treasury_account.key,
                lamports_required,
                TREASURY_ACCOUNT_SIZE as u64,
                program_id,
            ),
            &[
                admin_account.clone(),
                treasury_account.clone(),
                system_program.clone(),
                sysvar_rent.clone(),
            ],
            &[seeds],
        )?;
        msg!("Treasury account created with correct size.");
    }

    // Initialize treasury data structure
    let mut treasury_data = treasury_account.try_borrow_mut_data()?;
    treasury_data[0] = 1; // Mark as initialized
    treasury_data[1..33].copy_from_slice(admin.as_ref()); // Store admin Pubkey
    treasury_data[33..41].copy_from_slice(&0u64.to_le_bytes()); // Initialize balance to zero

    msg!("Treasury account initialized with admin and balance.");

    // Derive Mint PDA
    let (mint_pda, mint_bump_seed) = Pubkey::find_program_address(&[SEED], program_id);

    // Check mint account data length; if uninitialized, create and initialize it
    if mint_account_info.try_borrow_data()?.len() < MINT_ACCOUNT_SIZE {
        let rent = Rent::from_account_info(sysvar_rent)?;
        let mint_size = MINT_ACCOUNT_SIZE;

        invoke_signed(
            &system_instruction::create_account(
                admin,
                &mint_pda,
                rent.minimum_balance(mint_size),
                mint_size as u64,
                &spl_token::id(),
            ),
            &[
                admin_account.clone(),
                mint_account_info.clone(),
                system_program.clone(),
                sysvar_rent.clone(),
            ],
            &[&[SEED, &[mint_bump_seed]]],
        )?;
        msg!("Mint account created.");

        // Initialize mint account with the token program
        invoke_signed(
            &initialize_mint(
                &spl_token::id(),
                &mint_pda,
                &treasury_account.key, // Treasury PDA as mint authority
                None,                  // No freeze authority
                9,                     // Decimals
            )?,
            &[
                mint_account_info.clone(),
                token_program.clone(),
                sysvar_rent.clone(), // Include rent account for mint initialization
            ],
            &[&[SEED, &[mint_bump_seed]]],
        )?;
        msg!("Mint account initialized.");
    } else {
        msg!("Mint account already initialized.");
    }

    // // Step 3: Initialize the Treasury Token Account
    // msg!("Validating Treasury Token Account...");

    // // Avoid multiple borrows of `treasury_token_account`
    // let treasury_token_account_data_len = treasury_token_account.data_len();
    // if treasury_token_account_data_len == 0 {
    //     msg!("Treasury Token Account not initialized. Proceeding with creation...");

    //     // Invoke to create the associated token account
    //     invoke_signed(
    //         &spl_associated_token_account::create_associated_token_account(
    //             &admin_account.key,     // Payer
    //             &treasury_account.key,  // Treasury PDA (Owner of the Treasury Token Account)
    //             &mint_account_info.key, // Mint Account
    //         ),
    //         &[
    //             admin_account.clone(),
    //             treasury_token_account.clone(),
    //             treasury_account.clone(),
    //             mint_account_info.clone(),
    //             system_program.clone(),
    //             token_program.clone(),
    //             sysvar_rent.clone(),
    //         ],
    //         &[], // No seeds needed
    //     )?;
    //     msg!("Treasury Token Account successfully created.");
    // } else {
    //     msg!("Treasury Token Account already initialized.");
    // }

    Ok(())
}

// Initialize treasury function without Anchor's Context
// pub fn initialize_treasury(
//     accounts: &[AccountInfo],
//     admin: &Pubkey,
//     program_id: &Pubkey,
// ) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();

//     // Treasury account is the first account passed
//     let treasury_account = next_account_info(accounts_iter)?;

//     let (pda, bump_seed) = Pubkey::find_program_address(&[TREASURY_AUTHORITY_SEED], program_id);
//     msg!("Derived Treasury PDA on-chain: {:?}", pda);
//     msg!("Bump Seed for Treasury PDA: {:?}", bump_seed);

//     // Ensure the treasury_account has the required space allocated (e.g., 40 bytes for admin and balance)
//     if treasury_account.data_len() < 40 {
//         msg!("Error: treasury_account does not have sufficient space allocated.");
//         return Err(ProgramError::InvalidAccountData);
//     }

//     Ok(())
// }

// Private deposit function requiring the PDA authority
pub fn deposit_tokens_internal(
    accounts: &[AccountInfo],
    amount: u64,
    treasury_pda: &Pubkey,
    bump: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let treasury_account = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?;

    // Verify the treasury PDA authority
    if treasury_account.owner != treasury_pda {
        msg!("Unauthorized attempt to deposit into treasury.");
        return Err(ProgramError::Custom(0)); // Unauthorized access error
    }

    // Perform the deposit with the PDA signature
    let seeds = &[TREASURY_AUTHORITY_SEED, &[bump]];
    let transfer_instruction =
        system_instruction::transfer(user_account.key, treasury_account.key, amount);
    invoke_signed(
        &transfer_instruction,
        &[user_account.clone(), treasury_account.clone()],
        &[seeds],
    )
}

// Private withdraw function requiring the PDA authority
pub fn withdraw_tokens_internal(
    accounts: &[AccountInfo],
    amount: u64,
    treasury_pda: &Pubkey,
    bump: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let treasury_account = next_account_info(accounts_iter)?;
    let admin_account = next_account_info(accounts_iter)?;

    // Verify that the treasury PDA is the authorized signer
    if treasury_account.owner != treasury_pda {
        msg!("Unauthorized attempt to withdraw from treasury.");
        return Err(ProgramError::Custom(1)); // Unauthorized access error
    }

    // Execute withdrawal with PDA authority
    let seeds = &[TREASURY_AUTHORITY_SEED, &[bump]];
    let transfer_instruction =
        system_instruction::transfer(treasury_account.key, admin_account.key, amount);
    invoke_signed(
        &transfer_instruction,
        &[treasury_account.clone(), admin_account.clone()],
        &[seeds],
    )
}

// Deposit tokens function without Anchor Context
// pub fn deposit_tokens(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();
//     let treasury_account = next_account_info(accounts_iter)?;
//     let user_account = next_account_info(accounts_iter)?;

//     // Update treasury balance
//     let mut treasury_data = treasury_account.try_borrow_mut_data()?;
//     let balance_bytes: [u8; 8] = treasury_data[32..40]
//         .try_into()
//         .expect("slice with incorrect length");

//     // Assuming this is within a function in haprtreasury.rs
//     let mut balance = u64::from_le_bytes(balance_bytes);
//     balance += amount;

//     treasury_data[32..40].copy_from_slice(&balance.to_le_bytes());

//     // Perform the transfer using Solana's invoke method
//     let transfer_instruction = solana_program::system_instruction::transfer(
//         user_account.key,
//         treasury_account.key,
//         amount,
//     );
//     invoke(
//         &transfer_instruction,
//         &[user_account.clone(), treasury_account.clone()],
//     )
// }

// // Withdraw tokens function without Anchor Context
// pub fn withdraw_tokens(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();
//     let treasury_account = next_account_info(accounts_iter)?;
//     let admin_account = next_account_info(accounts_iter)?;

//     // Verify that admin has permission to withdraw
//     let treasury_data = treasury_account.try_borrow_data()?;
//     // Manually handle the conversion error
//     let stored_admin_pubkey =
//         Pubkey::try_from(&treasury_data[0..32]).map_err(|_| ProgramError::InvalidAccountData)?;
//     if *admin_account.key != stored_admin_pubkey {
//         msg!("Unauthorized access to withdraw");
//         return Err(ProgramError::Custom(0)); // Custom error for unauthorized access
//     }

//     // Check balance and update after successful withdrawal
//     let mut balance = u64::from_le_bytes(treasury_data[32..40].try_into().unwrap());
//     if balance < amount {
//         msg!("Insufficient funds in the treasury");
//         return Err(ProgramError::Custom(1)); // Custom error for insufficient funds
//     }

//     balance -= amount;
//     let mut treasury_data = treasury_account.try_borrow_mut_data()?; // This gives mutable access directly
//     treasury_data[32..40].copy_from_slice(&balance.to_le_bytes());

//     // Transfer funds back to the admin
//     let transfer_instruction = solana_program::system_instruction::transfer(
//         treasury_account.key,
//         admin_account.key,
//         amount,
//     );
//     invoke_signed(
//         &transfer_instruction,
//         &[treasury_account.clone(), admin_account.clone()],
//         &[],
//     )
// }