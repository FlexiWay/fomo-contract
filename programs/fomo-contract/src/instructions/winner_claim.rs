use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_core::{
    accounts::BaseAssetV1,
    fetch_plugin,
    types::{UpdateAuthority, VerifiedCreators},
};

use crate::{errors::CustomErrors, NftKey, Round};

#[derive(Accounts)]
pub struct WinnerClaimContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut,constraint = authority_ata.mint ==  token_mint.key(), constraint = authority_ata.owner == authority.key())]
    pub authority_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [b"round", round_account.seed.to_le_bytes().as_ref()],bump = round_account.bump )]
    pub round_account: Box<Account<'info, Round>>,

    /// CHECK: checking later
    #[account(mut, constraint = asset.key() == key_account.nft_mint.key())]
    pub asset: UncheckedAccount<'info>,

    #[account(mut,
        constraint =  key_account.key_index == round_account.mint_counter,
        seeds = [b"key",round_account.key().as_ref(),key_account.key_index.to_le_bytes().as_ref()],
        bump = key_account.bump
    )]
    pub key_account: Box<Account<'info, NftKey>>,

    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
           address = round_account.treasure_vault.key()
    )]
    pub treasure_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl WinnerClaimContext<'_> {
    pub fn validate(&self) -> Result<()> {
        let current_slot = Clock::get().unwrap().slot;
        // check if round is over or not
        require_gt!(
            current_slot,
            self.round_account.round_close_slot,
            CustomErrors::RoundOver
        );

        let asset_data = BaseAssetV1::from_bytes(&self.asset.to_account_info().data.borrow())?;

        if asset_data.update_authority != UpdateAuthority::Collection(self.round_account.collection)
        {
            return Err(CustomErrors::InvalidAsset.into());
        }

        // check for verified creators
        let attributes_plugin = fetch_plugin::<BaseAssetV1, VerifiedCreators>(
            &self.asset.to_account_info(),
            mpl_core::types::PluginType::VerifiedCreators,
        )
        .unwrap();

        let is_verified: Option<bool> = match attributes_plugin
            .1
            .signatures
            .iter()
            .find(|sig| sig.verified && sig.address == self.round_account.key())
        {
            Some(_) => Some(true),
            None => Some(false),
        };

        match is_verified {
            Some(true) => {
                // Proceed with the rest of your code
            }
            _ => {
                return Err(CustomErrors::InvalidAsset.into());
            }
        }
        Ok(())
    }

    #[access_control(ctx.accounts.validate())]
    pub fn winner_claim(ctx: Context<WinnerClaimContext>) -> Result<()> {
        let round_account = &mut ctx.accounts.round_account;

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"round",
            &round_account.seed.to_le_bytes(),
            &[round_account.bump],
        ]];

        let transfer_instruction_burn = anchor_spl::token::Transfer {
            from: ctx.accounts.treasure_vault.to_account_info(),
            to: ctx.accounts.authority_ata.to_account_info(),
            authority: round_account.to_account_info(),
        };

        let cpi_ctx_burn = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_burn,
        )
        .with_signer(signer_seeds);

        anchor_spl::token::transfer(cpi_ctx_burn, ctx.accounts.treasure_vault.amount)?;

        Ok(())
    }
}
