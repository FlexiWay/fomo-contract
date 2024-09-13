import { program } from 'commander';
import {
    PublicKey
} from '@solana/web3.js';

import { setClusterConfig, createRound, createVaults, createKey, burnKey, startRound } from './scripts';

program.version('0.0.1');

programCommand('create-round')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);

        await setClusterConfig(env, keypair, rpc)

        const txId = await createRound();

    });

programCommand('create-vaults')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);

        await setClusterConfig(env, keypair, rpc)

        const txId = await createVaults();

    });

programCommand('start-round')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);

        await setClusterConfig(env, keypair, rpc)

        const txId = await startRound();

    });

programCommand('create-key')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);

        await setClusterConfig(env, keypair, rpc)

        const txId = await createKey();

    });

programCommand('burn-key')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .option('-i, --index <string>', 'key')
    .option('-a, --address <string>', 'mint')
    .action(async (directory, cmd) => {
        const { env, keypair, rpc, index, address } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);

        await setClusterConfig(env, keypair, rpc)

        const txId = await burnKey(index, new PublicKey(address));

    });

function programCommand(name: string) {
    return program
        .command(name)
        .option('-e, --env <string>', 'Solana cluster env name', 'devnet') //mainnet-beta, testnet, devnet
        .option('-r, --rpc <string>', 'Solana cluster RPC name', 'https://devnet.helius-rpc.com/?api-key=9d3cf223-274e-43d8-9a62-003782aab591')
        .option('-k, --keypair <string>', 'Solana wallet Keypair Path', '/root/fomo-contract-wip/keys/test.json')
}

program.parse(process.argv);
