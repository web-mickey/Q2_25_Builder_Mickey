import {
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair } from "@solana/web3.js";
import BN from "bn.js";
it("test", async () => {
  const provider = anchor.AnchorProvider.env();

  anchor.setProvider(provider);

  const connection = provider.connection;

  // const confirm = async (signature:string): Promise<string>=> {
  //     const block = await connection.getLatestBlockhash();
  //     await connection.confirmTransaction({
  // }

  const tokenProgram = TOKEN_2022_PROGRAM_ID;

  const [maker, taker, mintA, mintB] = Array.from({ length: 4 }, () =>
    Keypair.generate()
  );

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  const [makerAtaA, makerAtaB, takerAtaA, takerAtaB] = [maker, taker].map((a) =>
    [mintA, mintB].map((m) => {
      getAssociatedTokenAddressSync(
        m.publicKey,
        a.publicKey,
        false,
        tokenProgram
      );
    })
  );

  const seed = new BN(randomBytes(8));

  const escrow = PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      maker.publicKey.toBuffer(),
      seed.toArrayLike(Buffer, "le", 8),
    ],
    programId
  );
});
