use anchor_lang::prelude::*;

#[error_code]
pub enum CustomErrors {
    #[msg("Invalid account key")]
    InvalidKeyAccount,
    #[msg("Round over")]
    RoundOver,
    #[msg("Round not over")]
    RoundNotOver,
    #[msg("Invalid Asset")]
    InvalidAsset,
}
