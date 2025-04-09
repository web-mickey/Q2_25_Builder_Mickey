import wallet from "../../tubin3-wallet.json";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createGenericFile,
  createSignerFromKeypair,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";

// Create a devnet connection
const umi = createUmi("https://api.devnet.solana.com");

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(irysUploader());
umi.use(signerIdentity(signer));

(async () => {
  try {
    const image =
      "https://devnet.irys.xyz/3LGzFz2v6hZtEi6tryDGADkSytVxLg8EjmwttbGScjGQ";
    const metadata = {
      name: "Rug Jeff",
      symbol: "RUG",
      uri: "https://arweave.net",
      description: "Turbin3 RUG DAY TDAY",
      image,
      attributes: [{ trait_type: "trait1", value: "legendary" }],
      properties: {
        files: [
          {
            type: "image/png",
            uri: image,
          },
        ],
      },
      creators: [],
    };

    const myUri = await umi.uploader.uploadJson(metadata);

    // const myUri = ???
    console.log("Your metadata URI: ", myUri);
  } catch (error) {
    console.log("Oops.. Something went wrong", error);
  }
})();
