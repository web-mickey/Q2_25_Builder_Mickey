import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { randomBytes } from "crypto";
import { Dexera } from "../target/types/dexera";
import { expect } from "chai";

describe("dexera", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.dexera as Program<Dexera>;
  const programId = program.programId;

  const tokenProgram = TOKEN_PROGRAM_ID;
  const associatedTokenProgram = ASSOCIATED_TOKEN_PROGRAM_ID;

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      ...block,
      signature,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  const profileId = new BN(randomBytes(8));

  const [
    dexAdmin,
    profileCreator,
    trader,
    poolCreator,
    mintX,
    mintY,
    protocolFeeAccount,
  ] = Array.from({ length: 7 }, () => Keypair.generate());

  const [poolCreatorAtaX, poolCreatorAtaY, traderAtaX, traderAtaY] = [
    poolCreator,
    trader,
  ]
    .map((a) =>
      [mintX, mintY].map((m) =>
        getAssociatedTokenAddressSync(
          m.publicKey,
          a.publicKey,
          false,
          tokenProgram
        )
      )
    )
    .flat();

  const [pool] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pool"),
      mintX.publicKey.toBuffer(),
      mintY.publicKey.toBuffer(),
    ],
    programId
  );

  const [mintLp] = PublicKey.findProgramAddressSync(
    [Buffer.from("lp"), pool.toBuffer()],
    programId
  );

  const poolAtaLp = getAssociatedTokenAddressSync(
    mintLp,
    pool,
    true,
    tokenProgram
  );

  const poolCreatorLpAta = getAssociatedTokenAddressSync(
    mintLp,
    poolCreator.publicKey,
    false,
    tokenProgram
  );

  const [config] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );

  const [profile] = PublicKey.findProgramAddressSync(
    [Buffer.from("profile"), profileCreator.publicKey.toBuffer()],
    program.programId
  );

  const [poolAtaX, poolAtaY] = [mintX, mintY].map((m) =>
    getAssociatedTokenAddressSync(m.publicKey, pool, true, tokenProgram)
  );

  const accounts = {
    dexAdmin: dexAdmin.publicKey,
    profileCreator: profileCreator.publicKey,
    poolCreator: poolCreator.publicKey,
    mintX: mintX.publicKey,
    mintY: mintY.publicKey,
    poolAtaX,
    poolAtaY,
    pool,
    profile,
    protocolFeeAccount: protocolFeeAccount.publicKey,
    poolAtaLp,
    poolCreatorLpAta,
    mintLp,
    poolCreatorAtaY,
    traderAtaY,
    config,
  };

  it("Airdrop and create mints", async () => {
    let lamports = await getMinimumBalanceForRentExemptMint(connection);
    let tx = new Transaction();

    // Airdrop more SOL to cover account creation
    tx.instructions = [
      ...[
        dexAdmin,
        poolCreator,
        trader,
        profileCreator,
        protocolFeeAccount,
      ].map((account) =>
        SystemProgram.transfer({
          fromPubkey: provider.publicKey,
          toPubkey: account.publicKey,
          lamports: 200 * LAMPORTS_PER_SOL,
        })
      ),
      ...[mintX, mintY].map((mint) =>
        SystemProgram.createAccount({
          fromPubkey: provider.publicKey,
          newAccountPubkey: mint.publicKey,
          lamports,
          space: MINT_SIZE,
          programId: tokenProgram,
        })
      ),
    ];

    await provider.sendAndConfirm(tx, [mintX, mintY]).then(log);

    // Create token accounts in a separate transaction
    tx = new Transaction();
    tx.instructions = [
      ...[
        {
          mint: mintX.publicKey,
          authority: poolCreator.publicKey,
          ata: poolCreatorAtaX,
        },
        {
          mint: mintY.publicKey,
          authority: poolCreator.publicKey,
          ata: poolCreatorAtaY,
        },
      ].flatMap((x) => [
        createInitializeMint2Instruction(
          x.mint,
          6,
          x.authority,
          null,
          tokenProgram
        ),
        createAssociatedTokenAccountIdempotentInstruction(
          provider.publicKey,
          x.ata,
          x.authority,
          x.mint,
          tokenProgram
        ),
        createMintToInstruction(
          x.mint,
          x.ata,
          x.authority,
          1e9,
          undefined,
          tokenProgram
        ),
      ]),
      // Do the same for trader mint X
      createAssociatedTokenAccountIdempotentInstruction(
        provider.publicKey,
        traderAtaX,
        trader.publicKey,
        mintX.publicKey,
        tokenProgram
      ),
      createMintToInstruction(
        mintX.publicKey,
        traderAtaX,
        poolCreator.publicKey,
        1e9,
        undefined,
        tokenProgram
      ),
      // Create trader's Y token account
      createAssociatedTokenAccountIdempotentInstruction(
        provider.publicKey,
        traderAtaY,
        trader.publicKey,
        mintY.publicKey,
        tokenProgram
      ),
      createMintToInstruction(
        mintY.publicKey,
        traderAtaY,
        poolCreator.publicKey,
        1e9,
        undefined,
        tokenProgram
      ),
      // Create referrer's (profileCreator) Y token account
      createAssociatedTokenAccountIdempotentInstruction(
        provider.publicKey,
        getAssociatedTokenAddressSync(
          mintY.publicKey,
          profileCreator.publicKey,
          false,
          tokenProgram
        ),
        profileCreator.publicKey,
        mintY.publicKey,
        tokenProgram
      ),
      // Create protocol fee account's Y token account
      createAssociatedTokenAccountIdempotentInstruction(
        provider.publicKey,
        getAssociatedTokenAddressSync(
          mintY.publicKey,
          protocolFeeAccount.publicKey,
          false,
          tokenProgram
        ),
        protocolFeeAccount.publicKey,
        mintY.publicKey,
        tokenProgram
      ),
    ];

    await provider.sendAndConfirm(tx, [poolCreator]).then(log);
  });

  it("Initialize protocol", async () => {
    // Initialize protocol
    await program.methods
      .initializeProtocol(100)
      .accountsStrict({
        admin: dexAdmin.publicKey,
        config,
        protocolFeeAccount: protocolFeeAccount.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([dexAdmin])
      .rpc()
      .then(log);
  });

  it("Create protocol fee token account", async () => {
    // Create protocol fee account's Y token account
    const tx = new Transaction().add(
      createAssociatedTokenAccountIdempotentInstruction(
        provider.publicKey,
        getAssociatedTokenAddressSync(
          mintY.publicKey,
          protocolFeeAccount.publicKey,
          false,
          tokenProgram
        ),
        protocolFeeAccount.publicKey,
        mintY.publicKey,
        tokenProgram
      )
    );

    await provider.sendAndConfirm(tx).then(log);
  });

  it("Create profile", async () => {
    await program.methods
      .createProfile(profileId)
      .accountsStrict({
        creator: profileCreator.publicKey,
        profile,
        systemProgram: SystemProgram.programId,
      })
      .signers([profileCreator])
      .rpc()
      .then(log);
  });

  it("Create pool", async () => {
    // Create the pool account and its ATAs
    await program.methods
      .createPool()
      .accountsStrict({
        creator: accounts.poolCreator,
        mintX: accounts.mintX,
        mintY: accounts.mintY,
        mintLp: accounts.mintLp,
        poolVaultXAta: accounts.poolAtaX,
        poolVaultYAta: accounts.poolAtaY,
        poolMintLpAta: accounts.poolAtaLp,
        creatorMintXAta: poolCreatorAtaX,
        creatorMintYAta: accounts.poolCreatorAtaY,
        creatorMintLpAta: accounts.poolCreatorLpAta,
        pool: accounts.pool,
        tokenProgram,
        associatedTokenProgram: associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([poolCreator])
      .rpc();
  });

  it("Calculate swap amounts", async () => {
    // Get pool state
    const poolAccount = await program.account.pool.fetch(accounts.pool);
    const poolVaultX = await connection.getTokenAccountBalance(
      accounts.poolAtaX
    );
    const poolVaultY = await connection.getTokenAccountBalance(
      accounts.poolAtaY
    );

    console.log("Pool state:");
    console.log("X tokens in pool:", poolVaultX.value.amount);
    console.log("Y tokens in pool:", poolVaultY.value.amount);

    // Calculate swap amounts using constant product formula
    // For exact out swap (we want 10,000 Y tokens)
    const amountOut = 10000;
    const reserveIn = Number(poolVaultX.value.amount);
    const reserveOut = Number(poolVaultY.value.amount);

    // Calculate amount in using constant product formula: (x + Δx)(y - Δy) = xy
    // where Δy is amountOut and we solve for Δx
    const amountIn = Math.ceil(
      (amountOut * reserveIn) / (reserveOut - amountOut)
    );

    // Add 1% slippage tolerance
    const amountInWithSlippage = Math.ceil(amountIn * 1.01);

    console.log("\nSwap calculation:");
    console.log("Want to receive:", amountOut, "Y tokens");
    console.log("Need to pay:", amountIn, "X tokens");
    console.log("With 1% slippage:", amountInWithSlippage, "X tokens");
    console.log(
      "Price impact:",
      ((amountIn / reserveIn) * 100).toFixed(4),
      "%"
    );

    // Verify the amounts are reasonable
    expect(amountIn).greaterThan(0);
    expect(amountIn).lessThan(reserveIn);
    expect(amountOut).lessThan(reserveOut);

    return { amountIn, amountInWithSlippage, amountOut };
  });

  it("Swap tokens", async () => {
    // Get pool state before swap
    const beforeX = await connection.getTokenAccountBalance(accounts.poolAtaX);
    const beforeY = await connection.getTokenAccountBalance(accounts.poolAtaY);
    const beforeTraderX = await connection.getTokenAccountBalance(traderAtaX);
    const beforeTraderY = await connection.getTokenAccountBalance(traderAtaY);

    // Calculate exact amount needed based on current pool state
    const amountOut = 10000; // exact amount we want to receive
    const reserveIn = Number(beforeX.value.amount);
    const reserveOut = Number(beforeY.value.amount);

    // Calculate amount in using constant product formula: (x + Δx)(y - Δy) = xy
    const amountIn = Math.ceil(
      (amountOut * reserveIn) / (reserveOut - amountOut)
    );

    // Add 10% slippage tolerance to input amount
    const maxAmountIn = Math.ceil(amountIn * 1.1); // willing to pay up to 10% more

    console.log("\nSwap calculation:");
    console.log("Current pool state:");
    console.log("X tokens:", reserveIn);
    console.log("Y tokens:", reserveOut);
    console.log("Want to receive exactly:", amountOut, "Y tokens");
    console.log("Estimated payment:", amountIn, "X tokens");
    console.log("Maximum payment with 10% slippage:", maxAmountIn, "X tokens");
    console.log(
      "Price impact:",
      ((amountIn / reserveIn) * 100).toFixed(4),
      "%"
    );

    await program.methods
      .swapExactOut(new BN(maxAmountIn), new BN(amountOut), null)
      .accountsPartial({
        user: trader.publicKey,
        mintX: accounts.mintX,
        mintY: accounts.mintY,
        mintLp: accounts.mintLp,
        profile: accounts.profile,
        config: accounts.config,
        poolVaultXAta: accounts.poolAtaX,
        poolVaultYAta: accounts.poolAtaY,
        userMintXAta: traderAtaX,
        userMintYAta: traderAtaY,
        pool,
        referrerAta: null,
        tokenProgram,
        associatedTokenProgram: associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([trader])
      .rpc();

    // Get pool state after swap
    const afterX = await connection.getTokenAccountBalance(accounts.poolAtaX);
    const afterY = await connection.getTokenAccountBalance(accounts.poolAtaY);

    const afterTraderX = await connection.getTokenAccountBalance(traderAtaX);
    const afterTraderY = await connection.getTokenAccountBalance(traderAtaY);

    console.log("\nSwap results:");
    console.log(
      "Pool X tokens:",
      beforeX.value.amount,
      "->",
      afterX.value.amount
    );
    console.log(
      "Pool Y tokens:",
      beforeY.value.amount,
      "->",
      afterY.value.amount
    );
    console.log(
      "Trader X tokens:",
      beforeTraderX.value.amount,
      "->",
      afterTraderX.value.amount
    );
    console.log(
      "Trader Y tokens:",
      beforeTraderY.value.amount,
      "->",
      afterTraderY.value.amount
    );
  });

  it("Swap tokens with referrer", async () => {
    // Get referrer's ATA for Y tokens
    const referrerAtaY = getAssociatedTokenAddressSync(
      mintY.publicKey,
      profileCreator.publicKey, // Use profileCreator as referrer
      false,
      tokenProgram
    );

    // Create referrer's ATA for Y tokens if it doesn't exist
    const createAtaIx = createAssociatedTokenAccountIdempotentInstruction(
      provider.publicKey,
      referrerAtaY,
      profileCreator.publicKey,
      mintY.publicKey,
      tokenProgram
    );

    const tx = new Transaction().add(createAtaIx);
    await provider.sendAndConfirm(tx).then(log);

    // Get pool state before swap
    const beforeX = await connection.getTokenAccountBalance(accounts.poolAtaX);
    const beforeY = await connection.getTokenAccountBalance(accounts.poolAtaY);
    const beforeTraderX = await connection.getTokenAccountBalance(traderAtaX);
    const beforeTraderY = await connection.getTokenAccountBalance(traderAtaY);
    const beforeReferrerY = await connection.getTokenAccountBalance(
      referrerAtaY
    );
    const protocolFeeAtaY = getAssociatedTokenAddressSync(
      mintY.publicKey,
      protocolFeeAccount.publicKey,
      false,
      tokenProgram
    );
    const beforeProtocolFeeY = await connection.getTokenAccountBalance(
      protocolFeeAtaY
    );

    // Calculate exact amount needed based on current pool state
    const amountOut = 10000; // exact amount we want to receive
    const reserveIn = Number(beforeX.value.amount);
    const reserveOut = Number(beforeY.value.amount);

    // Calculate amount in using constant product formula: (x + Δx)(y - Δy) = xy
    const amountIn = Math.ceil(
      (amountOut * reserveIn) / (reserveOut - amountOut)
    );

    // Add 10% slippage tolerance to input amount
    const maxAmountIn = Math.ceil(amountIn * 1.1); // willing to pay up to 10% more

    console.log("\nSwap calculation with referrer:");
    console.log("Current pool state:");
    console.log("X tokens:", reserveIn);
    console.log("Y tokens:", reserveOut);
    console.log("Want to receive exactly:", amountOut, "Y tokens");
    console.log("Estimated payment:", amountIn, "X tokens");
    console.log("Maximum payment with 10% slippage:", maxAmountIn, "X tokens");
    console.log(
      "Price impact:",
      ((amountIn / reserveIn) * 100).toFixed(4),
      "%"
    );

    await program.methods
      .swapExactOut(
        new BN(maxAmountIn),
        new BN(amountOut),
        profileCreator.publicKey
      ) // Use profileCreator as referrer
      .accountsPartial({
        user: trader.publicKey,
        mintX: accounts.mintX,
        mintY: accounts.mintY,
        mintLp: accounts.mintLp,
        profile: accounts.profile, // Use existing profile
        config: accounts.config,
        poolVaultXAta: accounts.poolAtaX,
        poolVaultYAta: accounts.poolAtaY,
        userMintXAta: traderAtaX,
        userMintYAta: traderAtaY,
        pool,
        referrerAta: referrerAtaY,
        tokenProgram,
        associatedTokenProgram: associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([trader])
      .rpc();

    // Get pool state after swap
    const afterX = await connection.getTokenAccountBalance(accounts.poolAtaX);
    const afterY = await connection.getTokenAccountBalance(accounts.poolAtaY);
    const afterTraderX = await connection.getTokenAccountBalance(traderAtaX);
    const afterTraderY = await connection.getTokenAccountBalance(traderAtaY);
    const afterReferrerY = await connection.getTokenAccountBalance(
      referrerAtaY
    );
    const afterProtocolFeeY = await connection.getTokenAccountBalance(
      protocolFeeAtaY
    );

    console.log("\nSwap results with referrer:");
    console.log(
      "Pool X tokens:",
      beforeX.value.amount,
      "->",
      afterX.value.amount
    );
    console.log(
      "Pool Y tokens:",
      beforeY.value.amount,
      "->",
      afterY.value.amount
    );
    console.log(
      "Trader X tokens:",
      beforeTraderX.value.amount,
      "->",
      afterTraderX.value.amount
    );
    console.log(
      "Trader Y tokens:",
      beforeTraderY.value.amount,
      "->",
      afterTraderY.value.amount
    );
    console.log("Referrer Y tokens:", "0", "->", afterReferrerY.value.amount);
    console.log(
      "Protocol fee Y tokens:",
      beforeProtocolFeeY.value.amount,
      "->",
      afterProtocolFeeY.value.amount
    );
  });

  it("Withdraw liquidity", async () => {
    // Get pool state before withdrawal
    const beforeX = await connection.getTokenAccountBalance(accounts.poolAtaX);
    const beforeY = await connection.getTokenAccountBalance(accounts.poolAtaY);
    const beforeCreatorLp = await connection.getTokenAccountBalance(
      accounts.poolCreatorLpAta
    );
    const beforeCreatorX = await connection.getTokenAccountBalance(
      poolCreatorAtaX
    );
    const beforeCreatorY = await connection.getTokenAccountBalance(
      accounts.poolCreatorAtaY
    );

    console.log("\nPool state before withdrawal:");
    console.log("Pool X tokens:", beforeX.value.amount);
    console.log("Pool Y tokens:", beforeY.value.amount);
    console.log("Creator LP tokens:", beforeCreatorLp.value.amount);
    console.log("Creator X tokens:", beforeCreatorX.value.amount);
    console.log("Creator Y tokens:", beforeCreatorY.value.amount);

    // Calculate withdrawal amounts
    const lpAmount = Number(beforeCreatorLp.value.amount);
    const totalLpSupply = Number(beforeCreatorLp.value.amount); // Since creator has all LP tokens
    const withdrawAmount = Math.floor(lpAmount * 0.5); // Withdraw 50% of LP tokens

    // Calculate expected token amounts
    const expectedX = Math.floor(
      (withdrawAmount * Number(beforeX.value.amount)) / totalLpSupply
    );
    const expectedY = Math.floor(
      (withdrawAmount * Number(beforeY.value.amount)) / totalLpSupply
    );

    console.log("\nWithdrawal calculation:");
    console.log("Total LP supply:", totalLpSupply);
    console.log("Withdrawing LP tokens:", withdrawAmount);
    console.log("Expected X tokens:", expectedX);
    console.log("Expected Y tokens:", expectedY);

    // Withdraw liquidity
    await program.methods
      .withdrawLiquidity(
        new BN(withdrawAmount),
        new BN(Math.floor(expectedX * 0.99)), // 1% slippage tolerance
        new BN(Math.floor(expectedY * 0.99)) // 1% slippage tolerance
      )
      .accountsPartial({
        withdrawer: poolCreator.publicKey,
        mintX: accounts.mintX,
        mintY: accounts.mintY,
        mintLp: accounts.mintLp,
        poolVaultXAta: accounts.poolAtaX,
        poolVaultYAta: accounts.poolAtaY,
        poolMintLpAta: accounts.poolAtaLp,
        withdrawerMintXAta: poolCreatorAtaX,
        withdrawerMintYAta: accounts.poolCreatorAtaY,
        withdrawerMintLpAta: accounts.poolCreatorLpAta,
        pool: accounts.pool,
        tokenProgram,
        associatedTokenProgram: associatedTokenProgram,
        systemProgram: SystemProgram.programId,
      })
      .signers([poolCreator])
      .rpc();

    // Get pool state after withdrawal
    const afterX = await connection.getTokenAccountBalance(accounts.poolAtaX);
    const afterY = await connection.getTokenAccountBalance(accounts.poolAtaY);
    const afterCreatorLp = await connection.getTokenAccountBalance(
      accounts.poolCreatorLpAta
    );
    const afterCreatorX = await connection.getTokenAccountBalance(
      poolCreatorAtaX
    );
    const afterCreatorY = await connection.getTokenAccountBalance(
      accounts.poolCreatorAtaY
    );

    console.log("\nWithdrawal results:");
    console.log(
      "Pool X tokens:",
      beforeX.value.amount,
      "->",
      afterX.value.amount
    );
    console.log(
      "Pool Y tokens:",
      beforeY.value.amount,
      "->",
      afterY.value.amount
    );
    console.log(
      "Creator LP tokens:",
      beforeCreatorLp.value.amount,
      "->",
      afterCreatorLp.value.amount
    );
    console.log(
      "Creator X tokens:",
      beforeCreatorX.value.amount,
      "->",
      afterCreatorX.value.amount
    );
    console.log(
      "Creator Y tokens:",
      beforeCreatorY.value.amount,
      "->",
      afterCreatorY.value.amount
    );

    // Verify the withdrawal amounts
    const actualX =
      Number(afterCreatorX.value.amount) - Number(beforeCreatorX.value.amount);
    const actualY =
      Number(afterCreatorY.value.amount) - Number(beforeCreatorY.value.amount);
    const actualLp =
      Number(beforeCreatorLp.value.amount) -
      Number(afterCreatorLp.value.amount);

    console.log("\nVerification:");
    console.log("Actual LP tokens burned:", actualLp);
    console.log("Actual X tokens received:", actualX);
    console.log("Actual Y tokens received:", actualY);
    console.log("Expected X tokens:", expectedX);
    console.log("Expected Y tokens:", expectedY);

    // Verify the amounts are reasonable
    expect(actualLp).to.equal(withdrawAmount);
    expect(actualX).to.be.closeTo(expectedX, 1); // Allow 1 token rounding error
    expect(actualY).to.be.closeTo(expectedY, 1); // Allow 1 token rounding error
  });
});
