import { web3 } from "@coral-xyz/anchor";
export const getPDA = (
  seeds: Array<Buffer | Uint8Array>,
  programId: web3.PublicKey
) => {
  return web3.PublicKey.findProgramAddressSync(seeds, programId);
};
