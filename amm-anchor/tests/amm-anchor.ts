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

  let userAtaX: anchor.web3.PublicKey;
  let userAtaY: anchor.web3.PublicKey;

  let vaultXAta: anchor.web3.PublicKey;
  let vaultYAta: anchor.web3.PublicKey;

  let mintLp: anchor.web3.PublicKey;
  let mintLpAta: anchor.web3.PublicKey;

  let pool: anchor.web3.PublicKey;

  let poolCreator = Keypair.generate();

  let user = Keypair.generate();

  let poolId = new BN(randomBytes(8));

  let amount = new BN(100);

  let maxX = new BN(100);
  let maxY = new BN(100);

  const poolCreatorMintXMintAmount = 200;
  const poolCreatorMintYMintAmount = 200;

  const userMintXMintAmount = 100;
  const userMintYMintAmount = 100;

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

    const tx = await program.methods
      .initializePool(poolId, 100, amount, maxX, maxY)
      .accounts(accounts)
      .signers([poolCreator])
      .preInstructions([
        anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
          units: 400_000,
        }),
      ])
      .rpc();

    let poolOwnerLpAta = await getAccount(connection, poolCreatorMintLpAta);

    console.log("Your transaction signature", tx);

    const poolState = await program.account.pool.fetch(pool);

    const mintLpInfo = await getMint(connection, mintLp);

    assert.equal(poolOwnerLpAta.amount.toString(), "100");
    assert.equal(mintLpInfo.supply.toString(), "100");
  });
});
