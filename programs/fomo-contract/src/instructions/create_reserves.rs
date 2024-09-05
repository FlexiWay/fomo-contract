use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::*;

#[derive(Accounts)]
pub struct CreateRoundReservesContext<'info> {
    #[account(mut, constraint = authority.key() == round_account.authority.key() )]
    pub authority: Signer<'info>,

    #[account(mut, seeds = [b"round", round_account.seed.to_le_bytes().as_ref()],bump = round_account.bump)]
    pub round_account: Box<Account<'info, Round>>,

    pub token_mint: Box<Account<'info, Mint>>,

    #[account(init,
        seeds = [b"pool", round_account.key().as_ref()],
        bump,
        payer = authority,
        token::mint = token_mint,
        token::authority = round_account,
    )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    #[account(init,
        seeds = [b"vault", round_account.key().as_ref()],
        bump,
        payer = authority,
        token::mint = token_mint,
        token::authority = round_account,
    )]
    pub treasure_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl CreateRoundReservesContext<'_> {
    pub fn create_reserves(ctx: Context<CreateRoundReservesContext>) -> Result<()> {
        let round_account = &mut ctx.accounts.round_account;

        round_account.create_reserves(RoundCreateReservesArgs {
            treasure_vault: ctx.accounts.treasure_vault.key(),
            pool_vault: ctx.accounts.pool_vault.key(),
        });

        Ok(())
    }
}
