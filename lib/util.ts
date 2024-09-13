
import * as anchor from "@project-serum/anchor";
import { 
    Connection,
    PublicKey
} from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
 
import {
    Metadata,
    PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID,
} from "@metaplex-foundation/mpl-token-metadata";
import { publicKey } from "@metaplex-foundation/umi";

export const METAPLEX = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
export const MPL_DEFAULT_RULE_SET = new PublicKey(
    "H6mX25exrJBXk86zGMX6Dd4WJoR6ZbjnzqUVT8d3NjAT"
);

export const getAssociatedTokenAccount = async (
    ownerPubkey: PublicKey, 
    mintPk: PublicKey
): Promise<PublicKey> => {
    let associatedTokenAccountPubkey = (PublicKey.findProgramAddressSync(
        [
            ownerPubkey.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            mintPk.toBuffer(), // mint address
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
    ))[0];

    return associatedTokenAccountPubkey;
}

export const getPDA = async (
    seeds: Array<Buffer | Uint8Array>,
    programId: PublicKey
  ) => {
    return PublicKey.findProgramAddressSync(seeds, programId);
  };
  