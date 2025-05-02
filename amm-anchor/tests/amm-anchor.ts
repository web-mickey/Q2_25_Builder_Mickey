import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BN } from "bn.js";
import { randomBytes } from "node:crypto";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import { AmmAnchor } from "../target/types/amm_anchor";

describe("amm-anchor", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();

  anchor.setProvider(provider);

  const connection = provider.connection;

  const program = anchor.workspace.AmmAnchor as Program<AmmAnchor>;

  let mintX: anchor.web3.PublicKey;
  let mintY: anchor.web3.PublicKey;

  let poolCreatorAtaX: anchor.web3.PublicKey;
  let poolCreatorAtaY: anchor.web3.PublicKey;

  let poolCreatorMintLpAta: anchor.web3.PublicKey;

  let vaultXAta: anchor.web3.PublicKey;
  let vaultYAta: anchor.web3.PublicKey;

  let mintLp: anchor.web3.PublicKey;
  let mintLpAta: anchor.web3.PublicKey;

  let pool: anchor.web3.PublicKey;

  let poolCreator = Keypair.generate();

  let user = Keypair.generate();

  let userAtaX: anchor.web3.PublicKey;
  let userAtaY: anchor.web3.PublicKey;

  let poolId = new BN(randomBytes(8));

  let lpTokensMinted = new BN(100);

  let maxX = new BN(100);
  let maxY = new BN(100);

  const poolCreatorMintXMintAmount = 200;
  const poolCreatorMintYMintAmount = 200;

  before(async () => {
    let poolCreatorAirdrop = await connection.requestAirdrop(
      poolCreator.publicKey,
      10 * LAMPORTS_PER_SOL
    );

    let userAirdrop = await connection.requestAirdrop(
      user.publicKey,
      8 * LAMPORTS_PER_SOL
    );

    const latestBlockhash = await connection.getLatestBlockhash();

    await connection.confirmTransaction({
      signature: poolCreatorAirdrop,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    });

    await connection.confirmTransaction({
      signature: userAirdrop,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    });

    // Create both mints for X and Y
    mintX = await createMint(
      connection,
      poolCreator,
      poolCreator.publicKey,
      null,
      6
    );

    mintY = await createMint(
      connection,
      poolCreator,
      poolCreator.publicKey,
      null,
      6
    );

    // Create pool ATA X and Y
    poolCreatorAtaX = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        poolCreator,
        mintX,
        poolCreator.publicKey
      )
    ).address;

    poolCreatorAtaY = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        poolCreator,
        mintY,
        poolCreator.publicKey
      )
    ).address;

    // Create user ATA X and Y
    userAtaX = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        user,
        mintX,
        user.publicKey
      )
    ).address;

    userAtaY = (
      await getOrCreateAssociatedTokenAccount(
        connection,
        user,
        mintY,
        user.publicKey
      )
    ).address;

    // Mint tokens to pool creator
    await mintTo(
      connection,
      poolCreator,
      mintX,
      poolCreatorAtaX,
      poolCreator,
      poolCreatorMintXMintAmount
    );
    await mintTo(
      connection,
      poolCreator,
      mintY,
      poolCreatorAtaY,
      poolCreator,
      poolCreatorMintYMintAmount
    );

    // Mint tokens to user
    await mintTo(connection, user, mintX, userAtaX, poolCreator, 100);
    await mintTo(connection, user, mintY, userAtaY, poolCreator, 100);

    [pool] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("pool"),
        mintX.toBuffer(),
        mintY.toBuffer(),
        poolId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    [mintLp] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), pool.toBuffer()],
      program.programId
    );

    vaultXAta = getAssociatedTokenAddressSync(mintX, pool, true);

    vaultYAta = getAssociatedTokenAddressSync(mintY, pool, true);

    mintLpAta = getAssociatedTokenAddressSync(mintLp, pool, true);

    poolCreatorMintLpAta = getAssociatedTokenAddressSync(
      mintLp,
      poolCreator.publicKey
    );
  });

  it("Pool was created and lp tokens were minted!", async () => {
    let accounts = {
      owner: poolCreator.publicKey,
      mintX,
      mintY,
      mintLp,
      mintLpAta,
      ownerMintLpAta: poolCreatorMintLpAta,
      ownerXAta: poolCreatorAtaX,
      ownerYAta: poolCreatorAtaY,
      vaultXAta,
      vaultYAta,
      poolState: pool,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    };

    // Add your test here.

    await program.methods
      .initializePool(poolId, 0, lpTokensMinted, maxX, maxY)
      .accounts(accounts)
      .signers([poolCreator])
      .preInstructions([
        anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
          units: 400_000,
        }),
      ])
      .rpc();

    let poolOwnerLpAta = await getAccount(connection, poolCreatorMintLpAta);

    const ownerMintXAta = await getAccount(connection, poolCreatorAtaX);
    const ownerMintYAta = await getAccount(connection, poolCreatorAtaY);

    const mintLpInfo = await getMint(connection, mintLp);

    const vaultXInfo = await getAccount(connection, vaultXAta);
    const vaultYInfo = await getAccount(connection, vaultXAta);

    assert.equal(poolOwnerLpAta.amount, BigInt(100));
    assert.equal(
      ownerMintXAta.amount,
      BigInt(poolCreatorMintXMintAmount - 100)
    );
    assert.equal(
      ownerMintYAta.amount,
      BigInt(poolCreatorMintYMintAmount - 100)
    );
    assert.equal(mintLpInfo.supply, BigInt(100));
    assert.equal(vaultXInfo.amount, BigInt(100));
    assert.equal(vaultYInfo.amount, BigInt(100));
  });

  it("Another user swapped tokens from X to Y", async () => {
    const vaultXInfo = await getAccount(connection, vaultXAta);
    const vaultYInfo = await getAccount(connection, vaultYAta);

    const userAtaXInfoBefore = await getAccount(connection, userAtaX);
    const userAtaYInfoBefore = await getAccount(connection, userAtaY);

    const vaultXAmount = vaultXInfo.amount;
    const vaultYAmount = vaultYInfo.amount;

    const amountIn = BigInt(5);

    const k = vaultXAmount * vaultYAmount;
    const newXAmount = vaultXAmount + amountIn;
    const newYAmount = Math.ceil(Number(k) / Number(newXAmount));
    const amountOut = vaultYAmount - BigInt(newYAmount);

    await program.methods
      .swap(
        poolId,
        true,
        new BN(amountIn.toString()),
        new BN(amountOut.toString())
      )
      .accountsStrict({
        signer: user.publicKey,
        owner: poolCreator.publicKey,
        mintX,
        mintY,
        mintLp,
        mintLpAta,
        vaultXAta,
        vaultYAta,
        signerXAta: userAtaX,
        signerYAta: userAtaY,
        pool,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const vaultXInfoAfter = await getAccount(connection, vaultXAta);
    const vaultYInfoAfter = await getAccount(connection, vaultYAta);

    const userAtaXInfoAfter = await getAccount(connection, userAtaX);
    const userAtaYInfoAfter = await getAccount(connection, userAtaY);

    assert.equal(vaultXInfoAfter.amount, BigInt(105));
    assert.equal(vaultYInfoAfter.amount, BigInt(96));
    assert.equal(
      userAtaXInfoAfter.amount,
      userAtaXInfoBefore.amount - amountIn
    );
    assert.equal(
      userAtaYInfoAfter.amount,
      userAtaYInfoBefore.amount + amountOut
    );
  });

  it("Same user swapped tokens from Y to X", async () => {
    const vaultXInfo = await getAccount(connection, vaultXAta);
    const vaultYInfo = await getAccount(connection, vaultYAta);

    const userAtaXInfoBefore = await getAccount(connection, userAtaX);
    const userAtaYInfoBefore = await getAccount(connection, userAtaY);

    const vaultXAmount = vaultXInfo.amount;
    const vaultYAmount = vaultYInfo.amount;

    const amountIn = BigInt(5);

    const k = vaultXAmount * vaultYAmount;
    const newYAmount = vaultYAmount + amountIn;
    const newXAmount = Math.ceil(Number(k) / Number(newYAmount));
    const amountOut = vaultXAmount - BigInt(newXAmount);

    await program.methods
      .swap(
        poolId,
        false,
        new BN(amountIn.toString()),
        new BN(amountOut.toString())
      )
      .accountsStrict({
        signer: user.publicKey,
        owner: poolCreator.publicKey,
        mintX,
        mintY,
        mintLp,
        mintLpAta,
        vaultXAta,
        vaultYAta,
        signerXAta: userAtaX,
        signerYAta: userAtaY,
        pool,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    const vaultXInfoAfter = await getAccount(connection, vaultXAta);
    const vaultYInfoAfter = await getAccount(connection, vaultYAta);

    const userAtaXInfoAfter = await getAccount(connection, userAtaX);
    const userAtaYInfoAfter = await getAccount(connection, userAtaY);

    assert.equal(vaultXInfoAfter.amount, BigInt(100));
    assert.equal(vaultYInfoAfter.amount, BigInt(101));
    assert.equal(
      userAtaXInfoAfter.amount,
      userAtaXInfoBefore.amount + amountOut
    );
    assert.equal(
      userAtaYInfoAfter.amount,
      userAtaYInfoBefore.amount - amountIn
    );
  });

  it("Pool owner withdrawed his lp tokens from pool", async () => {
    // Get initial balances
    const vaultXInfoBefore = await getAccount(connection, vaultXAta);
    const vaultYInfoBefore = await getAccount(connection, vaultYAta);
    const ownerXBefore = await getAccount(connection, poolCreatorAtaX);
    const ownerYBefore = await getAccount(connection, poolCreatorAtaY);

    await program.methods
      .withdrawFromPool(poolId, lpTokensMinted, maxX, maxY)
      .accountsStrict({
        owner: poolCreator.publicKey,
        mintX,
        mintY,
        mintLp,
        mintLpAta,
        vaultMintXAta: vaultXAta,
        vaultMintYAta: vaultYAta,
        ownerMintXAta: poolCreatorAtaX,
        ownerMintYAta: poolCreatorAtaY,
        ownerMintLpAta: poolCreatorMintLpAta,
        pool,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([poolCreator])
      .rpc();

    // Get final balances
    const vaultXInfoAfter = await getAccount(connection, vaultXAta);
    const vaultYInfoAfter = await getAccount(connection, vaultYAta);
    const ownerXAfter = await getAccount(connection, poolCreatorAtaX);
    const ownerYAfter = await getAccount(connection, poolCreatorAtaY);
    const ownerMintLpAta = await getAccount(connection, poolCreatorMintLpAta);

    // Verify tokens were moved from vaults to owner ATAs
    assert.equal(
      ownerXAfter.amount - ownerXBefore.amount,
      vaultXInfoBefore.amount - vaultXInfoAfter.amount
    );
    assert.equal(
      ownerYAfter.amount - ownerYBefore.amount,
      vaultYInfoBefore.amount - vaultYInfoAfter.amount
    );
    assert.equal(ownerMintLpAta.amount, BigInt(0));
  });
});
