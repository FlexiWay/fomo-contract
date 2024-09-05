import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FomoContract } from "../target/types/fomo_contract";
import IDL from "../target/idl/fomo_contract.json";
import { getPDA } from "./utils";
import { BN } from "bn.js";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("fomo-contract", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const programId = new anchor.web3.PublicKey(
    "EqbUYzqofViyLNBRMpK1fJ1J3CJrgchK6xNh8Q2tsdqC"
  );
  const program = new Program(IDL as any) as unknown as Program<FomoContract>;
  const tokenMint = new anchor.web3.PublicKey(
    "266xrmsgpgHZ8nMNyx2xT5kRrpmDhQp7Zfn4QscFu31J"
  );

  const seed = Math.floor(Math.random() * 1000);

  it("Create Round", async () => {
    const [roundAccount] = getPDA(
      [Buffer.from("round"), new BN(seed).toArrayLike(Buffer, "le", 8)],
      programId
    );

    const tx = await program.methods
      .createRound(new BN(seed), {
        name: "test",
        uri: "",
        plugins: [],
        externalPluginAdapters: [],
      })
      .accounts({
        authority: program.provider.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        roundAccount: roundAccount,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });
  it("Create Round Reaserves", async () => {
    const [roundAccount] = getPDA(
      [Buffer.from("round"), new BN(seed).toArrayLike(Buffer, "le", 8)],
      programId
    );

    const [poolVault] = getPDA(
      [Buffer.from("pool"), roundAccount.toBuffer()],
      programId
    );
    const [teamVault] = getPDA(
      [Buffer.from("team"), roundAccount.toBuffer()],
      programId
    );
    const [treasureVault] = getPDA(
      [Buffer.from("vault"), roundAccount.toBuffer()],
      programId
    );
    const tx = await program.methods
      .createRoundReserves()
      .accounts({
        authority: program.provider.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        roundAccount: roundAccount,
        poolVault,
        teamVault,
        tokenMint,
        treasureVault,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });
  it("Create Key", async () => {
    const [roundAccount] = getPDA(
      [Buffer.from("round"), new BN(seed).toArrayLike(Buffer, "le", 8)],
      programId
    );
    const [keyAccount] = getPDA(
      [Buffer.from("key"), new BN(seed).toArrayLike(Buffer, "le", 8)],
      programId
    );
  });
});
