mod state;
use state::*;
mod utils;
use utils::*;
mod instructions;
use instructions::*;
mod errors;
use anchor_lang::prelude::*;
use mpl_core::instructions::*;

declare_id!("BXPuyjuKMVtYMdiHumY42cSF7vGWVX2sEyP1jSfBbwR2");

#[program]
pub mod fomo_contract {

    use super::*;

    pub fn create_round(
        ctx: Context<CreateRoundContext>,
        seed: u64,
        name: String,
        uri: String,
    ) -> Result<()> {
        CreateRoundContext::create_round(ctx, seed, name, uri)?;
        Ok(())
    }
    pub fn create_reserves(ctx: Context<CreateRoundReservesContext>) -> Result<()> {
        CreateRoundReservesContext::create_reserves(ctx)?;
        Ok(())
    }

    pub fn create_key(ctx: Context<CreateKeyContext>) -> Result<()> {
        CreateKeyContext::create_key(ctx)?;
        Ok(())
    }
    pub fn burn_key(ctx: Context<BurnKeyContext>) -> Result<()> {
        BurnKeyContext::burn(ctx)?;
        Ok(())
    }

    pub fn winner_claim(ctx: Context<WinnerClaimContext>) -> Result<()> {
        WinnerClaimContext::winner_claim(ctx)?;
        Ok(())
    }
    pub fn claim_round(ctx: Context<RoundClaimContext>) -> Result<()> {
        RoundClaimContext::claim(ctx)?;
        Ok(())
    }
}
