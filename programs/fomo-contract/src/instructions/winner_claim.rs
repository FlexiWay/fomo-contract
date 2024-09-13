use anchor_lang::{ prelude::*, system_program };
use anchor_spl::{ token::{ Mint, Token, TokenAccount }, associated_token::AssociatedToken };
use mpl_core::{ accounts::BaseAssetV1, types::UpdateAuthority };
use crate::{ errors::FomoErrors, NftKey, Round };

/// Context for the `WinnerClaim` instruction, defining all required accounts and constraints.
#[derive(Accounts)]
pub struct WinnerClaimContext<'info> {
    /// Authority of the round, must match the `round_account`'s authority.
    #[account(mut, constraint = round_account.authority == authority.key())]
    pub authority: Signer<'info>,

    /// Winner's account. This is unchecked but will be validated later in the logic.
    /// CHECK: Verified in the logic.
    #[account(mut)]
    pub winner: UncheckedAccount<'info>,

    /// Winner's associated token account. This will be initialized if needed.
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = winner
    )]
    pub winner_ata: Box<Account<'info, TokenAccount>>,

    /// The round account, derived using seeds and bump.
    #[account(mut, seeds = [b"round", round_account.seed.to_le_bytes().as_ref()], bump = round_account.bump)]
    pub round_account: Box<Account<'info, Round>>,

    /// Asset account (NFT mint), validated to match the key account's mint.
    /// CHECK: Asset address will be validated in the logic.
    #[account(mut, constraint = asset.key() == key_account.nft_mint.key())]
    pub asset: UncheckedAccount<'info>,

    /// Key account for the NFT, constrained by the round's mint counter and seeds.
    #[account(mut,
        constraint = key_account.key_index == round_account.mint_counter,
        seeds = [b"key", round_account.key().as_ref(), key_account.key_index.to_le_bytes().as_ref()],
        bump = key_account.bump
    )]
    pub key_account: Box<Account<'info, NftKey>>,

    /// The token mint for the asset being claimed.
    pub token_mint: Box<Account<'info, Mint>>,

    /// Vault for tokens, owned by the `round_account`.
    #[account(address = round_account.main_pool_vault.key())]
    pub main_pool_vault: Box<Account<'info, TokenAccount>>,

    /// Token program used for token-related operations.
    pub token_program: Program<'info, Token>,

    /// Associated token program required for creating associated token accounts.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// System program for managing lamports and account creation.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl WinnerClaimContext<'_> {
    /// Validate the conditions before processing the claim.
    pub fn validate(&self) -> Result<()> {
        let current_slot = Clock::get().unwrap().slot;

        // Ensure that the round has ended before proceeding.
        require_gt!(current_slot, self.round_account.round_close_slot, FomoErrors::RoundNotOver);

        // Validate the asset's update authority to ensure it matches the expected collection.
        let asset_data = BaseAssetV1::from_bytes(&self.asset.data.borrow())?;
        if
            asset_data.update_authority !=
            UpdateAuthority::Collection(self.round_account.collection)
        {
            return Err(FomoErrors::InvalidKeyAccount.into());
        }

        Ok(())
    }

    /// Process the winner's claim, transferring assets and validating necessary conditions.
    #[access_control(ctx.accounts.validate())]
    pub fn winner_claim(ctx: Context<WinnerClaimContext>) -> Result<()> {
        let round_account = &mut ctx.accounts.round_account;

        // Define signer seeds for authorization.
        let signer_seeds: &[&[&[u8]]] = &[
            &[b"round", &round_account.seed.to_le_bytes(), &[round_account.bump]],
        ];

        // Prepare token transfer instruction for transferring the winner's assets.
        let transfer_instruction_burn = anchor_spl::token::Transfer {
            from: ctx.accounts.main_pool_vault.to_account_info(),
            to: ctx.accounts.winner_ata.to_account_info(),
            authority: round_account.to_account_info(),
        };

        // Create CPI context with signer seeds for secure transfer.
        let cpi_ctx_burn = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_burn
        ).with_signer(signer_seeds);

        // Execute token transfer to the winner.
        anchor_spl::token::transfer(cpi_ctx_burn, ctx.accounts.main_pool_vault.amount)?;

        Ok(())
    }
}
