// use crate::haprtreasury;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::rent::Rent,
    sysvar::Sysvar,
};

use borsh::{BorshDeserialize, BorshSerialize};
use spl_token::instruction::{burn, initialize_account, mint_to, transfer};

use solana_program::program_pack::Pack; // Add this import for unpack_from_slice
                                        //use spl_token::state::Account;
use spl_token::state::Mint;
use std::io::Cursor;

const SEED: &[u8] = b"mintthissuperhyperAPRtoken"; // Seed for PDA
                                                   //const TICKET_SEED: &[u8] = b"ticketthissuperhyperAPRtoken"; //The treasury seed
const TREASURY_SEED: &[u8] = b"treasurythissuperhyperAPRtoken";
const TICKET_SEED: &[u8] = b"ticket_seed";
const MINT_AUTHORITY_SEED: &[u8] = b"mint-authority-seed"; // Unique seed for mint authority

const YIELD_INTEREST_RATE: u64 = 5; // 5% interest rate
const MAX_SUPPLY: u64 = 1_000_000_000; // 1 billion max supply
const FIXED_TICKET_PRICE: u64 = 1_000_000;

// Define a seed and bump for the PDA (could be any seed)
//const TREASURY_AUTHORITY_SEED: &[u8] = SEED; //b"treasury_authority";

// Helper function to get the treasury PDA
// fn get_treasury_pda(program_id: &Pubkey) -> (Pubkey, u8) {
//     Pubkey::find_program_address(&[TREASURY_AUTHORITY_SEED], program_id)
// }

// //Public function for depositing tokens
// pub fn token_deposit(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
//     let (treasury_pda, bump) = get_treasury_pda(program_id);

//     // Pass the PDA and bump seed to the treasury's deposit function
//     haprtreasury::deposit_tokens_internal(accounts, amount, &treasury_pda, bump)
// }

// // Public function for withdrawing tokens
// pub fn token_withdraw(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
//     let (treasury_pda, bump) = get_treasury_pda(program_id);

//     // Pass the PDA and bump seed to the treasury's withdraw function
//     haprtreasury::withdraw_tokens_internal(accounts, amount, &treasury_pda, bump)
// }

// Helper function to calculate staking rewards
fn calculate_staking_rewards(amount_staked: u64, last_staked_time: i64, current_time: i64) -> u64 {
    let staking_duration_seconds = current_time - last_staked_time;
    let annual_reward_rate = 0.05;
    let seconds_in_a_year = 365 * 24 * 60 * 60;
    ((amount_staked as f64)
        * (annual_reward_rate * (staking_duration_seconds as f64) / (seconds_in_a_year as f64)))
        .round() as u64
}

pub fn mint_tokens(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer = next_account_info(accounts_iter)?;
    msg!("Payer: {:?}", payer.key);

    let mint_account = next_account_info(accounts_iter)?;
    msg!("Mint Account: {:?}", mint_account.key);

    let to_account = next_account_info(accounts_iter)?;
    msg!("Recipient Token Account: {:?}", to_account.key);

    let mint_authority = next_account_info(accounts_iter)?;
    msg!("Mint Authority (Treasury PDA): {:?}", mint_authority.key);

    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // let token_account_data = to_account.try_borrow_data()?;
    // let token_account_info = Account::unpack(&token_account_data)?;
    // msg!("Token Account Mint: {:?}", token_account_info.mint);
    // msg!("Token Account Owner: {:?}", token_account_info.owner);

    // Replace SEED with TREASURY_SEED for mint authority PDA check
    let (treasury_pda, bump_seed) = Pubkey::find_program_address(&[TREASURY_SEED], program_id);

    if *mint_authority.key != treasury_pda {
        msg!("Error: Mint authority PDA does not match derived PDA.");
        return Err(ProgramError::InvalidArgument);
    }
    msg!("Mint authority (PDA) verified successfully.");

    // Step 1: Check Mint Account Initialization
    {
        let mint_data = mint_account.try_borrow_data()?; // Immutable borrow
        if mint_account.owner != &spl_token::id() {
            msg!("Error: Mint Account is not owned by the SPL Token Program.");
            return Err(ProgramError::InvalidAccountData);
        }
        if Mint::unpack_from_slice(&mint_data).is_err() {
            msg!("Error: Mint Account data is invalid or uninitialized.");
            return Err(ProgramError::InvalidAccountData);
        }
        msg!("Mint Account is initialized and valid.");
    } // Release mint_account borrow here

    // Step 2: Initialize Recipient Token Account
    if to_account.owner != &spl_token::id() || to_account.data_len() == 0 {
        msg!("Recipient Token Account not initialized. Initializing...");
        let rent = Rent::get()?;
        let account_size = 165; // SPL Token Account size

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                to_account.key,
                rent.minimum_balance(account_size),
                account_size as u64,
                &spl_token::id(),
            ),
            &[payer.clone(), to_account.clone(), system_program.clone()],
            &[&[TREASURY_SEED, &[bump_seed]]],
        )?;
        msg!("Recipient token account created.");

        invoke_signed(
            &initialize_account(
                &spl_token::id(),
                to_account.key,
                mint_account.key,
                payer.key,
            )?,
            &[
                to_account.clone(),
                mint_account.clone(),
                payer.clone(),
                token_program.clone(),
            ],
            &[&[TREASURY_SEED, &[bump_seed]]],
        )?;
        msg!("Recipient token account initialized.");
    } else {
        msg!("Recipient Token Account is already initialized.");
    }

    // Step 3: Check Max Supply Constraint
    {
        let mint_data = mint_account.try_borrow_data()?; // Immutable borrow
        let current_supply = u64::from_le_bytes(mint_data[36..44].try_into().unwrap());

        if current_supply + amount > MAX_SUPPLY {
            msg!("Error: Minting would exceed max supply.");
            return Err(ProgramError::Custom(1));
        }
        msg!("Current supply: {}, Minting: {}", current_supply, amount);
    } // Release mint_account borrow here

    // Step 4: Mint Tokens
    let mint_instruction = mint_to(
        &spl_token::id(),
        mint_account.key,
        to_account.key,
        mint_authority.key,
        &[],
        amount,
    )?;

    invoke_signed(
        &mint_instruction,
        &[
            mint_account.clone(),
            to_account.clone(),
            mint_authority.clone(),
            token_program.clone(),
        ],
        &[&[TREASURY_SEED, &[bump_seed]]],
    )?;

    msg!("Mint operation completed successfully.");
    Ok(())
}

pub fn test_derived_pda(program_id: &Pubkey) -> ProgramResult {
    let (pda, bump_seed) = Pubkey::find_program_address(&[SEED], program_id);
    msg!("Expected PDA on-chain: {:?}", pda);
    msg!("Bump Seed on-chain: {:?}", bump_seed);
    Ok(())
}

pub fn burn_tokens(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
    // Accounts
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?; // Payer for required account creation
    let burn_account = next_account_info(accounts_iter)?; // Token account to burn from
    let mint_account = next_account_info(accounts_iter)?; // Mint account
    let burn_authority = next_account_info(accounts_iter)?; // PDA for authority
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let sysvar_rent = next_account_info(accounts_iter)?;

    msg!("Payer: {:?}", payer.key);
    msg!("Burn Account: {:?}", burn_account.key);
    msg!("Mint Account: {:?}", mint_account.key);
    msg!("Burn Authority (PDA): {:?}", burn_authority.key);

    // Verify the mint account is owned by the SPL Token Program
    if mint_account.owner != &spl_token::id() {
        msg!("Error: Mint account is not owned by the SPL Token Program.");
        return Err(ProgramError::IncorrectProgramId);
    }
    msg!("Mint account ownership verified.");

    // Verify the burn account is owned by the SPL Token Program
    if burn_account.owner != &spl_token::id() {
        msg!("Error: Burn account is not owned by the SPL Token Program.");
        return Err(ProgramError::IncorrectProgramId);
    }
    msg!("Burn account ownership verified.");

    // Verify the PDA for burn authority
    let (expected_burn_pda, _bump_seed) =
        Pubkey::find_program_address(&[TREASURY_SEED], program_id);
    if *burn_authority.key != expected_burn_pda {
        msg!("Error: Invalid burn authority PDA");
        return Err(ProgramError::IncorrectProgramId);
    }
    msg!("Burn authority PDA verified successfully.");

    // Execute the burn instruction
    let burn_instruction = burn(
        &spl_token::id(),
        &burn_account.key, // User's associated token account
        &mint_account.key, // Mint account
        &payer.key,        // User wallet (burn_account owner) must sign
        &[],               // Additional signers
        amount,            // Amount to burn
    )?;

    invoke(
        &burn_instruction,
        &[
            burn_account.clone(),
            mint_account.clone(),
            payer.clone(),         // Payer signs as the burn account owner
            token_program.clone(), // SPL Token Program
        ],
    )?;
    msg!("Burn operation completed successfully.");

    Ok(())
}

// Transfer Tokens
pub fn transfer_tokens(
    accounts: &[AccountInfo],
    amount: u64,
    _program_id: &Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let sender_account = next_account_info(accounts_iter)?;
    let recipient_account = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?;
    let sender_owner = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Transfer tokens using SPL Token Program's transfer instruction
    let transfer_instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        sender_account.key,
        recipient_account.key,
        sender_owner.key,
        &[],
        amount,
    )?;

    msg!(
        "Transferring {} tokens from {:?} to {:?}",
        amount,
        sender_account.key,
        recipient_account.key
    );

    invoke(
        &transfer_instruction,
        &[
            sender_account.clone(),
            recipient_account.clone(),
            sender_owner.clone(),
            token_program.clone(),
        ],
    )?;

    msg!("Transfer operation completed successfully.");
    Ok(())
}

// Define the Staker struct to store staking information
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Staker {
    pub amount_staked: u64,
    pub last_staked_time: i64,
    pub staking_duration: i64,
    pub owner_pubkey: Pubkey,
}

pub fn stake_tokens(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let user_token_account = next_account_info(accounts_iter)?;
    let treasury_token_account = next_account_info(accounts_iter)?;
    let treasury_pda = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    const DEFAULT_DURATION: i64 = 24 * 60 * 60; // One day in seconds
    const STAKER_ACCOUNT_SIZE: usize = 8 + 8 + 8 + 32; // Total: 56 bytes

    let staker_seed = &[user.key.as_ref(), b"staker"];
    let (staker_pda, bump_seed) = Pubkey::find_program_address(staker_seed, program_id);
    let staker_account = next_account_info(accounts_iter)?;

    if *staker_account.key != staker_pda {
        msg!("Error: Staker account does not match derived PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    if staker_account.data_len() == 0 {
        let rent = Rent::get()?;
        let lamports_required = rent.minimum_balance(STAKER_ACCOUNT_SIZE);

        invoke_signed(
            &system_instruction::create_account(
                user.key,
                staker_account.key,
                lamports_required,
                STAKER_ACCOUNT_SIZE as u64,
                program_id,
            ),
            &[user.clone(), staker_account.clone(), system_program.clone()],
            &[&[user.key.as_ref(), b"staker", &[bump_seed]]],
        )?;
        msg!("Staker account created.");
    } else {
        msg!(
            "Staker account size: {}, Expected: {}",
            staker_account.data_len(),
            STAKER_ACCOUNT_SIZE
        );
    }

    let mut staker_data = if staker_account.data_len() == 0 {
        Staker {
            amount_staked: 0,
            last_staked_time: 0,
            staking_duration: DEFAULT_DURATION,
            owner_pubkey: *user.key,
        }
    } else {
        Staker::try_from_slice(&staker_account.try_borrow_data()?)?
    };

    msg!("Current staker data: {:?}", staker_data);

    let transfer_instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        user_token_account.key,
        treasury_token_account.key,
        user.key,
        &[],
        amount,
    )?;
    invoke(
        &transfer_instruction,
        &[
            user_token_account.clone(),
            treasury_token_account.clone(),
            user.clone(),
            token_program.clone(),
        ],
    )?;
    msg!("Staked {} tokens successfully.", amount);

    staker_data.amount_staked += amount;
    staker_data.last_staked_time = Clock::get()?.unix_timestamp;
    staker_data.staking_duration = DEFAULT_DURATION;
    staker_data.owner_pubkey = *user.key;

    staker_data.serialize(&mut Cursor::new(
        &mut staker_account.try_borrow_mut_data()?[..],
    ))?;
    msg!("Updated staker data: {:?}", staker_data);

    Ok(())
}

// Unstake Tokens
pub fn unstake_tokens(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Retrieve Accounts
    let user = next_account_info(accounts_iter)?; // User attempting to unstake
    let staker_account = next_account_info(accounts_iter)?; // Staker metadata account
    let user_token_account = next_account_info(accounts_iter)?; // User's token account
    let treasury_token_account = next_account_info(accounts_iter)?; // Treasury's token account
    let authority_account = next_account_info(accounts_iter)?; // Treasury PDA authority
    let token_program = next_account_info(accounts_iter)?; // SPL Token program

    // Verify PDA Authority
    let (treasury_pda, bump_seed) = Pubkey::find_program_address(&[TREASURY_SEED], program_id);
    if *authority_account.key != treasury_pda {
        msg!("Error: Invalid Treasury PDA authority");
        return Err(ProgramError::IncorrectProgramId);
    }
    msg!("Treasury PDA authority verified.");

    // Ensure token accounts are owned by the SPL Token program
    if user_token_account.owner != &spl_token::id() {
        msg!("Error: User Token Account is not owned by SPL Token Program.");
        return Err(ProgramError::IncorrectProgramId);
    }
    if treasury_token_account.owner != &spl_token::id() {
        msg!("Error: Treasury Token Account is not owned by SPL Token Program.");
        return Err(ProgramError::IncorrectProgramId);
    }
    msg!("Token account ownership verified.");

    // Deserialize Staker Metadata
    let mut staker_data = Staker::try_from_slice(&staker_account.data.borrow())?;

    // Validate Ownership
    if staker_data.owner_pubkey != *user.key {
        msg!(
            "Error: Unstake attempted by unauthorized user. Expected: {}, Found: {}",
            staker_data.owner_pubkey,
            user.key
        );
        return Err(ProgramError::IllegalOwner);
    }

    msg!("Ownership validation successful.");
    msg!(
        "Staker Metadata: Staker: {}, Amount Staked: {}, Last Staked Time: {}, Staking Duration: {}",
        staker_data.owner_pubkey,
        staker_data.amount_staked,
        staker_data.last_staked_time,
        staker_data.staking_duration,
    );

    // Fetch current time`
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    // Check staking duration
    if current_time < staker_data.last_staked_time + staker_data.staking_duration {
        msg!(
            "Error: Staking period not complete. Current time: {}, Required time: {}",
            current_time,
            staker_data.last_staked_time + staker_data.staking_duration
        );
        return Err(ProgramError::Custom(
            CustomError::StakingPeriodNotComplete as u32,
        ));
    }

    let unstake_amount = if amount >= staker_data.amount_staked {
        staker_data.amount_staked
    } else {
        amount
    };

    // Calculate Rewards
    let reward_amount =
        calculate_staking_rewards(unstake_amount, staker_data.last_staked_time, current_time);
    let total_amount = unstake_amount + reward_amount;

    msg!(
        "Rewards calculated: Amount Staked: {}, Reward: {}, Total: {}",
        unstake_amount,
        reward_amount,
        total_amount
    );

    // Transfer tokens from Treasury to User
    let seeds = &[TREASURY_SEED, &[bump_seed]];
    let transfer_instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        &treasury_token_account.key,
        &user_token_account.key,
        &authority_account.key,
        &[], // No additional signers
        total_amount,
    )?;
    invoke_signed(
        &transfer_instruction,
        &[
            treasury_token_account.clone(),
            user_token_account.clone(),
            authority_account.clone(),
            token_program.clone(),
        ],
        &[seeds],
    )?;
    msg!("Tokens transferred: {}.", total_amount);

    // Reset or adjust staker data based on the unstaked amount
    if amount >= staker_data.amount_staked {
        // Fully unstake
        staker_data.amount_staked = 0;
        staker_data.last_staked_time = 0;
        staker_data.staking_duration = 0;
        msg!("Full unstake completed. Staker data reset.");
    } else {
        // Partial unstake
        staker_data.amount_staked -= amount;
        //staker_data.last_staked_time = Clock::get()?.unix_timestamp; // Update staking timestamp
        msg!(
            "Partial unstake completed. Remaining staked amount: {}",
            staker_data.amount_staked
        );
    }

    // Serialize the updated staker data back into the account
    staker_data.serialize(&mut Cursor::new(
        &mut staker_account.try_borrow_mut_data()?[..],
    ))?;

    msg!("Staker data updated successfully.");

    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Ticket {
    // pub owner: Pubkey,
    //pub deposit_amount: u64,
    pub number_of_tickets: u64,
    pub deposit_time: i64,
    pub vesting_period: i64,
    pub claimed: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TicketAccount {
    pub owner: Pubkey,        // Owner of this ticket account
    pub tickets: Vec<Ticket>, // List of tickets
    pub ticket_total: u64,
}

//Purchase a ticket for a certain amount via PDA
pub fn purchase_tickets(
    accounts: &[AccountInfo],
    amount: u64,
    vesting_period: i64,
    program_id: &Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let buyer_account = next_account_info(accounts_iter)?; // Buyer
    let buyer_token_account = next_account_info(accounts_iter)?; // Buyer's token account
    let treasury_token_account = next_account_info(accounts_iter)?; // Treasury's token account
                                                                    //let authority_account = next_account_info(accounts_iter)?; // Treasury PDA
    let ticket_account = next_account_info(accounts_iter)?; // Ticket account
                                                            // let mint_account: &AccountInfo = next_account_info(accounts_iter); //Mint account (to get current token supply)
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Validate Treasury PDA
    let (treasury_pda, treasury_bump_seed) =
        Pubkey::find_program_address(&[TREASURY_SEED], program_id);
    // if *authority_account.key != treasury_pda {
    //     msg!("Error: Invalid PDA authority for treasury");
    //     return Err(ProgramError::IncorrectProgramId);
    // }
    // Derive the Ticket PDA
    let (ticket_pda, ticket_bump_seed) =
        Pubkey::find_program_address(&[TICKET_SEED, buyer_account.key.as_ref()], program_id);
    if *ticket_account.key != ticket_pda {
        msg!("Error: Invalid Ticket PDA");
        return Err(ProgramError::InvalidAccountData);
    }
    msg!("Ticket PDA verified successfully.");

    // Calculate the total cost for tickets
    let number_of_tickets = amount / FIXED_TICKET_PRICE;
    let total_cost = FIXED_TICKET_PRICE * number_of_tickets;

    if number_of_tickets == 0 {
        msg!(
            "Error: Amount {} is insufficient to purchase even one ticket.",
            amount
        );
        return Err(ProgramError::Custom(0x01)); // Custom error for insufficient funds
    }

    msg!(
        "Purchasing {} tickets for {} tokens (Price per ticket: {}).",
        number_of_tickets,
        amount,
        FIXED_TICKET_PRICE
    );

    msg!(
        "Total cost for {} tickets: {}",
        number_of_tickets,
        total_cost
    );

    // Perform the token transfer from buyer to treasury
    let seeds = &[TREASURY_SEED, &[treasury_bump_seed]];
    let transfer_instruction = transfer(
        &spl_token::id(),
        &buyer_token_account.key,
        &treasury_token_account.key,
        &buyer_account.key,
        &[],
        total_cost,
    )?;
    invoke_signed(
        &transfer_instruction,
        &[
            buyer_token_account.clone(),    
            treasury_token_account.clone(),
            buyer_account.clone(),
            token_program.clone(),
        ],
        &[&seeds[..]],
    )?;
    msg!("Token transfer successful.");

    // Handle TicketAccount initialization or update
    let mut ticket_account_data: TicketAccount = if ticket_account.data_len() == 0 {
        msg!("Initializing new TicketAccount Struct...");
        TicketAccount {
            owner: *buyer_account.key,
            tickets: Vec::new(),
            ticket_total: 0,
        }
    } else {
        msg!("Loading existing TicketAccount Struct...");
        TicketAccount::try_from_slice(&ticket_account.data.borrow())?
    };
    msg!("Ticket Account Struct intialized and loaded!");
    // Ensure the owner matches the buyer
    if ticket_account_data.owner != *buyer_account.key {
        msg!("Error: Ticket account owner mismatch.");
        return Err(ProgramError::InvalidAccountData);
    }

    // Add new tickets to the account

    // Serialize ticket data for this user
    let ticket = Ticket {
        // owner: *buyer_account.key,
        number_of_tickets,
        deposit_time: Clock::get()?.unix_timestamp,
        vesting_period,
        claimed: false,
    };

    ticket_account_data.tickets.push(ticket);
    ticket_account_data.ticket_total += number_of_tickets;

    // Serialize and save updated TicketAccount
    //let current_supply = u64::from_le_bytes(mint_data[36..44].try_into().unwrap());
    // Define the maximum number of tickets that can be stored
    let max_tickets = 178; //(MAX_SUPPLY / FIXED_TICKET_PRICE) as usize;
    if ticket_account_data.tickets.len() + 1 > max_tickets {
        msg!("Error: Exceeding maximum ticket limit.");
        return Err(ProgramError::Custom(0x02)); // Custom error code for max ticket limit
    }
    // Calculate the size required for the TicketAccount
    let ticket_size = 8 + 8 + 8 + 1; // Size of each ticket
    let required_size = 32                    // Owner (Pubkey)
    + 8                                   // ticket_total (u64)
    + 4                                   // Vec metadata (length)
    + (max_tickets * ticket_size); // Maximum tickets

    if ticket_account.data_len() == 0 {
        msg!("There isn't an onchain Ticket Account matching the struct, creating one now!");
        let rent = Rent::get()?;
        let lamports_required = rent.minimum_balance(required_size);
        invoke_signed(
            &system_instruction::create_account(
                buyer_account.key,    // Payer
                ticket_account.key,   //    Ticket PDA
                lamports_required,    // Rent-exempt balance
                required_size as u64, // Account size
                program_id,           // Owner program
            ),
            &[
                buyer_account.clone(),
                ticket_account.clone(),
                system_program.clone(), // Add the System Program account here
            ],
            &[&[TICKET_SEED, buyer_account.key.as_ref(), &[ticket_bump_seed]]],
        )?;
        msg!("Ticket account intialized successfully.");
    } else {
        msg!("Ticket account already initialized.");
    }

    ticket_account_data.serialize(&mut &mut ticket_account.data.borrow_mut()[..])?;

    // let account_size = ticket_account.data_len();
    // msg!("Ticket Account Size: {}", account_size);

    msg!("Tickets added: {:?}", ticket_account_data.tickets);

    Ok(())
}

//Reedem purchased ticket for yield via PDA
pub fn redeem_tickets(accounts: &[AccountInfo], amount: u64, program_id: &Pubkey) -> ProgramResult {
    msg!("Redeem tickets function called.");

    let accounts_iter = &mut accounts.iter();
    let ticket_account = next_account_info(accounts_iter)?; // Ticket account
    let owner_account = next_account_info(accounts_iter)?; // User's main account
    let mint_account = next_account_info(accounts_iter)?; // Token mint account
    let owner_token_account = next_account_info(accounts_iter)?; // User's token account
    let treasury_pda = next_account_info(accounts_iter)?; // Treasury PDA
    let token_program = next_account_info(accounts_iter)?; // Token program
    msg!("Checkpoint: Accounts Loaded!");
    // // Verify PDA authority
    //
    // if *authority_account.key != pda {
    //     msg!("Error: Invalid PDA authority");
    //     return Err(ProgramError::IncorrectProgramId);
    // }
    // msg!("PDA authority verified successfully.");

    // Load the ticket account data
    let mut ticket_account_data =
        match TicketAccount::try_from_slice_unchecked(&ticket_account.data.borrow()).unwrap() {
            Ok(data) => data,
            Err(e) => {
                msg!("Error deserializing TicketAccount: {:?}", e);
                return Err(ProgramError::InvalidAccountData);
            }
        };

    msg!("Checkpoint: Ticket Account Data Loaded");
    if ticket_account.data_len() == 0 {
        msg!("Error: TicketAccount data is uninitialized.");
        return Err(ProgramError::UninitializedAccount);
    }

    // Verify ticket account ownership
    if ticket_account_data.owner != *owner_account.key {
        msg!("Unauthorized: Only the owner can redeem tickets.");
        return Err(ProgramError::Custom(CustomError::UnauthorizedAccess as u32));
    }
    msg!("Checkpoint: Ticket Owner verfied");
    // Sanity check: Ensure sufficient tickets are available
    if ticket_account_data.ticket_total < amount {
        msg!(
            "Error: Requested {} tickets but only {} tickets are available.",
            amount,
            ticket_account_data.ticket_total
        );
        return Err(ProgramError::Custom(
            CustomError::InsufficientTickets as u32,
        ));
    }
    msg!("Checkpoint: You have enough tickets to redeem!");

    // Use Clock to get the current time
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    let mut remaining_amount = amount;
    let mut total_yield = 0;
    let mut index = 0;

    // Process tickets in the array
    while remaining_amount > 0 && index < ticket_account_data.tickets.len() {
        let ticket = &mut ticket_account_data.tickets[index];

        // Check if the vesting period has elapsed
        if current_time < ticket.deposit_time + ticket.vesting_period {
            msg!(
                "Ticket at index {} is still vesting. Deposit time: {}, Vesting period: {}.",
                index,
                ticket.deposit_time,
                ticket.vesting_period
            );
            index += 1;
            continue;
        }

        let redeemable_tickets = std::cmp::min(ticket.number_of_tickets, remaining_amount);

        // Calculate yield for this batch of tickets
        let yield_amount = (redeemable_tickets as u64
            * FIXED_TICKET_PRICE
            * YIELD_INTEREST_RATE
            * (current_time as u64 - ticket.deposit_time as u64)
            / (365 * 24 * 60 * 60))
            / 100;
        total_yield += redeemable_tickets * FIXED_TICKET_PRICE + yield_amount;

        // Update ticket state
        ticket.number_of_tickets -= redeemable_tickets;
        remaining_amount -= redeemable_tickets;

        if ticket.number_of_tickets == 0 {
            // Remove the ticket if fully redeemed
            ticket_account_data.tickets.remove(index);
        } else {
            index += 1;
        }
    }

    if remaining_amount > 0 {
        msg!(
            "Error: Insufficient vested tickets to redeem {} tickets.",
            amount
        );
        return Err(ProgramError::Custom(
            CustomError::InsufficientVestedTickets as u32,
        ));
    }

    // Update the total ticket count in the account
    ticket_account_data.ticket_total -= amount;
    let (treasury_pda, bump_seed) = Pubkey::find_program_address(&[TREASURY_SEED], program_id);

    // Mint the total yield to the owner's token account
    let mint_instruction = mint_to(
        &spl_token::id(),
        &mint_account.key,
        &owner_token_account.key,
        &treasury_pda,
        &[],
        total_yield,
    )?;
    let seeds = &[TREASURY_SEED, &[bump_seed]];
    invoke_signed(
        &mint_instruction,
        &[mint_account.clone(), owner_token_account.clone()],
        &[&seeds[..]],
    )?;
    msg!(
        "Yield of {} tokens minted to user's account successfully.",
        total_yield
    );

    // Serialize the updated ticket account data
    ticket_account_data.serialize(&mut &mut ticket_account.data.borrow_mut()[..])?;
    msg!(
        "Tickets redeemed successfully. Remaining tickets: {}",
        ticket_account_data.ticket_total
    );

    Ok(())
}

// Custom Errors for various states
#[derive(Debug)]
pub enum CustomError {
    AlreadyStaking,
    InsufficientFunds,
    VestingNotComplete,
    TicketAlreadyClaimed,
    StakingPeriodNotComplete,
    UnauthorizedAccess,
    InsufficientTickets,
    InsufficientVestedTickets,
}

impl From<CustomError> for ProgramError {
    fn from(e: CustomError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
