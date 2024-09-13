use anchor_lang::prelude::*;

mod state;
use state::*;
mod utils;
use utils::*;
mod instructions;
use instructions::*;
mod errors;

declare_id!("HAsnCcr3E3Yq4uP9NqdPkRH8dSbPFPH6t8VJ16z6HfwQ");

#[program]
pub mod fomo_contract {
    use super::*;

    pub fn create_round(
        ctx: Context<CreateRoundContext>,
        seed: u64,
        name: String,
        uri: String
    ) -> Result<()> {
        CreateRoundContext::create_round(ctx, seed, name, uri)?;
        Ok(())
    }

    pub fn create_vaults(ctx: Context<CreateRoundVaultsContext>) -> Result<()> {
        CreateRoundVaultsContext::create_vaults(ctx)?;
        Ok(())
    }

    pub fn start_round(ctx: Context<StartRoundContext>) -> Result<()> {
        StartRoundContext::start_round(ctx)?;
        Ok(())
    }

    pub fn update_increment(ctx: Context<UpdateRoundContext>, increment_amount: u64) -> Result<()> {
        UpdateRoundContext::update_increment(ctx, increment_amount)?;
        Ok(())
    }

    pub fn create_key(ctx: Context<CreateKeyContext>) -> Result<()> {
        CreateKeyContext::create_key(ctx)?;
        Ok(())
    }

    pub fn burn_key(ctx: Context<BurnKeyContext>) -> Result<()> {
        BurnKeyContext::burn_key(ctx)?;
        Ok(())
    }

    pub fn winner_claim(ctx: Context<WinnerClaimContext>) -> Result<()> {
        WinnerClaimContext::winner_claim(ctx)?;
        Ok(())
    }

    pub fn fee_claim(ctx: Context<FeeClaimContext>) -> Result<()> {
        FeeClaimContext::fee_claim(ctx)?;
        Ok(())
    }

}
