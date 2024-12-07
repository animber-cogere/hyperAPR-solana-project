// Use pg.connection and pg.wallet instead of manual imports

import * as splToken from "@solana/spl-token";

import BN from "bn.js";

// Constants for program ID and PDA seed
const programId = new web3.PublicKey(
  "ABHENVYtMXfAdN741mJzwoLtqGW7ntpT9uhr2f1Q7wB1"
);

// Define the PDA seed
const seed = "mintthissuperhyperAPRtoken";
const treasury_seed = "treasurythissuperhyperAPRtoken";
//const ticket_seed = "ticketthissuperhyperAPRtoken";

// Function to get the Treasury PDA
async function getTreasuryPDA() {
  return web3.PublicKey.findProgramAddressSync(
    [Buffer.from(treasury_seed)],
    programId
  );
}

// Function to get Ticket PDA based on user public key
async function getTicketPDA(userPublicKey: web3.PublicKey) {
  return web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ticket_seed"), userPublicKey.toBuffer()],
    programId
  );
}
//MintPDA
async function getMintAccount() {
  return web3.PublicKey.findProgramAddressSync(
    [Buffer.from(seed)], // Use the same seed as during initialization
    programId
  );
}

// Verify that the mint account is owned by the SPL Token Program
async function checkMintAccountOwnership(mintAddress: web3.PublicKey) {
  const accountInfo = await pg.connection.getAccountInfo(mintAddress);
  if (!accountInfo) {
    throw new Error("Mint account does not exist");
  }

  // Check if the account is owned by the SPL Token Program
  if (!accountInfo.owner.equals(splToken.TOKEN_PROGRAM_ID)) {
    console.error("Mint account is not owned by the SPL Token Program");
    throw new Error("Invalid mint account owner");
  }

  console.log("Mint account is correctly owned by the SPL Token Program");
}

// Verify that a token account is owned by the SPL Token Program
async function checkTokenAccountOwnership(tokenAccountAddress: web3.PublicKey) {
  const accountInfo = await splToken.getAccount(
    pg.connection,
    tokenAccountAddress
  );
  if (!accountInfo.owner.equals(splToken.TOKEN_PROGRAM_ID)) {
    console.error("Token account is not owned by the SPL Token Program");
    throw new Error("Invalid token account owner");
  }
}

async function callTestDerivedPDA() {
  // Create a transaction with the test instruction (2)
  const instruction = new web3.TransactionInstruction({
    programId, // Your Solana program's ID
    keys: [
      { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: false },
      { pubkey: pg.wallet.publicKey, isSigner: false, isWritable: false }, // Second placeholder
      { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: false },
    ], // No specific accounts needed here, use a placeholder
    data: Buffer.from([2]), // Instruction ID for `test_derived_pda`
  });

  // Send the transaction
  const transaction = new web3.Transaction().add(instruction);
  const signature = await pg.connection.sendTransaction(transaction, [
    pg.wallet.keypair, // Payer’s keypair
  ]);

  console.log("Onchain Test PDA transaction signature:", signature);
  console.log(
    "Check Solana Explorer or transaction logs for the PDA output from on-chain."
  );
}

async function minimalTest() {
  try {
    console.log("Running minimal test...");
    console.log("Wallet Public Key:", pg.wallet.publicKey.toBase58());

    const balance = await pg.connection.getBalance(pg.wallet.publicKey);
    console.log(`Current balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

    // Function to create the Treasury PDA account
    // async function createTreasuryAccount() {
    //   const [treasuryPDA, bump] = await getTreasuryPDA();

    //   // Calculate lamports needed for rent exemption based on required space (e.g., 128 bytes)
    //   const lamports = await pg.connection.getMinimumBalanceForRentExemption(
    //     40
    //   );

    //   const transaction = new web3.Transaction().add(
    //     web3.SystemProgram.createAccountWithSeed({
    //       fromPubkey: pg.wallet.publicKey,
    //       newAccountPubkey: treasuryPDA,
    //       basePubkey: pg.wallet.publicKey,
    //       seed: seed,
    //       lamports,
    //       space: 40, // Ensure it matches the expected size in your program
    //       programId,
    //     })
    //   );

    //   const signature = await pg.connection.sendTransaction(transaction, [
    //     pg.wallet.keypair,
    //   ]);
    //   console.log(
    //     "Treasury account created successfully. Signature:",
    //     signature
    //   );
    // }

    // Initialize Treasury after the account is created
    // async function initializeTreasury() {
    //   try {
    //     const [treasuryPDA, bump] = await getTreasuryPDA();

    //     console.log("Treasury PDA:", treasuryPDA.toBase58());
    //     console.log("Wallet Public Key:", pg.wallet.publicKey.toBase58());
    //     console.log("Program ID:", programId.toBase58());

    //     // Step 1: Create a dedicated SPL Token Mint account with `treasuryPDA` as the mint authority
    //     const mintAccount = await splToken.createMint(
    //       pg.connection,
    //       pg.wallet.keypair, // Payer
    //       treasuryPDA, // Mint Authority (Treasury PDA)
    //       null, // No Freeze Authority
    //       9, // Decimals
    //       undefined,
    //       undefined,
    //       splToken.TOKEN_PROGRAM_ID
    //     );

    //     console.log("Mint Account created:", mintAccount.toBase58());

    //     // Step 2: Initialize the treasury account with custom program
    //     const transaction = new web3.Transaction().add(
    //       new web3.TransactionInstruction({
    //         keys: [
    //           { pubkey: treasuryPDA, isSigner: false, isWritable: true },
    //           {
    //             pubkey: pg.wallet.publicKey,
    //             isSigner: true,
    //             isWritable: false,
    //           },
    //           {
    //             pubkey: web3.SystemProgram.programId,
    //             isSigner: false,
    //             isWritable: false,
    //           },
    //         ],
    //         programId,
    //         data: Buffer.from([3]), // Instruction identifier for `initialize_treasury`
    //       })
    //     );

    //     const signature = await pg.connection.sendTransaction(transaction, [
    //       pg.wallet.keypair,
    //     ]);
    //     console.log("Treasury initialized successfully. Signature:", signature);

    //     // Return both treasuryPDA and mintAccount for use in minting
    //     return { treasuryPDA, mintAccount };
    //   } catch (error) {
    //     console.error("Initialization step failed:", error);
    //   }
    // }

    // Mint tokens
    async function mintTokens(
      amount: number,
      recipientTokenAccount1?: web3.PublicKey
    ) {
      const [treasuryPDA] = await getTreasuryPDA();
      const [mintAccount] = await getMintAccount();
      // console.log("Mint Account: ", mintAccount.toBase58());
      // return;

      // Ensure the recipient token account is initialized
      const recipientTokenAccount =
        await splToken.getOrCreateAssociatedTokenAccount(
          pg.connection,
          pg.wallet.keypair, // Payer
          mintAccount, // Mint
          pg.wallet.publicKey // Owner
        );

      console.log(
        "Token Account Owner addy: ",
        recipientTokenAccount.owner.toBase58()
      );
      const instruction = new web3.TransactionInstruction({
        programId,
        keys: [
          { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true }, // Payer
          { pubkey: mintAccount, isSigner: false, isWritable: true }, // Mint Account
          {
            pubkey: recipientTokenAccount1 || recipientTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // Recipient's token account
          { pubkey: treasuryPDA, isSigner: false, isWritable: false }, // Mint Authority (PDA), no signer privileges
          {
            pubkey: web3.SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: splToken.TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          },
        ],
        data: Buffer.from([0, ...new BN(amount).toArray("le", 8)]), // Instruction data for `mint_tokens`
      });

      const transaction = new web3.Transaction().add(instruction);
      const signature = await pg.connection.sendTransaction(transaction, [
        pg.wallet.keypair,
      ]);

      console.log(`Minted ${amount} tokens. Signature:`, signature);
    }

    // Burn Tokens
    async function burnTokens(amount: number) {
      const [treasuryPDA] = await getTreasuryPDA(); // Burn authority (PDA)
      const [mintAccount] = await getMintAccount(); // Mint account

      // Retrieve or create the associated token account
      const recipientTokenAccount =
        await splToken.getOrCreateAssociatedTokenAccount(
          pg.connection,
          pg.wallet.keypair, // Payer
          mintAccount, // Mint address
          pg.wallet.publicKey // Owner of the token account
        );

      console.log("Treasury PDA:", treasuryPDA.toBase58());
      console.log("Mint Account:", mintAccount.toBase58());
      console.log(
        "Token Account Address:",
        recipientTokenAccount.address.toBase58()
      );
      console.log(
        "Token Account Owner:",
        recipientTokenAccount.owner.toBase58()
      );

      // Create the instruction to burn tokens
      const instruction = new web3.TransactionInstruction({
        programId, // Your program's ID
        keys: [
          { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true }, // Payer
          {
            pubkey: recipientTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // Burn Account
          { pubkey: mintAccount, isSigner: false, isWritable: true }, // Mint Account
          { pubkey: treasuryPDA, isSigner: false, isWritable: false }, // Burn Authority (PDA)
          {
            pubkey: web3.SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: splToken.TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: web3.SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
          }, // Sysvar Rent
        ],
        data: Buffer.from(Uint8Array.of(1, ...new BN(amount).toArray("le", 8))), // Instruction data
      });

      const transaction = new web3.Transaction().add(instruction);

      // Send the transaction
      const signature = await pg.connection.sendTransaction(transaction, [
        pg.wallet.keypair, // Payer’s keypair
      ]);

      console.log(`Burned ${amount} tokens. Signature:`, signature);
    }

    // Transfer Tokens
    async function transferTokens(
      amount: number,
      recipientPublicKeyStr: string
    ) {
      // Convert the recipient's public key from string to PublicKey
      const recipientPublicKey = new web3.PublicKey(recipientPublicKeyStr);
      const [mintAccount] = await getMintAccount();

      // Derive the sender's associated token account
      const senderTokenAccount =
        await splToken.getOrCreateAssociatedTokenAccount(
          pg.connection,
          pg.wallet.keypair,
          mintAccount,
          pg.wallet.publicKey
        );

      // Derive the recipient's associated token account
      const recipientTokenAccount =
        await splToken.getOrCreateAssociatedTokenAccount(
          pg.connection,
          pg.wallet.keypair,
          mintAccount,
          recipientPublicKey
        );

      console.log(
        "Sender Token Account:",
        senderTokenAccount.address.toBase58()
      );
      console.log(
        "Recipient Token Account:",
        recipientTokenAccount.address.toBase58()
      );

      // Create the transfer instruction with correct identifier (4)
      const instruction = new web3.TransactionInstruction({
        programId,
        keys: [
          {
            pubkey: senderTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // Sender Token Account
          {
            pubkey: recipientTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // Recipient Token Account
          { pubkey: mintAccount, isSigner: false, isWritable: false }, // Mint Account
          { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: false }, // Sender (Owner of the sender token account)
          {
            pubkey: web3.SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: splToken.TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          },
        ],
        data: Buffer.from(Uint8Array.of(4, ...new BN(amount).toArray("le", 8))), // Correct identifier for `transfer_tokens`
      });

      // Send the transaction
      const transaction = new web3.Transaction().add(instruction);
      const signature = await pg.connection.sendTransaction(transaction, [
        pg.wallet.keypair, // Sender's keypair
      ]);

      console.log(`Transferred ${amount} tokens. Signature:`, signature);
    }

    // Stake Tokens
    async function stakeTokens(amount: number) {
      const [treasuryPDA, bump1] = await getTreasuryPDA();
      const [mintAccount, bump2] = await getMintAccount();

      // Derive the user's associated token account
      const userTokenAccount = await splToken.getOrCreateAssociatedTokenAccount(
        pg.connection,
        pg.wallet.keypair, // Payer
        mintAccount, // Mint
        pg.wallet.publicKey // Owner
      );
      console.log(
        "Found the User Token Account: ",
        userTokenAccount.address.toBase58()
      );

      // Derive the Treasury PDA's associated token account
      const treasuryTokenAccount =
        await splToken.getOrCreateAssociatedTokenAccount(
          pg.connection,
          pg.wallet.keypair, // Payer
          mintAccount, // Mint
          treasuryPDA, // Owner
          true
        );

      console.log(
        "Treasury Token Account:",
        treasuryTokenAccount.address.toBase58()
      );
      const [stakerPDA] = await web3.PublicKey.findProgramAddress(
        [pg.wallet.publicKey.toBuffer(), Buffer.from("staker")],
        programId
      );
      // Create the staking instruction
      const instruction = new web3.TransactionInstruction({
        programId,
        keys: [
          {
            pubkey: userTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // User's token account
          {
            pubkey: treasuryTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // Treasury's token account
          { pubkey: treasuryPDA, isSigner: false, isWritable: false }, // Treasury PDA
          { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: false }, // User
          {
            pubkey: splToken.TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: web3.SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: stakerPDA,
            isSigner: false,
            isWritable: true,
          }, // Staker PDA,
        ],
        data: Buffer.from(Uint8Array.of(5, ...new BN(amount).toArray("le", 8))), // Identifier `5` for `stake_tokens`
      });

      const transaction = new web3.Transaction().add(instruction);
      const signature = await pg.connection.sendTransaction(
        transaction,
        [
          pg.wallet.keypair, // User's keypair,
        ]
        // {
        //   skipPreflight: true,
        //   preflightCommitment: "confirmed", // Ensures logs are retrieved after execution
        // }
      );

      console.log(`Staked ${amount} tokens. Signature:`, signature);
    }

    // Unstake Tokens
    async function unstakeTokens(amount: number) {
      const [treasuryPDA] = await getTreasuryPDA();
      const [mintAccount] = await getMintAccount();

      // Derive the user's associated token account
      const userTokenAccount = await splToken.getOrCreateAssociatedTokenAccount(
        pg.connection,
        pg.wallet.keypair, // Payer
        mintAccount, // Mint
        pg.wallet.publicKey // Owner
      );

      // Derive the Treasury PDA's associated token account
      const treasuryTokenAccount =
        await splToken.getOrCreateAssociatedTokenAccount(
          pg.connection,
          pg.wallet.keypair, // Payer
          mintAccount, // Mint
          treasuryPDA, // Owner
          true // Allow PDA as owner
        );

      console.log("User Token Account:", userTokenAccount.address.toBase58());
      console.log(
        "Treasury Token Account:",
        treasuryTokenAccount.address.toBase58()
      );

      // Derive the staker account (PDA) for the user
      const [stakerAccount] = await web3.PublicKey.findProgramAddress(
        [pg.wallet.publicKey.toBuffer(), Buffer.from("staker")],
        programId
      );

      console.log("Staker Account PDA:", stakerAccount.toBase58());

      // Create the unstaking instruction
      const instruction = new web3.TransactionInstruction({
        programId,
        keys: [
          { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: false }, // User
          { pubkey: stakerAccount, isSigner: false, isWritable: true }, // Staker's metadata account
          {
            pubkey: userTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // User's token account
          {
            pubkey: treasuryTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // Treasury's token account
          { pubkey: treasuryPDA, isSigner: false, isWritable: false }, // Treasury PDA
          {
            pubkey: splToken.TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          }, // Token Program
        ],
        data: Buffer.from(Uint8Array.of(6, ...new BN(amount).toArray("le", 8))), // Identifier `6` for `unstake_tokens`
      });

      // Send the transaction
      const transaction = new web3.Transaction().add(instruction);
      const signature = await pg.connection.sendTransaction(transaction, [
        pg.wallet.keypair, // User's keypair
      ]);

      console.log(`Unstaked ${amount} tokens. Signature:`, signature);
    }

    // Purchase Ticket
    async function purchaseTickets(
      amount: number,
      vestingPeriod: number //in seconds
    ) {
      const [treasuryPDA] = await getTreasuryPDA();
      const [mintAccount] = await getMintAccount();

      // Derive the buyer's associated token account
      const buyerTokenAccount =
        await splToken.getOrCreateAssociatedTokenAccount(
          pg.connection,
          pg.wallet.keypair, // Payer
          mintAccount, // Mint
          pg.wallet.publicKey // Owner
        );

      // Derive the Treasury PDA's associated token account
      const treasuryTokenAccount =
        await splToken.getOrCreateAssociatedTokenAccount(
          pg.connection,
          pg.wallet.keypair, // Payer
          mintAccount, // Mint
          treasuryPDA, // Owner
          true
        );

      console.log("Buyer Token Account:", buyerTokenAccount.address.toBase58());
      console.log(
        "Treasury Token Account:",
        treasuryTokenAccount.address.toBase58()
      );

      // Derive the Ticket PDA
      const [ticketPDA] = await web3.PublicKey.findProgramAddress(
        [Buffer.from("ticket_seed"), pg.wallet.publicKey.toBuffer()],
        programId
      );

      // Create the instruction data buffer
      const instructionData = Buffer.concat([
        Buffer.from(Uint8Array.of(7)), // Instruction identifier
        new BN(amount).toArrayLike(Buffer, "le", 8), // Number of tickets
        new BN(vestingPeriod).toArrayLike(Buffer, "le", 8), // Vesting period
      ]);

      // Create the purchase instruction
      const instruction = new web3.TransactionInstruction({
        programId,
        keys: [
          { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: false }, // Buyer
          {
            pubkey: buyerTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // Buyer's token account
          {
            pubkey: treasuryTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // Treasury token account
          // { pubkey: treasuryPDA, isSigner: false, isWritable: false }, // Treasury PDA
          { pubkey: ticketPDA, isSigner: false, isWritable: true }, // Ticket PDA
          {
            pubkey: splToken.TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          }, // Token program
          {
            pubkey: web3.SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          }, // Add System Program account here
        ],
        data: instructionData, // Encoded arguments
      });

      const transaction = new web3.Transaction().add(instruction);
      const signature = await pg.connection.sendTransaction(transaction, [
        pg.wallet.keypair, // Buyer's keypair
      ]);

      console.log(
        `Purchased ${amount} tokens worth of tickets. Signature:`,
        signature
      );
    }

    // Redeem Ticket
    async function redeemTickets(amount: number) {
      const [treasuryPDA] = await getTreasuryPDA();
      const [ticketPDA] = await getTicketPDA(pg.wallet.publicKey);
      const [mintAccount] = await getMintAccount();

      // Derive the buyer's associated token account
      const userTokenAccount = await splToken.getOrCreateAssociatedTokenAccount(
        pg.connection,
        pg.wallet.keypair, // Payer
        mintAccount, // Mint
        pg.wallet.publicKey // Owner
      );

      console.log("Ticket PDA:", ticketPDA.toBase58());
      console.log("User Token Account:", userTokenAccount.address.toBase58());

      // Create the redeem ticket instruction
      const instruction = new web3.TransactionInstruction({
        programId,
        keys: [
          { pubkey: ticketPDA, isSigner: false, isWritable: true }, // Ticket account
          { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: false }, // Owner account
          { pubkey: mintAccount, isSigner: false, isWritable: false }, // Mint account
          {
            pubkey: userTokenAccount.address,
            isSigner: false,
            isWritable: true,
          }, // Owner's token account
          { pubkey: treasuryPDA, isSigner: false, isWritable: false }, // Treasury PDA
          {
            pubkey: splToken.TOKEN_PROGRAM_ID,
            isSigner: false,
            isWritable: false,
          }, // Token program
        ],
        data: Buffer.from(
          Uint8Array.of(8, ...new BN(amount).toArray("le", 8)) // Instruction identifier and arguments
        ),
      });

      // Create and send the transaction
      try {
        const transaction = new web3.Transaction().add(instruction);
        const signature = await pg.connection.sendTransaction(transaction, [
          pg.wallet.keypair, // Owner's keypair
        ]);

        console.log(
          "Redeemed ticket and credited yield. Signature:",
          signature
        );
      } catch (error) {
        console.error("Failed to redeem ticket:", error);
      }
    }

    // Run All Tests async function runTests() {
    //await mintTokens(50000000);
    // await burnTokens(200);
    //await transferTokens(100, "GL8UPqjDgE2VgVDe8LoKvYxFahozAfNyHr8qhhZLFjgk"); // Replace with actual recipient public key
    // await stakeTokens(500); // Stake for 1 day
    //await unstakeTokens(200);
    await purchaseTickets(5586592, 860); // 1-day vesting period
    //await redeemTickets(4);
  } catch (error) {
    console.error("Minimal test failed with error:", error);
  }
}

minimalTest();
