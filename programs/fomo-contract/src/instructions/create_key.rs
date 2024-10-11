use anchor_lang::{ prelude::*, system_program };
use anchor_lang::solana_program::program::invoke;
use anchor_spl::{ token::{ Mint, Token, TokenAccount }, associated_token::AssociatedToken };
use spl_token::instruction as spl_instruction;
use mpl_core::{
    instructions::{
        CreateV2Cpi,
        CreateV2InstructionArgs,
        UpdateV2Cpi,
        UpdateV2InstructionArgs,
        UpdatePluginV1Cpi,
        UpdatePluginV1InstructionArgs,
    },
    types::{ DataState, FreezeDelegate, Plugin },
};
use crate::{
    errors::FomoErrors,
    state::*,
    Config,
    MAIN_FEE_BASIS_POINTS,
    NFT_FEE_BASIS_POINTS,
    MINT_FEE_BASIS_POINTS,
    TIME_TO_CHANGE,
};

/// Context structure for the `create_key` instruction.
#[derive(Accounts)]
pub struct CreateKeyContext<'info> {
    /// The authority who initiates the transaction.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Signer representing the new asset to be created.
    #[account(mut)]
    pub asset: Signer<'info>,

    /// Existing asset account; should match the `nft_mint` of the current key account.
    /// CHECK: Verified via the address constraint.
    #[account(mut, address = current_key_account.nft_mint)]
    pub current_asset: AccountInfo<'info>,

    /// Round account which tracks the minting round information.
    #[account(mut, seeds = [b"round", round_account.seed.to_le_bytes().as_ref()], bump = round_account.bump)]
    pub round_account: Box<Account<'info, Round>>,

    /// Collection to which the asset belongs.
    /// CHECK: Verified in mpl-core.
    #[account(mut, address = round_account.collection.key())]
    pub collection: AccountInfo<'info>,

    /// The account representing the newly created key.
    #[account(
        init,
        payer = authority,
        space = 8 + NftKey::INIT_SPACE,
        seeds = [
            b"key",
            round_account.key().as_ref(),
            (round_account.mint_counter + 1).to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub key_account: Box<Account<'info, NftKey>>,

    /// The current key account for the asset.
    #[account(mut, seeds = [b"key", round_account.key().as_ref(), round_account.mint_counter.to_le_bytes().as_ref()], bump = current_key_account.bump)]
    pub current_key_account: Box<Account<'info, NftKey>>,

    /// Vaults for mintfee, nftpool, and mainpool, respectively.
    #[account(mut, address = round_account.mint_fee_vault.key())]
    pub mint_fee_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = round_account.nft_pool_vault.key())]
    pub nft_pool_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = round_account.main_pool_vault.key())]
    pub main_pool_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: Pool account (PDA)
    pub pool: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Vault account for token a. token a of the pool will be deposit / withdraw from this vault account.
    pub a_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Vault account for token b. token b of the pool will be deposit / withdraw from this vault account.
    pub b_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Token vault account of vault A
    pub a_token_vault: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Token vault account of vault B
    pub b_token_vault: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Lp token mint of vault a
    pub a_vault_lp_mint: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Lp token mint of vault b
    pub b_vault_lp_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: LP token account of vault A. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    pub a_vault_lp: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: LP token account of vault B. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    pub b_vault_lp: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Admin fee token account. Used to receive trading fee. It's mint field must matched with user_source_token mint field.
    pub admin_token_fee: UncheckedAccount<'info>,

    /// Authority's token account associated with the token mint.
    /// If the account does not exist, it will be initialized.
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = wsol_mint,
        associated_token::authority = authority
    )]
    pub user_source_token: Box<Account<'info, TokenAccount>>,

    /// Mint of the tokens associated with the round.
    pub wsol_mint: Box<Account<'info, Mint>>,

    /// CHECK: Vault program. the pool will deposit/withdraw liquidity from the vault.
    pub vault_program: UncheckedAccount<'info>,

    /// Token program used for token transfers.
    pub token_program: Program<'info, Token>,

    /// Associated Token Program for creating associated token accounts.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Optional log wrapper program for SPL Noop.
    /// CHECK: Verified in mpl-core.
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// MPL Core program for managing assets.
    /// CHECK: Verified via the address constraint.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,

    #[account(address = dynamic_amm::ID)]
    /// CHECK: Dynamic AMM program account
    pub dynamic_amm_program: UncheckedAccount<'info>,

    /// System program for standard Solana operations.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl CreateKeyContext<'_> {
    /// Validation function to ensure the round has not ended.
    pub fn validate(&self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp as u64;

        // Ensure the round is still active by comparing current slot with the round close slot.
        require_gt!(self.round_account.round_close_timestamp, current_time, FomoErrors::RoundOver);

        Ok(())
    }

    /// Instruction to create a new key.
    #[access_control(ctx.accounts.validate())]
    pub fn create_key(ctx: Context<CreateKeyContext>) -> Result<()> {
        let key_account = &mut ctx.accounts.key_account;
        let round_account = &mut ctx.accounts.round_account;
        let current_counter = round_account.mint_counter + 1;
        let current_timestamp = Clock::get()?.unix_timestamp as u64;

        let key_bump = *ctx.bumps.get("key_account").unwrap();

        // Initialize the new key account.
        key_account.create(CreateKeyArgs {
            nft_mint: ctx.accounts.asset.key(),
            bump: key_bump,
            key_index: current_counter,
        });

        // Calculate the total amount of tokens to be used based on the key index.
        let total_amount_for_index = 10_000_000 + round_account.round_increment * current_counter;

        // Update the round account to reflect the new key.
        round_account.mint_counter = current_counter;
        round_account.round_close_timestamp = current_timestamp + TIME_TO_CHANGE;
        msg!("herere1");

        // SOL trasfer to Wrap SOL account
        let sol_ix = solana_program::system_instruction::transfer(
            &ctx.accounts.authority.to_account_info().key,
            &ctx.accounts.user_source_token.to_account_info().key,
            total_amount_for_index
        );

        invoke(
            &sol_ix,
            &[
                ctx.accounts.authority.to_account_info().clone(),
                ctx.accounts.user_source_token.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info(),
            ]
        )?;

        let wrap_ix = spl_instruction::sync_native(
            &spl_token::id(),
            &ctx.accounts.user_source_token.key()
        )?;

        invoke(&wrap_ix, &[ctx.accounts.user_source_token.to_account_info().clone()])?;

        // Perform token transfers for various fees (Burn, Team, Pool, Treasure).
        Self::process_fee_transfers(&ctx, total_amount_for_index)?;

        // Process asset creation and plugin update for the new key.
        Self::process_asset_creation(&ctx)?;

        Ok(())
    }

    /// Helper function to process fee-related token transfers.
    fn process_fee_transfers(ctx: &Context<CreateKeyContext>, total_amount: u64) -> Result<()> {
        let fee_bps = [
            (MAIN_FEE_BASIS_POINTS, &ctx.accounts.main_pool_vault),
            (NFT_FEE_BASIS_POINTS, &ctx.accounts.nft_pool_vault),
            (MINT_FEE_BASIS_POINTS, &ctx.accounts.mint_fee_vault),
        ];

        for (bps, vault) in fee_bps.iter() {
            let amount = total_amount.checked_mul(*bps).unwrap().checked_div(10_000).unwrap();

            let accounts = dynamic_amm::cpi::accounts::Swap {
                pool: ctx.accounts.pool.to_account_info(),
                user_source_token: ctx.accounts.user_source_token.to_account_info(),
                user_destination_token: vault.to_account_info(),
                a_vault: ctx.accounts.a_vault.to_account_info(),
                b_vault: ctx.accounts.b_vault.to_account_info(),
                a_token_vault: ctx.accounts.a_token_vault.to_account_info(),
                b_token_vault: ctx.accounts.b_token_vault.to_account_info(),
                a_vault_lp_mint: ctx.accounts.a_vault_lp_mint.to_account_info(),
                b_vault_lp_mint: ctx.accounts.b_vault_lp_mint.to_account_info(),
                a_vault_lp: ctx.accounts.a_vault_lp.to_account_info(),
                b_vault_lp: ctx.accounts.b_vault_lp.to_account_info(),
                admin_token_fee: ctx.accounts.admin_token_fee.to_account_info(),
                user: ctx.accounts.authority.to_account_info(),
                vault_program: ctx.accounts.vault_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };

            let cpi_context = CpiContext::new(
                ctx.accounts.dynamic_amm_program.to_account_info(),
                accounts
            );

            let _ = dynamic_amm::cpi::swap(cpi_context, amount, 0);
        }

        Ok(())
    }

    /// Helper function to process asset creation and plugin update.
    fn process_asset_creation(ctx: &Context<CreateKeyContext>) -> Result<()> {
        let round_account = &ctx.accounts.round_account;
        let seeds: &[&[&[u8]]] = &[
            &[b"round", &round_account.seed.to_le_bytes(), &[round_account.bump]],
        ];

        // Create a new asset.
        let config_default = Config::get_default(round_account.key());
        let config_master = Config::get_master(round_account.key());

        (CreateV2Cpi {
            asset: &ctx.accounts.asset.to_account_info(),
            collection: Some(ctx.accounts.collection.as_ref()),
            authority: Some(round_account.to_account_info().as_ref()),
            payer: &ctx.accounts.authority.to_account_info(),
            owner: Some(ctx.accounts.authority.as_ref()),
            update_authority: None,
            system_program: &ctx.accounts.system_program.to_account_info(),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            __program: &ctx.accounts.mpl_core,
            __args: CreateV2InstructionArgs {
                name: config_master.name,
                uri: config_master.uri,
                data_state: DataState::AccountState,
                plugins: Some(config_master.plugins),
                external_plugin_adapters: None,
            },
        }).invoke_signed(seeds)?;

        (UpdateV2Cpi {
            asset: &ctx.accounts.current_asset,
            collection: Some(ctx.accounts.collection.as_ref()),
            authority: Some(round_account.to_account_info().as_ref()),
            payer: &ctx.accounts.authority.to_account_info(),
            system_program: &ctx.accounts.system_program.to_account_info(),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            __program: &ctx.accounts.mpl_core,
            __args: UpdateV2InstructionArgs {
                new_name: Some(config_default.name),
                new_uri: Some(config_default.uri),
                new_update_authority: None,
            },
            new_collection: None,
        }).invoke_signed(seeds)?;

        // Update the plugin for the current asset.
        (UpdatePluginV1Cpi {
            asset: &ctx.accounts.current_asset,
            collection: Some(ctx.accounts.collection.as_ref()),
            authority: Some(round_account.to_account_info().as_ref()),
            payer: &ctx.accounts.authority.to_account_info(),
            system_program: &ctx.accounts.system_program.to_account_info(),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            __program: &ctx.accounts.mpl_core,
            __args: UpdatePluginV1InstructionArgs {
                plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
            },
        }).invoke_signed(seeds)?;

        Ok(())
    }
}
