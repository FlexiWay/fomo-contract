use anchor_lang::{ prelude::*, system_program };
use anchor_spl::token::{ Mint, Token, TokenAccount };
use crate::state::*;

#[derive(Accounts)]
pub struct CreateRoundVaultsContext<'info> {
    /// Authority account must match the `authority` field in the `round_account`.
    #[account(mut, constraint = authority.key() == round_account.authority.key())]
    pub authority: Signer<'info>,

    /// The `round_account` storing all necessary round data, derived using a seed and bump.
    #[account(mut, seeds = [b"round", round_account.seed.to_le_bytes().as_ref()], bump = round_account.bump)]
    pub round_account: Box<Account<'info, Round>>,

    /// The token mint for the vaults to be created.
    pub token_mint: Box<Account<'info, Mint>>,

    /// Vault for mint fee tokens, initialized and owned by the `round_account`.
    #[account(
        init,
        seeds = [b"mint_fee", round_account.key().as_ref()],
        bump,
        payer = authority,
        token::mint = token_mint,
        token::authority = round_account
    )]
    pub mint_fee_vault: Box<Account<'info, TokenAccount>>,

    /// Vault for NFTs, initialized and owned by the `round_account`.
    #[account(
        init,
        seeds = [b"nft_pool", round_account.key().as_ref()],
        bump,
        payer = authority,
        token::mint = token_mint,
        token::authority = round_account
    )]
    pub nft_pool_vault: Box<Account<'info, TokenAccount>>,

    /// Main vault for tokens, initialized and owned by the `round_account`.
    #[account(
        init,
        seeds = [b"main_pool", round_account.key().as_ref()],
        bump,
        payer = authority,
        token::mint = token_mint,
        token::authority = round_account
    )]
    pub main_pool_vault: Box<Account<'info, TokenAccount>>,

    /// Token program required for token-related operations.
    pub token_program: Program<'info, Token>,

    /// System program, required for initializing accounts and managing lamports.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl CreateRoundVaultsContext<'_> {
    pub fn create_vaults(ctx: Context<CreateRoundVaultsContext>) -> Result<()> {
        let round_account = &mut ctx.accounts.round_account;

        round_account.create_vaults(RoundCreateVaultsArgs {
            mint_fee_vault: ctx.accounts.mint_fee_vault.key(),
            nft_pool_vault: ctx.accounts.nft_pool_vault.key(),
            main_pool_vault: ctx.accounts.main_pool_vault.key(),
        });

        Ok(())
    }
}
