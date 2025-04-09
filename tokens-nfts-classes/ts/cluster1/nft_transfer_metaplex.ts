import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createSignerFromKeypair,
  signerIdentity,
  publicKey,
} from "@metaplex-foundation/umi";
import { mplTokenMetadata } from "@metaplex-foundation/mpl-token-metadata";
import { transferV1 } from "@metaplex-foundation/mpl-token-metadata";
import wallet from "../../tubin3-wallet.json";

// Create a UMI connection
const RPC_ENDPOINT =
  "https://turbine-solanad-4cde.devnet.rpcpool.com/168dd64f-ce5e-4e19-a836-f6482ad6b396";
const umi = createUmi(RPC_ENDPOINT);

// Create keypair from wallet
const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(signer));
umi.use(mplTokenMetadata());

const mintAddress = "9VfTmdyQVuqYFWckf6XyXyzZaTmvEBHLs2VoKjYvtdxo"; // Your NFT's mint address
const recipientAddress = "BvhV49WPYBbzPu8Fpy8YnPnwhNWLbm9Vmdj2T5bNSotS"; // Address to send the NFT to

(async () => {
  try {
    const mintPubkey = publicKey(mintAddress);
    const recipientPubkey = publicKey(recipientAddress);

    // Transfer the NFT
    const result = await transferV1(umi, {
      mint: mintPubkey,
      authority: signer,
      tokenOwner: signer.publicKey,
      destinationOwner: recipientPubkey,
      tokenStandard: 0, // NonFungible
    }).sendAndConfirm(umi);

    console.log(`Success! Check out your TX here: 
    https://explorer.solana.com/tx/${result.signature}?cluster=devnet`);
  } catch (e) {
    console.error(`Oops, something went wrong: ${e}`);
  }
})();
