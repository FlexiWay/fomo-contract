use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_core::{
    accounts::BaseAssetV1,
    instructions::{BurnV1Cpi, BurnV1InstructionArgs},
    types::UpdateAuthority,
};

use crate::{errors::CustomErrors, state::*};

#[derive(Accounts)]
pub struct BurnKeyContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut,constraint = authority_ata.mint ==  token_mint.key(), constraint = authority_ata.owner == authority.key())]
    pub authority_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        seeds = [b"round", round_account.seed.to_le_bytes().as_ref()],
        bump = round_account.bump
    )]
    pub round_account: Box<Account<'info, Round>>,

    /// CHECK: address check added
    #[account(address = round_account.collection.key())]
    pub collection: AccountInfo<'info>,

    /// CHECK: checking later
    #[account(mut, constraint = asset.key() == key_account.nft_mint.key())]
    pub asset: UncheckedAccount<'info>,

    #[account(mut,
        seeds = [b"key",round_account.key().as_ref(),key_account.key_index.to_le_bytes().as_ref()],
        bump = key_account.bump
    )]
    pub key_account: Box<Account<'info, NftKey>>,

    pub token_mint: Box<Account<'info, Mint>>,

    #[account( address = round_account.pool_vault.key() )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    /// The SPL Noop program.
    /// CHECK: Checked in mpl-core.
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// The MPL Core program.
    /// CHECK: Checked in mpl-core.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl BurnKeyContext<'_> {
    pub fn validate(&self) -> Result<()> {
        // Not needed:  check if round is over
        // let current_slot = Clock::get().unwrap().slot;

        // require_gt!(
        //     self.round_account.round_close_slot,
        //     current_slot,
        //     CustomErrors::RoundOver
        // );

        let asset_data = BaseAssetV1::from_bytes(&self.asset.to_account_info().data.borrow())?;

        if asset_data.update_authority != UpdateAuthority::Collection(self.round_account.collection)
        {
            return Err(CustomErrors::InvalidKeyAccount.into());
        }
        Ok(())
    }

    #[access_control(ctx.accounts.validate())]
    pub fn burn(ctx: Context<BurnKeyContext>) -> Result<()> {
        let key_account = &mut ctx.accounts.key_account;
        let round_account = &mut ctx.accounts.round_account;
        let current_holder_counter = round_account
            .mint_counter
            .checked_sub(round_account.nft_burn_counter)
            .unwrap();

        require_neq!(key_account.exited, 1, CustomErrors::InvalidKeyAccount);

        let token_in_pool = ctx.accounts.pool_vault.amount;
        let avg_amount_pool = token_in_pool.checked_div(current_holder_counter).unwrap();

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"round",
            &round_account.seed.to_le_bytes(),
            &[round_account.bump],
        ]];

        let transfer_instruction_burn = anchor_spl::token::Transfer {
            from: ctx.accounts.pool_vault.to_account_info(),
            to: ctx.accounts.authority_ata.to_account_info(),
            authority: round_account.to_account_info(),
        };

        let cpi_ctx_burn = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_burn,
        )
        .with_signer(signer_seeds);

        anchor_spl::token::transfer(cpi_ctx_burn, avg_amount_pool)?;

        round_account.nft_burn_counter += 1;

        key_account.exited = 1;

        BurnV1Cpi {
            system_program: Some(&ctx.accounts.system_program.to_account_info()),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            collection: Some(&ctx.accounts.collection),
            asset: &ctx.accounts.asset.to_account_info(),
            payer: &ctx.accounts.authority.to_account_info(),
            __program: &ctx.accounts.mpl_core,
            authority: Some(&ctx.accounts.authority.to_account_info()),
            __args: BurnV1InstructionArgs {
                compression_proof: None,
            },
        }
        .invoke()?;
        Ok(())
    }
}
