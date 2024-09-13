import * as anchor from "@coral-xyz/anchor";
import { Program } from "@project-serum/anchor";
import { FomoContract } from "../target/types/fomo_contract";

describe("fomo-contract", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.FomoContract as Program<FomoContract>;

  it("Is initialized!", async () => {
    // Add your test here.

  });

});
