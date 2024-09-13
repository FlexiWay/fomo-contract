use anchor_lang::prelude::*;

#[error_code]
pub enum FomoErrors {
    #[msg("Invalid account key")]
    InvalidKeyAccount,
    #[msg("Round already started")]
    RoundStarted,
    #[msg("Round over")]
    RoundOver,
    #[msg("Round not over")]
    RoundNotOver,
    #[msg("Invalid Asset")]
    InvalidAsset,
    #[msg("An error occurred during calculation.")]
    CalculationError,
    #[msg("An error occurred during division, likely division by zero.")]
    DivisionError,
    #[msg("Doesn't matched with token owner")]
    InvalidOwner
}
