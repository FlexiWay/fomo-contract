use anchor_lang::{ prelude::*, system_program };
use anchor_spl:: {
    token::{ Mint, Token, TokenAccount },
    associated_token::AssociatedToken
};
use crate::{ errors::FomoErrors, Round };

/// Accounts structure for the `fee_claim` instruction
#[derive(Accounts)]
pub struct FeeClaimContext<'info> {
    /// Authority account, must match the authority of the round
    #[account(mut, constraint = round_account.authority == authority.key())]
    pub authority: Signer<'info>,

    /// Associated Token Account (ATA) of the authority for the token mint
    /// Constraints ensure the mint matches and the owner is the authority.
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = authority
    )]
    pub authority_ata: Box<Account<'info, TokenAccount>>,

    /// The round account, must be signed by the authority
    #[account(mut, seeds = [b"round", round_account.seed.to_le_bytes().as_ref()], bump = round_account.bump)]
    pub round_account: Box<Account<'info, Round>>,

    /// The token mint associated with the round
    pub token_mint: Box<Account<'info, Mint>>,

    /// The NFT pool vault, which holds NFTs for the round, must match the vault in the round account
    #[account(mut, address = round_account.mint_fee_vault.key())]
    pub mint_fee_vault: Box<Account<'info, TokenAccount>>,

    /// The SPL Token program
    pub token_program: Program<'info, Token>,

    /// Associated Token Program for creating associated token accounts.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// The Solana system program
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl FeeClaimContext<'_> {
    /// Validates the context before processing the fee claim
    ///
    /// Ensures the round has ended by comparing the current slot with the round's close slot
    pub fn validate(&self) -> Result<()> {
        let current_slot = Clock::get()?.slot;

        // Check if the current blockchain slot is greater than the round's close slot
        require_gt!(current_slot, self.round_account.round_close_slot, FomoErrors::RoundNotOver);
        Ok(())
    }

    /// Claims the fee from the NFT pool vault and transfers it to the authority's ATA
    #[access_control(ctx.accounts.validate())] // Access control based on validation
    pub fn fee_claim(ctx: Context<FeeClaimContext>) -> Result<()> {
        let round_account = &mut ctx.accounts.round_account;

        // Create signer seeds for signing instructions with the program-derived address (PDA)
        let signer_seeds: &[&[&[u8]]] = &[
            &[b"round", &round_account.seed.to_le_bytes(), &[round_account.bump]],
        ];

        // Create the transfer instruction to transfer tokens from the NFT pool to the authority's ATA
        let transfer_instruction_team = anchor_spl::token::Transfer {
            from: ctx.accounts.mint_fee_vault.to_account_info(),
            to: ctx.accounts.authority_ata.to_account_info(),
            authority: round_account.to_account_info(),
        };

        // Build the CPI (Cross-Program Invocation) context for transferring the tokens
        let cpi_ctx_team = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_team
        ).with_signer(signer_seeds);

        // Perform the transfer, sending the total amount from the NFT pool vault to the authority's ATA
        anchor_spl::token::transfer(cpi_ctx_team, ctx.accounts.mint_fee_vault.amount)?;

        Ok(())
    }
}
