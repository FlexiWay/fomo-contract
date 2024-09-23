import { Program, web3 } from '@project-serum/anchor';
import * as anchor from '@project-serum/anchor';
import fs from 'fs';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { FOMO_PROGRAM_ID, ROUND_SEED, MPL_CORE, TOKEN_MINT, SPL_NOOP_PROGRAM_ID, POOL_ADDRESS, DYNAMIC_VAULT_PROGRAM_ID } from '../lib/constant';
import { Connection, Enum, Keypair, ComputeBudgetProgram, Transaction, PublicKey, SYSVAR_CLOCK_PUBKEY } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, NATIVE_MINT } from '@solana/spl-token';
import { IDL } from "../target/types/fomo_contract";
import { BN } from "bn.js";
import {
    getPDA,
    getAssociatedTokenAccount
} from '../lib/util';
import { Round, Key } from '../lib/types';
import AmmImpl, { PROGRAM_ID } from '@mercurial-finance/dynamic-amm-sdk';

let solConnection: web3.Connection = null;
let program: Program = null;
let provider: anchor.Provider = null;
let payer: NodeWallet = null;

// Address of the deployed program.
let programId = new anchor.web3.PublicKey(FOMO_PROGRAM_ID);

/**
 * Set cluster, provider, program
 * If rpc != null use rpc, otherwise use cluster param
 * @param cluster - cluster ex. mainnet-beta, devnet ...
 * @param keypair - wallet keypair
 * @param rpc - rpc
 */
export const setClusterConfig = async (
    cluster: web3.Cluster,
    keypair: string, rpc?: string
) => {

    if (!rpc) {
        solConnection = new web3.Connection(web3.clusterApiUrl(cluster));
    } else {
        solConnection = new web3.Connection(rpc);
    }

    const walletKeypair = web3.Keypair.fromSecretKey(
        Uint8Array.from(JSON.parse(fs.readFileSync(keypair, 'utf-8'))),
        { skipValidation: true });

    const wallet = new NodeWallet(walletKeypair);

    // Configure the client to use the local cluster.
    anchor.setProvider(new anchor.AnchorProvider(
        solConnection,
        wallet,
        { skipPreflight: true, commitment: 'confirmed' }));
    payer = wallet;

    provider = anchor.getProvider();
    console.log('Wallet Address: ', wallet.publicKey.toBase58());

    // Generate the program client from IDL.
    program = new anchor.Program(IDL as anchor.Idl, programId);
    console.log('ProgramId: ', program.programId.toBase58());
}

export const getRoundState = async (program: anchor.Program): Promise<Round | null> => {

    const [roundAccount] = await getPDA(
        [Buffer.from("round"), new BN(ROUND_SEED).toArrayLike(Buffer, "le", 8)],
        programId
    );

    try {
        let roundState = await program.account.round.fetch(roundAccount, "confirmed") as unknown as Round;
        return roundState;
    } catch (error) {
        return null
    }
}

export const createRound = async () => {

    const [roundAccount] = await getPDA(
        [Buffer.from("round"), new BN(ROUND_SEED).toArrayLike(Buffer, "le", 8)],
        programId
    );

    const newCollection = Keypair.generate();

    const tx = await program.methods
        .createRound(
            new BN(ROUND_SEED),
            "fomo-test",
            ""
        )
        .accounts({
            authority: payer.publicKey,
            collection: newCollection.publicKey,
            roundAccount,
            mplCore: MPL_CORE,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId
        })
        .transaction();

    const txId = await provider.sendAndConfirm(tx, [newCollection], {
        commitment: "confirmed",
    });

    console.log("txHash: ", txId);

    return txId;
}

export const createVaults = async () => {

    const [roundAccount] = await getPDA(
        [Buffer.from("round"), new BN(ROUND_SEED).toArrayLike(Buffer, "le", 8)],
        programId
    );

    const [mintFeeVault] = await getPDA(
        [Buffer.from("mint_fee"), roundAccount.toBuffer()], programId
    );

    const [nftPoolVault] = await getPDA(
        [Buffer.from("nft_pool"), roundAccount.toBuffer()], programId
    );
    const [mainPoolVault] = await getPDA(
        [Buffer.from("main_pool"), roundAccount.toBuffer()], programId
    );

    console.log('mintFeeVault-->>>>', mintFeeVault.toBase58());
    console.log('nftPoolVault-->>>>', nftPoolVault.toBase58());
    console.log('mainPoolVault-->>>>', mainPoolVault.toBase58());

    const tx = await program.methods
        .createVaults()
        .accounts({
            authority: payer.publicKey,
            roundAccount,
            tokenMint: TOKEN_MINT,
            mintFeeVault,
            nftPoolVault,
            mainPoolVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId
        })
        .transaction();

    const txId = await provider.sendAndConfirm(tx, [], {
        commitment: "confirmed",
    });

    console.log("txHash: ", txId);

    return txId;
}

export const startRound = async () => {

    const [roundAccount] = await getPDA(
        [Buffer.from("round"), new BN(ROUND_SEED).toArrayLike(Buffer, "le", 8)],
        programId
    );

    const newMint = Keypair.generate();

    const roundState = await getRoundState(program);

    const [keyAccount] = await getPDA(
        [Buffer.from("key"), roundAccount.toBuffer(), new BN(Number(roundState.mintCounter) + 1).toArrayLike(Buffer, "le", 8)],
        programId
    );

    const tx = await program.methods
        .startRound()
        .accounts({
            authority: payer.publicKey,
            asset: newMint.publicKey,
            roundAccount,
            collection: roundState.collection,
            keyAccount,
            mintFeeVault: roundState.mintFeeVault,
            nftPoolVault: roundState.nftPoolVault,
            mainPoolVault: roundState.mainPoolVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            logWrapper: SPL_NOOP_PROGRAM_ID,
            mplCore: MPL_CORE,
            systemProgram: anchor.web3.SystemProgram.programId
        })
        // .preInstructions(createIx.instructions)
        .transaction();

    const txId = await provider.sendAndConfirm(tx, [newMint], {
        commitment: "confirmed",
    });

    console.log("txHash: ", txId);

    return txId;
}

export const createKey = async () => {

    const [roundAccount] = await getPDA(
        [Buffer.from("round"), new BN(ROUND_SEED).toArrayLike(Buffer, "le", 8)],
        programId
    );

    const tokenAccount = await getAssociatedTokenAccount(payer.publicKey, NATIVE_MINT);

    console.log('tokenAccount--->>>>', tokenAccount.toBase58());

    const newMint = Keypair.generate();

    const roundState = await getRoundState(program);
    console.log('roundState---->>>>>', roundState);

    const [currentKeyAccount] = await getPDA(
        [Buffer.from("key"), roundAccount.toBuffer(), new BN(Number(roundState.mintCounter)).toArrayLike(Buffer, "le", 8)],
        programId
    );
    console.log('currentKeyAccount--->>>>', currentKeyAccount.toBase58());

    const [keyAccount] = await getPDA(
        [Buffer.from("key"), roundAccount.toBuffer(), new BN(Number(roundState.mintCounter) + 1).toArrayLike(Buffer, "le", 8)],
        programId
    );
    console.log('keyAccount--->>>>', keyAccount.toBase58());
    console.log('mintCounter--->>>>', Number(Number(roundState.mintCounter)));

    let currentAsset = await program.account.nftKey.fetch(currentKeyAccount, "confirmed") as unknown as Key;
    console.log('currentAssetMint--->>>>', currentAsset.nftMint.toBase58());
    console.log('currentAsset--->>>>', Number(currentAsset.keyIndex));

    const pool: any = await AmmImpl.create(provider.connection, POOL_ADDRESS);

    const poolState = pool.poolState;

    const remainingAccounts = pool.swapCurve.getRemainingAccounts();

    const tx = await program.methods
        .createKey()
        .accounts({
            authority: payer.publicKey,
            asset: newMint.publicKey,
            currentAsset: currentAsset.nftMint,
            roundAccount,
            collection: roundState.collection,
            keyAccount,
            currentKeyAccount,
            mintFeeVault: roundState.mintFeeVault,
            nftPoolVault: roundState.nftPoolVault,
            mainPoolVault: roundState.mainPoolVault,
            pool: POOL_ADDRESS,
            aVault: poolState.aVault,
            bVault: poolState.bVault,
            aTokenVault: pool.vaultA.tokenVaultPda,
            bTokenVault: pool.vaultB.tokenVaultPda,
            aVaultLpMint: pool.vaultA.vaultState.lpMint,
            bVaultLpMint: pool.vaultB.vaultState.lpMint,
            aVaultLp: poolState.aVaultLp,
            bVaultLp: poolState.bVaultLp,
            adminTokenFee: poolState.protocolTokenBFee,
            userSourceToken: tokenAccount,
            wsolMint: NATIVE_MINT,
            vaultProgram: DYNAMIC_VAULT_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            logWrapper: SPL_NOOP_PROGRAM_ID,
            mplCore: MPL_CORE,
            dynamicAmmProgram: PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId
        })
        .remainingAccounts(remainingAccounts)
        .transaction();

    tx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 2_000_000 }));

    const txId = await provider.sendAndConfirm(tx, [newMint], {
        commitment: "confirmed",
    });

    console.log("txHash: ", txId);

    return txId;
}

export const burnKey = async (index: number, address: PublicKey) => {

    const [roundAccount] = await getPDA(
        [Buffer.from("round"), new BN(ROUND_SEED).toArrayLike(Buffer, "le", 8)],
        programId
    );

    const tokenAccount = await getAssociatedTokenAccount(payer.publicKey, TOKEN_MINT);

    console.log('tokenAccount--->>>>', tokenAccount.toBase58());

    const roundState = await getRoundState(program);
    console.log('roundstate--->>>', roundState)

    const [keyAccount] = await getPDA(
        [Buffer.from("key"), roundAccount.toBuffer(), new BN(index).toArrayLike(Buffer, "le", 8)],
        programId
    );
    console.log('keyAccount--->>>>', keyAccount.toBase58());

    const tx = await program.methods
        .burnKey()
        .accounts({
            authority: payer.publicKey,
            authorityAta: tokenAccount,
            roundAccount,
            collection: roundState.collection,
            asset: address,
            keyAccount,
            nftPoolVault: roundState.nftPoolVault,
            tokenMint: TOKEN_MINT,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            logWrapper: SPL_NOOP_PROGRAM_ID,
            mplCore: MPL_CORE,
            systemProgram: anchor.web3.SystemProgram.programId
        })
        .transaction();

    const txId = await provider.sendAndConfirm(tx, [], {
        commitment: "confirmed",
    });

    console.log("txHash: ", txId);

    return txId;
}

export const feeClaim = async () => {

    const [roundAccount] = await getPDA(
        [Buffer.from("round"), new BN(ROUND_SEED).toArrayLike(Buffer, "le", 8)],
        programId
    );

    const tokenAccount = await getAssociatedTokenAccount(payer.publicKey, TOKEN_MINT);

    console.log('tokenAccount--->>>>', tokenAccount.toBase58());

    const roundState = await getRoundState(program);
    console.log('roundstate--->>>', roundState)

    const tx = await program.methods
        .feeClaim()
        .accounts({
            authority: payer.publicKey,
            authorityAta: tokenAccount,
            roundAccount,
            tokenMint: TOKEN_MINT,
            mintFeeVault: roundState.mintFeeVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId
        })
        .transaction();

    const txId = await provider.sendAndConfirm(tx, [], {
        commitment: "confirmed",
    });

    console.log("txHash: ", txId);

    return txId;
}

export const updateRound = async (increment_amount: number) => {

    const [roundAccount] = await getPDA(
        [Buffer.from("round"), new BN(ROUND_SEED).toArrayLike(Buffer, "le", 8)],
        programId
    );

    const tx = await program.methods
        .updateRound(increment_amount)
        .accounts({
            authority: payer.publicKey,
            roundAccount
        })
        .transaction();

    const txId = await provider.sendAndConfirm(tx, [], {
        commitment: "confirmed",
    });

    console.log("txHash: ", txId);

    return txId;
}

export const winnerClaim = async (address: PublicKey) => {

    const [roundAccount] = await getPDA(
        [Buffer.from("round"), new BN(ROUND_SEED).toArrayLike(Buffer, "le", 8)],
        programId
    );

    const roundState = await getRoundState(program);
    console.log('roundstate--->>>', roundState)

    const tokenAccount = await getAssociatedTokenAccount(address, TOKEN_MINT);

    console.log('tokenAccount--->>>>', tokenAccount.toBase58());

    const [keyAccount] = await getPDA(
        [Buffer.from("key"), roundAccount.toBuffer(), new BN(roundState.mintCounter).toArrayLike(Buffer, "le", 8)],
        programId
    );

    let currentAsset = await program.account.nftKey.fetch(keyAccount, "confirmed") as unknown as Key;
    console.log('currentAssetMint--->>>>', currentAsset.nftMint.toBase58());
    console.log('currentAsset--->>>>', Number(currentAsset.keyIndex));

    const tx = await program.methods
        .winnerClaim()
        .accounts({
            authority: payer.publicKey,
            winner: address,
            winnerAta: tokenAccount,
            roundAccount,
            asset: currentAsset.nftMint,
            keyAccount,
            tokenMint: TOKEN_MINT,
            mainPoolVault: roundState.mainPoolVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId
        })
        .transaction();

    const txId = await provider.sendAndConfirm(tx, [], {
        commitment: "confirmed",
    });

    console.log("txHash: ", txId);

    return txId;
}






