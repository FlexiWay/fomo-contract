use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{errors::CustomErrors, Round};

#[derive(Accounts)]
pub struct RoundClaimContext<'info> {
    #[account(mut, constraint = round_account.authority == authority.key())]
    pub authority: Signer<'info>,

    #[account(mut,constraint = authority_ata.mint ==  token_mint.key(), constraint = authority_ata.owner == authority.key())]
    pub authority_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [b"round", round_account.seed.to_le_bytes().as_ref()],bump = round_account.bump )]
    pub round_account: Box<Account<'info, Round>>,

    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        address = round_account.team_vault.key()
    )]
    pub team_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl RoundClaimContext<'_> {
    pub fn validate(&self) -> Result<()> {
        let current_slot = Clock::get().unwrap().slot;
        // check if round is over or not

        require_gt!(
            current_slot,
            self.round_account.round_close_slot,
            CustomErrors::RoundNotOver
        );
        Ok(())
    }
    #[access_control(ctx.accounts.validate())]
    pub fn claim(ctx: Context<RoundClaimContext>) -> Result<()> {
        let round_account = &mut ctx.accounts.round_account;

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"round",
            &round_account.seed.to_le_bytes(),
            &[round_account.bump],
        ]];

        // Team Transfer Ix
        let transfer_instruction_team = anchor_spl::token::Transfer {
            from: ctx.accounts.team_vault.to_account_info(),
            to: ctx.accounts.authority_ata.to_account_info(),
            authority: round_account.to_account_info(),
        };

        let cpi_ctx_team = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_team,
        )
        .with_signer(signer_seeds);

        anchor_spl::token::transfer(cpi_ctx_team, ctx.accounts.team_vault.amount)?;
        Ok(())
    }
}
