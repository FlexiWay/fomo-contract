use anchor_lang::prelude::*;
use crate::state::*;

/// Accounts required for the `update_increment` instruction.
/// This context includes the authority responsible for updating
/// the round and the round account itself.
#[derive(Accounts)]
pub struct UpdateRoundContext<'info> {
    /// The authority account, which must match the round account's authority.
    /// The authority is responsible for making updates to the round account.
    #[account(mut, constraint = authority.key() == round_account.authority.key())]
    pub authority: Signer<'info>,

    /// The round account that is being updated.
    /// This account is initialized with a PDA and is mutable for updates.
    #[account(
        mut,
        seeds = [b"round", round_account.seed.to_le_bytes().as_ref()],
        bump = round_account.bump
    )]
    pub round_account: Box<Account<'info, Round>>,
}

impl UpdateRoundContext<'_> {
    /// Updates the increment value of the `round_account`.
    pub fn update_increment(ctx: Context<UpdateRoundContext>, increment_amount: u64) -> Result<()> {
        // Access the round account from the context and apply the new increment amount.
        let round_account = &mut ctx.accounts.round_account;

        // Update the round account's increment value using the provided argument.
        round_account.update_increment(UpdateRoundIncrementArg {
            round_increment: increment_amount,
        });

        Ok(())
    }
}
