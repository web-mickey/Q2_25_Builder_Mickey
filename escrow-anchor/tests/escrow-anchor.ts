import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { seed } from "@coral-xyz/anchor/dist/cjs/idl";
import { BN } from "bn.js";
import { randomBytes } from "node:crypto";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAccount,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { assert } from "chai";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { associatedAddress } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { EscrowAnchor } from "../target/types/escrow_anchor";

describe("escrow-anchor", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();

  anchor.setProvider(provider);

  const connection = provider.connection;

  const program = anchor.workspace.EscrowAnchor as Program<EscrowAnchor>;

  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;

  let makerAtaA: anchor.web3.PublicKey;
  let makerAtaB: anchor.web3.PublicKey;

  let takerAtaA: anchor.web3.PublicKey;
  let takerAtaB: anchor.web3.PublicKey;

  let vaultAtaA: anchor.web3.PublicKey;
  let escrow: anchor.web3.PublicKey;

  let maker = Keypair.generate();
  let taker = Keypair.generate();

  let escrowId = new BN(randomBytes(8));

  let depositAmount = new anchor.BN(100);
  let receiveAmount = new anchor.BN(80);

  const makerMintAMintAmount = 200;
  const takerMintBMintAmount = 100;

  before(async () => {
    let makerAirdrop = await connection.requestAirdrop(
      maker.publicKey,
      10 * LAMPORTS_PER_SOL
    );

    let takerAirdrop = await connection.requestAirdrop(
      taker.publicKey,
      8 * LAMPORTS_PER_SOL
    );

    const latestBlockhash = await connection.getLatestBlockhash();

    await connection.confirmTransaction({
      signature: makerAirdrop,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    });

    await connection.confirmTransaction({
      signature: takerAirdrop,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    });

    mintA = await createMint(connection, maker, maker.publicKey, null, 6);
    mintB = await createMint(connection, taker, taker.publicKey, null, 6);

    makerAtaA = await createAccount(connection, maker, mintA, maker.publicKey);
    makerAtaB = await createAccount(connection, maker, mintB, maker.publicKey);

    takerAtaA = await createAccount(connection, taker, mintA, taker.publicKey);
    takerAtaB = await createAccount(connection, taker, mintB, taker.publicKey);

    await mintTo(
      connection,
      maker,
      mintA,
      makerAtaA,
      maker,
      makerMintAMintAmount
    );
    await mintTo(
      connection,
      taker,
      mintB,
      takerAtaB,
      taker,
      takerMintBMintAmount
    );

    [escrow] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        maker.publicKey.toBuffer(),
        escrowId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    vaultAtaA = anchor.utils.token.associatedAddress({
      mint: mintA,
      owner: escrow,
    });
  });

  it("Create Escrow", async () => {
    await program.methods
      .initialize(escrowId, depositAmount, receiveAmount)
      .accountsPartial({
        maker: maker.publicKey,
        mintA,
        mintB,
        userAtaA: makerAtaA,
        vaultAtaA,
        escrow,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc();

    const vaultAccount = await getAccount(connection, vaultAtaA);

    assert.ok(vaultAccount.amount === BigInt(depositAmount.toString()));
  });

  it("Refund Escrow", async () => {
    let makerMintABalanceBefore = await getAccount(connection, makerAtaA);

    let vaultBalanceBefore = await getAccount(connection, vaultAtaA);

    await program.methods
      .refund()
      .accountsPartial({
        maker: maker.publicKey,
        mintA,
        mintB,
        makerAtaA,
        vaultAtaA,
        escrow,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc();

    let makerMintABalanceAfter = await getAccount(connection, makerAtaA);

    assert.ok(
      makerMintABalanceAfter.amount ===
        vaultBalanceBefore.amount + makerMintABalanceBefore.amount
    );
  });

  it("Create Escrow again", async () => {
    await program.methods
      .initialize(escrowId, depositAmount, receiveAmount)
      .accountsPartial({
        maker: maker.publicKey,
        mintA,
        mintB,
        userAtaA: makerAtaA,
        vaultAtaA,
        escrow,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc();

    const vaultAccount = await getAccount(connection, vaultAtaA);

    assert.ok(vaultAccount.amount === BigInt(depositAmount.toString()));
  });

  it("Take offer", async () => {
    let makerMintABalanceBefore = await getAccount(connection, makerAtaA);
    let makerMintBBalanceBefore = await getAccount(connection, makerAtaB);
    let takerMintABalanceBefore = await getAccount(connection, takerAtaA);
    let takerMintBBalanceBefore = await getAccount(connection, takerAtaB);

    let vaultBalanceBefore = await getAccount(connection, vaultAtaA);

    // console.log("Taker A balance before:", takerMintABalanceBefore);
    // console.log("Vault A balance before:", vaultBalanceBefore);

    const escrowAccount = await program.account.escrow.fetch(escrow);

    // console.log("Maker", maker.publicKey);

    // console.log("Escrow state:", {
    //   maker: escrowAccount.maker.toString(),
    //   receiveAmountB: escrowAccount.receiveMintB.toString(),
    //   mintA: escrowAccount.mintA.toString(),
    //   mintB: escrowAccount.mintB.toString(),
    // });

    await program.methods
      .take()
      .accountsStrict({
        taker: taker.publicKey,
        maker: maker.publicKey,
        mintA,
        mintB,
        makerAtaB,
        takerAtaA,
        takerAtaB,
        vaultAtaA,
        escrow,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([taker])
      .rpc();

    let makerMintABalanceAfter = await getAccount(connection, makerAtaA);
    let makerMintBBalanceAfter = await getAccount(connection, makerAtaB);
    let takerMintABalanceAfter = await getAccount(connection, takerAtaA);
    let takerMintBBalanceAfter = await getAccount(connection, takerAtaB);

    // let vaultBalanceAfter = await getAccount(connection, vaultAtaA);

    // console.log(vaultBalanceAfter);

    // Check if taker balance was subtracted
    assert.ok(
      takerMintBBalanceAfter.amount ===
        BigInt(takerMintBMintAmount - receiveAmount.toNumber())
    );

    // Check if taker A balance was added
    assert.ok(takerMintABalanceAfter.amount === vaultBalanceBefore.amount);
  });
});
