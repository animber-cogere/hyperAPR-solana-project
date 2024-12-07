import * as splToken from "@solana/spl-token";

// Constants for program ID and PDA seeds
const programId = new web3.PublicKey(
  "ABHENVYtMXfAdN741mJzwoLtqGW7ntpT9uhr2f1Q7wB1"
);
const treasury_seed = "treasurythissuperhyperAPRtoken";
const mint_seed = "mintthissuperhyperAPRtoken";

// Function to get the Treasury PDA
async function getTreasuryPDA() {
  return web3.PublicKey.findProgramAddressSync(
    [Buffer.from(treasury_seed)],
    programId
  );
}

// Function to get the Mint PDA
async function getMintPDA() {
  return web3.PublicKey.findProgramAddressSync(
    [Buffer.from(mint_seed)],
    programId
  );
}

async function isTreasuryInitialized(): Promise<boolean> {
  try {
    const [treasuryPDA] = await getTreasuryPDA();
    const treasuryAccountInfo = await pg.connection.getAccountInfo(treasuryPDA);

    if (!treasuryAccountInfo) {
      console.log("Treasury account does not exist.");
      return false;
    }

    const treasuryData = treasuryAccountInfo.data;
    if (treasuryData && treasuryData.length > 0 && treasuryData[0] === 1) {
      const isInitialized = treasuryData[0] === 1; // Check the `is_initialized` flag
      console.log(`Treasury account already initialized!`);
      return isInitialized;
    } else {
      //console.log("Treasury account data is empty or invalid.");
      return false;
    }
  } catch (error) {
    console.error("Failed to check treasury initialization:", error);
    return false;
  }
}

async function createTreasuryTokenAccount() {
  const [treasuryPDA] = await getTreasuryPDA();
  const [mintAccount] = await getMintPDA();

  console.log("Treasury PDA:", treasuryPDA.toBase58());
  console.log("Mint Account:", mintAccount.toBase58());

  try {
    // Use `getOrCreateAssociatedTokenAccount` with `allowOwnerOffCurve: true`
    const treasuryTokenAccount =
      await splToken.getOrCreateAssociatedTokenAccount(
        pg.connection, // Connection to Solana
        pg.wallet.keypair, // Payer of fees
        mintAccount, // Mint associated with the account
        treasuryPDA, // Owner of the ATA (PDA)
        true // Allow owner to be off curve (PDA)
      );

    console.log(
      "Treasury Token Account:",
      treasuryTokenAccount.address.toBase58()
    );
    console.log("Treasury Token Account already initialized.");
    return treasuryTokenAccount.address;
  } catch (error) {
    console.error("Error creating Treasury Token Account:", error);
    throw error;
  }
}

// Initialize Treasury after the account is created
// Ensure the accounts match the order expected by the on-chain function.
async function initializeTreasury() {
  const [treasuryPDA] = await getTreasuryPDA();
  if (await isTreasuryInitialized()) {
    return;
  }

  const [mintPDA] = await getMintPDA();

  // const treasuryTokenAccount = (
  //   await splToken.getOrCreateAssociatedTokenAccount(
  //     pg.connection,
  //     pg.wallet.keypair, // Payer
  //     mintAccount[0], // Mint
  //     treasuryPDA // Owner
  //   )
  // ).address;

  //OR YOU CAN DO THIS WAY:

  // Derive the Treasury Token Account address
  // let treasuryTokenAccount = await splToken.getAssociatedTokenAddress(
  //   mintPDA, // Mint Address
  //   treasuryPDA, // Treasury PDA as owner
  //   true // Allow creation of associated token account
  // );

  console.log("Created the Treasury  ATA!");
  console.log("Attempting to initialize the treasury in TS now...");

  // console.log("Client Treasury PDA: ", treasuryPDA.toBase58());
  // console.log("Client Mint Account: ", mintAccount[0].toBase58());
  // console.log(
  //   "Client Treasury Token Account: ",
  //   treasuryTokenAccount.toBase58()
  // );

  const transaction = new web3.Transaction().add(
    new web3.TransactionInstruction({
      programId,
      keys: [
        { pubkey: treasuryPDA, isSigner: false, isWritable: true },
        { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: false },
        { pubkey: mintPDA, isSigner: false, isWritable: true },
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
        { pubkey: web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }, // Add SysvarRent here
        // {
        //   pubkey: treasuryTokenAccount,
        //   isSigner: false,
        //   isWritable: true,
        // }, // Treasury Token Account
      ],
      data: Buffer.from([3]), // Instruction to initialize
    })
  );
  // transaction.add(
  //   web3.ComputeBudgetProgram.setComputeUnitLimit({
  //     units: 300000, // Adjust as necessary
  //   })
  // );

  const signature = await pg.connection.sendTransaction(
    transaction,
    [pg.wallet.keypair]
    //IMPORTANT: TURN THE NEXT THREE LINES ON IF YOU WANT TO READ ON CHAIN DEBUG MESSAGES AND SOLANA PLAYGROUND ISNT SHOWING THEM TO YOU IN YOUR LOGS!
    // {
    //   skipPreflight: true,
    //   preflightCommitment: "confirmed", // Ensures logs are retrieved after execution
    // }
  );

  console.log("Treasury initialized successfully. Signature:", signature);
}

initializeTreasury().then(() => {
  createTreasuryTokenAccount();
});
