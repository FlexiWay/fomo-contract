use anchor_lang::{ prelude::*, system_program };
use anchor_spl::token::{ Token, TokenAccount };
use mpl_core::{ instructions::{ CreateV2Cpi, CreateV2InstructionArgs }, types::DataState };

use crate::{ errors::FomoErrors, state::*, Config, SLOT_TO_CHANGE };

/// Context structure for the `create_key` instruction.
#[derive(Accounts)]
pub struct StartRoundContext<'info> {
    /// The authority who initiates the transaction.
    #[account(mut, constraint = authority.key() ==  round_account.authority.key())]
    pub authority: Signer<'info>,

    /// Signer representing the new asset to be created.
    #[account(mut)]
    pub asset: Signer<'info>,

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

    /// Vaults for mintfee, nftpool, and mainpool, respectively.
    #[account(mut, address = round_account.mint_fee_vault.key())]
    pub mint_fee_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = round_account.nft_pool_vault.key())]
    pub nft_pool_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = round_account.main_pool_vault.key())]
    pub main_pool_vault: Box<Account<'info, TokenAccount>>,

    /// Token program used for token transfers.
    pub token_program: Program<'info, Token>,

    /// Optional log wrapper program for SPL Noop.
    /// CHECK: Verified in mpl-core.
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// MPL Core program for managing assets.
    /// CHECK: Verified via the address constraint.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,

    /// System program for standard Solana operations.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl StartRoundContext<'_> {
    /// Validation function to ensure the round has not ended.
    pub fn validate(&self) -> Result<()> {
        require_eq!(self.round_account.mint_counter, 0, FomoErrors::RoundStarted);

        Ok(())
    }

    /// Instruction to create a new key.
    #[access_control(ctx.accounts.validate())]
    pub fn start_round(ctx: Context<StartRoundContext>) -> Result<()> {
        let key_account = &mut ctx.accounts.key_account;
        let round_account = &mut ctx.accounts.round_account;

        let current_counter = 1;
        let current_slot = Clock::get()?.slot;

        round_account.mint_counter = 1;
        round_account.round_close_slot = current_slot + SLOT_TO_CHANGE;

        let key_bump = *ctx.bumps.get("key_account").unwrap();

        // Initialize the new key account.
        key_account.create(CreateKeyArgs {
            nft_mint: ctx.accounts.asset.key(),
            bump: key_bump,
            key_index: current_counter,
        });

        // Process asset creation and plugin update for the new key.
        Self::process_asset_creation(&ctx)?;

        Ok(())
    }

    /// Helper function to process asset creation and plugin update.
    fn process_asset_creation(ctx: &Context<StartRoundContext>) -> Result<()> {
        let round_account = &ctx.accounts.round_account;
        let seeds: &[&[&[u8]]] = &[
            &[b"round", &round_account.seed.to_le_bytes(), &[round_account.bump]],
        ];

        // Create a new asset.
        let config = Config::get_master(round_account.key());

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
                name: config.name,
                uri: config.uri,
                data_state: DataState::AccountState,
                plugins: Some(config.plugins),
                external_plugin_adapters: None,
            },
        }).invoke_signed(seeds)?;

        Ok(())
    }
}
