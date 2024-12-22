// Standard imports for Solana programs
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::rent,
};

// Import the token and treasury modules
pub mod haprtoken;
pub mod haprtreasury;

// Entrypoint macro to specify the program entry function
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Define the seed for the PDA
    const SEED: &[u8] = b"mintthissuperhyperAPRtoken";
    const TREASURY_AUTHORITY_SEED: &[u8] = b"treasurythissuperhyperAPRtoken";
    //let (pda, _bump_seed) = Pubkey::find_program_address(&[SEED], program_id);
    let (treasury_pda, _treasury_bump) =
        Pubkey::find_program_address(&[TREASURY_AUTHORITY_SEED], program_id);

    // Extract amount from instruction data if present (bytes 1-8)
    let amount = if instruction_data.len() > 1 {
        msg!("The amount is greater than one, let's see if it triggers an error");
        u64::from_le_bytes(
            instruction_data[1..9]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        )
    } else {
        0
    };
    // Determine the instruction based on the first byte in `instruction_data`
    match instruction_data[0] {
        // Match case for instruction `0`: Mint Tokens
        0 => {
            let accounts_iter = &mut accounts.iter();

            // Explicitly assign each account to a variable with clear names
            let payer = next_account_info(accounts_iter)?; // Payer for fees
            let mint_account = next_account_info(accounts_iter)?; // Mint account (SPL Token Mint)
            let to_account = next_account_info(accounts_iter)?; // Recipient's token account
            let mint_authority = next_account_info(accounts_iter)?; // PDA with mint authority (treasuryPDA)

            msg!("Initial Payer: {:?}", payer.key);

            msg!("Initial Mint Account: {:?}", mint_account.key);

            msg!("Initial Recipient Token Account: {:?}", to_account.key);

            msg!("Initial Mint Authority (PDA): {:?}", mint_authority.key);

            if *mint_authority.key != treasury_pda {
                msg!("Error: Mint authority PDA does not match derived PDA.");
                return Err(ProgramError::IncorrectProgramId);
            }

            // Added accounts
            let system_program = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;

            // Construct account array with all required accounts
            let mint_tokens_accounts = [
                payer.clone(),
                mint_account.clone(),
                to_account.clone(),
                mint_authority.clone(),
                system_program.clone(),
                token_program.clone(),
            ];

            msg!("No issues so far, initiating the minting");
            // Call `mint_tokens` with the explicitly structured accounts and amount
            haprtoken::mint_tokens(&mint_tokens_accounts, amount, program_id)
        }

        1 => {
            // Burn tokens: similar account retrieval and validation as needed
            // Add appropriate retrieval and call to `burn_tokens`
            // Example:
            let accounts_iter = &mut accounts.iter();
            let payer = next_account_info(accounts_iter)?; // Payer for required account creation
            let burn_account = next_account_info(accounts_iter)?; // Token account to burn from
            let mint_account = next_account_info(accounts_iter)?; // Mint account
            let burn_authority = next_account_info(accounts_iter)?; // PDA for authority
            let system_program = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;
            let sysvar_rent = next_account_info(accounts_iter)?;

            msg!("Burn Account Owner: {:?}", burn_account.owner);
            msg!("Payer: {:?}", payer.key);

            if burn_account.owner != &spl_token::id() {
                msg!("Error: Burn account owner does not match the expected payer.");
                return Err(ProgramError::IllegalOwner);
            }

            // Derive the PDA and verify it matches the burn_authority
            if *burn_authority.key != treasury_pda {
                msg!("Error: Invalid burn authority PDA");
                return Err(ProgramError::IncorrectProgramId);
            }

            if sysvar_rent.key != &rent::id() {
                msg!("Error: Rent sysvar account missing or incorrect");
                return Err(ProgramError::InvalidAccountData);
            }
            msg!("Rent sysvar account verified.");
            haprtoken::burn_tokens(
                &[
                    payer.clone(),
                    burn_account.clone(),
                    mint_account.clone(),
                    burn_authority.clone(),
                    system_program.clone(),
                    token_program.clone(),
                    sysvar_rent.clone(),
                ],
                amount,
                program_id,
            )
        }
        2 => {
            // Test derived PDA: No accounts needed, just pass the program_id
            haprtoken::test_derived_pda(program_id)
        }
        3 => {
            // Retrieve accounts for treasury and mint initialization
            let accounts_iter = &mut accounts.iter();
            let treasury_account = next_account_info(accounts_iter)?;
            let admin_account = next_account_info(accounts_iter)?;
            let mint_account = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;
            let sysvar_rent = next_account_info(accounts_iter)?;
            //let treasury_token_account = next_account_info(accounts_iter)?; // Treasury Token Account

            // Derive PDAs for both Treasury and Mint

            let (mint_pda, _mint_bump) = Pubkey::find_program_address(&[SEED], program_id);
            msg!("Attempting to initialize the treasury now...");

            // Verify that the provided treasury account matches the derived Treasury PDA
            if *treasury_account.key != treasury_pda {
                msg!("Error: Treasury PDA does not match derived PDA.");
                return Err(ProgramError::InvalidArgument);
            }

            // Verify that the provided mint account matches the derived Mint PDA
            if *mint_account.key != mint_pda {
                msg!("Error: Mint PDA does not match derived PDA.");
                return Err(ProgramError::InvalidArgument);
            }

            // Call `initialize_treasury` with the required accounts
            haprtreasury::initialize_treasury(
                &[
                    treasury_account.clone(),
                    admin_account.clone(),
                    mint_account.clone(),
                    system_program.clone(),
                    token_program.clone(),
                    sysvar_rent.clone(),
                    //treasury_token_account.clone(),
                ],
                &admin_account.key,
                program_id,
            )
        }
        9 => {
            // Retrieve accounts for treasury and mint initialization
            let accounts_iter = &mut accounts.iter();
            let treasury_account = next_account_info(accounts_iter)?;
            let admin_account = next_account_info(accounts_iter)?;
            let mint_account = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;
            let sysvar_rent = next_account_info(accounts_iter)?;
            let treasury_token_account = next_account_info(accounts_iter)?; // Treasury Token Account
            let associated_token_account = next_account_info(accounts_iter)?; // associated Token Account
                                                                              // Call `initialize_treasury` with the required accounts

            msg!("Treasury Account: {}", treasury_account.key);
            haprtreasury::create_treasury_ata(
                &[
                    treasury_account.clone(),
                    admin_account.clone(),
                    mint_account.clone(),
                    system_program.clone(),
                    token_program.clone(),
                    sysvar_rent.clone(),
                    treasury_token_account.clone(),
                    associated_token_account.clone(),
                ],
                &admin_account.key,
                program_id,
            )
        }
        4 => {
            // Transfer tokens
            let accounts_iter = &mut accounts.iter();

            let sender_account = next_account_info(accounts_iter)?; // Sender's token account
            let recipient_account = next_account_info(accounts_iter)?; // Recipient's token account
            let mint_account = next_account_info(accounts_iter)?; // Mint account
            let sender_owner = next_account_info(accounts_iter)?; // Owner of the sender's token account
            let system_program = next_account_info(accounts_iter)?;
            let token_program = next_account_info(accounts_iter)?;

            msg!("Sender Token Account: {:?}", sender_account.key);
            msg!("Recipient Token Account: {:?}", recipient_account.key);
            msg!("Mint Account: {:?}", mint_account.key);
            msg!("Sender: {:?}", sender_owner.key);

            // Validate sender's token account ownership
            if sender_account.owner != &spl_token::id() {
                msg!("Error: Sender token account is not owned by the SPL Token Program");
                return Err(ProgramError::IncorrectProgramId);
            }
            msg!("Sender token account ownership verified.");

            // Validate mint account ownership
            if mint_account.owner != &spl_token::id() {
                msg!("Error: Mint account is not owned by the SPL Token Program");
                return Err(ProgramError::IncorrectProgramId);
            }
            msg!("Mint account ownership verified.");

            // Validate recipient's token account ownership
            if recipient_account.owner != &spl_token::id() {
                msg!("Error: Recipient token account is not owned by the SPL Token Program");
                return Err(ProgramError::IncorrectProgramId);
            }
            msg!("Recipient token account ownership verified.");

            haprtoken::transfer_tokens(
                &[
                    sender_account.clone(),
                    recipient_account.clone(),
                    mint_account.clone(),
                    sender_owner.clone(),
                    system_program.clone(),
                    token_program.clone(),
                ],
                amount,
                program_id,
            )
        }
        5 => {
            let accounts_iter = &mut accounts.iter();
            let user_token_account = next_account_info(accounts_iter)?; // User's token account
            let treasury_token_account = next_account_info(accounts_iter)?; // Treasury's token account
            let treasury_pda = next_account_info(accounts_iter)?; // Treasury PDA
            let user = next_account_info(accounts_iter)?; // User
            let token_program = next_account_info(accounts_iter)?; // SPL Token program
            let system_program = next_account_info(accounts_iter)?;
            let staker_account = next_account_info(accounts_iter)?; // Staker's PDA
                                                                    // Log information for debugging
            msg!("Staking: User Token Account: {:?}", user_token_account.key);
            msg!(
                "Staking: Treasury Token Account: {:?}",
                treasury_token_account.key
            );
            msg!("Staking: Treasury PDA: {:?}", treasury_pda.key);
            msg!("Staking: User: {:?}", user.key);

            haprtoken::stake_tokens(
                &[
                    user_token_account.clone(),
                    treasury_token_account.clone(),
                    treasury_pda.clone(),
                    user.clone(),
                    token_program.clone(),
                    system_program.clone(),
                    staker_account.clone(),
                ],
                amount, // Amount passed from instruction data
                program_id,
            )
        }
        6 => {
            let accounts_iter = &mut accounts.iter();

            let user = next_account_info(accounts_iter)?; // User
            let staker_account = next_account_info(accounts_iter)?; // Staker metadata account
            let user_token_account = next_account_info(accounts_iter)?; // User's token account
            let treasury_token_account = next_account_info(accounts_iter)?; // Treasury's token account
            let authority_account = next_account_info(accounts_iter)?; // Treasury PDA
            let token_program = next_account_info(accounts_iter)?; // SPL Token program

            // Log information for debugging
            msg!(
                "Unstaking: Treasury Token Account: {:?}",
                treasury_token_account.key
            );
            msg!(
                "Unstaking: User Token Account: {:?}",
                user_token_account.key
            );
            msg!("Unstaking: Treasury PDA: {:?}", treasury_pda);
            msg!("Unstaking: User: {:?}", user.key);

            haprtoken::unstake_tokens(
                &[
                    user.clone(),
                    staker_account.clone(),
                    user_token_account.clone(),
                    treasury_token_account.clone(),
                    authority_account.clone(),
                    token_program.clone(),
                ],
                amount, // Amount passed from instruction data
                program_id,
            )
        }
        7 => {
            // Retrieve the accounts for ticket purchase
            let accounts_iter = &mut accounts.iter();

            let buyer_account = next_account_info(accounts_iter)?; // Buyer's main account
            let buyer_token_account = next_account_info(accounts_iter)?; // Buyer's token account
            let treasury_token_account = next_account_info(accounts_iter)?; // Treasury's token account
                                                                            // let authority_account = next_account_info(accounts_iter)?; // Treasury PDA
            let ticket_account = next_account_info(accounts_iter)?; // Ticket PDA
            let token_program = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            // Decode the instruction data
            let amount = u64::from_le_bytes(
                instruction_data[1..9]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            );

            let vesting_period = i64::from_le_bytes(
                instruction_data[9..17]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            );

            msg!(
                "Purchasing tickets with amount: {}, vesting period: {} seconds",
                amount,
                vesting_period
            );

            // Call the `purchase_ticket` function
            haprtoken::purchase_tickets(
                &[
                    buyer_account.clone(),
                    buyer_token_account.clone(),
                    treasury_token_account.clone(),
                    //authority_account.clone(),
                    ticket_account.clone(),
                    token_program.clone(),
                    system_program.clone(),
                ],
                amount,
                vesting_period,
                program_id,
            )
        }

        8 => {
            // Redeem tickets
            let accounts_iter = &mut accounts.iter();

            let ticket_account = next_account_info(accounts_iter)?; // Ticket account
            let owner_account = next_account_info(accounts_iter)?; // User's main account
            let mint_account = next_account_info(accounts_iter)?; // Token mint account
            let owner_token_account = next_account_info(accounts_iter)?; // User's token account
            let treasury_pda = next_account_info(accounts_iter)?; // Treasury PDA
            let token_program = next_account_info(accounts_iter)?; // Token program

            // Deserialize instruction data to retrieve the `amount`
            // Decode the instruction data
            let amount = u64::from_le_bytes(
                instruction_data[1..9]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            );

            msg!("Instruction: Redeem Tickets");
            msg!("Amount to redeem: {}", amount);

            // Call the redeem_ticket function
            haprtoken::redeem_tickets(
                &[
                    ticket_account.clone(),
                    owner_account.clone(),
                    mint_account.clone(),
                    owner_token_account.clone(),
                    treasury_pda.clone(),
                    token_program.clone(),
                ],
                amount,
                program_id,
            );
            Ok(())
        }

        _ => Err(ProgramError::InvalidInstructionData),
    }
}

