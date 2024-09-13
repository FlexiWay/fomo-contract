use anchor_lang::{ prelude::*, system_program };
use anchor_spl::{ token::{ Mint, Token, TokenAccount }, associated_token::AssociatedToken };
use mpl_core::{
    accounts::BaseAssetV1,
    instructions::{ BurnV1Cpi, BurnV1InstructionArgs },
    types::UpdateAuthority,
};
use crate::{ errors::FomoErrors, state::* };

/// Context for the `BurnKey` instruction, defining all necessary accounts and constraints.
#[derive(Accounts)]
pub struct BurnKeyContext<'info> {
    /// Authority account, must be a signer.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Authority's token account associated with the asset mint.
    /// If the account does not exist, it will be initialized.
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = authority
    )]
    pub authority_ata: Box<Account<'info, TokenAccount>>,

    /// Round account storing round-specific data, constrained by seed and bump.
    #[account(
        mut,
        seeds = [b"round", round_account.seed.to_le_bytes().as_ref()],
        bump = round_account.bump
    )]
    pub round_account: Box<Account<'info, Round>>,

    /// Collection account associated with the round.
    /// CHECK: Manually validated by comparing the address.
    #[account(
        mut,
        address = round_account.collection.key()
    )]
    pub collection: AccountInfo<'info>,

    /// Asset (NFT) to be burned, checked against the key account's mint.
    /// CHECK: The asset's validity is enforced by logic.
    #[account(
        mut,
        constraint = asset.key() == key_account.nft_mint.key()
    )]
    pub asset: UncheckedAccount<'info>,

    /// Key account representing the NFT being burned, constrained by seed and bump.
    #[account(
        mut,
        seeds = [b"key", round_account.key().as_ref(), key_account.key_index.to_le_bytes().as_ref()],
        bump = key_account.bump
    )]
    pub key_account: Box<Account<'info, NftKey>>,

    /// Vault holding the NFT pool for the round.
    #[account(
        mut,
        address = round_account.nft_pool_vault.key()
    )]
    pub nft_pool_vault: Box<Account<'info, TokenAccount>>,

    /// Mint of the tokens associated with the round.
    pub token_mint: Box<Account<'info, Mint>>,

    /// Token program for handling token operations.
    pub token_program: Program<'info, Token>,

    /// Associated Token Program for creating associated token accounts.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Optional SPL Noop program for logging events.
    /// CHECK: Verified by Metaplex Core.
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// Metaplex Core program used for burning the NFT.
    /// CHECK: Address validated against Metaplex Core ID.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,

    /// System Program for creating and transferring lamports.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl BurnKeyContext<'_> {
    /// Validates the accounts and checks if the asset's authority matches the expected collection.
    ///
    /// Ensures the update authority of the asset is aligned with the collection and the owner
    /// of the asset is the authority performing the burn.
    pub fn validate(&self) -> Result<()> {
        let asset_data = BaseAssetV1::from_bytes(&self.asset.to_account_info().data.borrow())?;

        // Check if the update authority of the asset matches the round collection.
        if
            asset_data.update_authority !=
            UpdateAuthority::Collection(self.round_account.collection)
        {
            return Err(FomoErrors::InvalidKeyAccount.into());
        }

        // Ensure the asset owner is the authority account.
        require_eq!(asset_data.owner, self.authority.key(), FomoErrors::InvalidOwner);

        Ok(())
    }

    /// Executes the burn operation for the NFT, distributing the token rewards and updating state.
    ///
    /// This function handles the following:
    /// - Verifies the key account is valid and not previously exited.
    /// - Calculates the average token amount in the pool per holder.
    /// - Transfers the appropriate amount of tokens to the authority's ATA.
    /// - Marks the NFT key as burned and invokes the Metaplex Core burn instruction.
    #[access_control(ctx.accounts.validate())]
    pub fn burn_key(ctx: Context<BurnKeyContext>) -> Result<()> {
        let key_account = &mut ctx.accounts.key_account;
        let round_account: &mut Box<Account<'_, Round>> = &mut ctx.accounts.round_account;

        // Calculate the current number of holders.
        let current_holder_counter = round_account.mint_counter
            .checked_sub(round_account.nft_burn_counter)
            .ok_or(FomoErrors::CalculationError)?;

        // Ensure the key account hasn't already been burned.
        require_neq!(key_account.exited, 1, FomoErrors::InvalidKeyAccount);

        // Calculate the average amount of tokens in the pool per holder.
        let token_in_pool = ctx.accounts.nft_pool_vault.amount;
        let avg_amount_pool = token_in_pool
            .checked_div(current_holder_counter)
            .ok_or(FomoErrors::DivisionError)?;

        // Seed for round authority verification.
        let signer_seeds: &[&[&[u8]]] = &[
            &[b"round", &round_account.seed.to_le_bytes(), &[round_account.bump]],
        ];

        // Perform the token transfer from the pool to the authority.
        let transfer_instruction_burn = anchor_spl::token::Transfer {
            from: ctx.accounts.nft_pool_vault.to_account_info(),
            to: ctx.accounts.authority_ata.to_account_info(),
            authority: round_account.to_account_info(),
        };

        let cpi_ctx_burn = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_burn
        ).with_signer(signer_seeds);

        anchor_spl::token::transfer(cpi_ctx_burn, avg_amount_pool)?;

        // Increment the burn counter and mark the key as burned.
        round_account.nft_burn_counter += 1;
        key_account.exited = 1;

        // Invoke the burn instruction through Metaplex Core.
        (BurnV1Cpi {
            asset: &ctx.accounts.asset.to_account_info(),
            authority: Some(ctx.accounts.authority.to_account_info().as_ref()),
            collection: Some(&ctx.accounts.collection),
            payer: &ctx.accounts.authority.to_account_info(),
            system_program: Some(&ctx.accounts.system_program.to_account_info()),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            __program: &ctx.accounts.mpl_core,
            __args: BurnV1InstructionArgs {
                compression_proof: None,
            },
        }).invoke_signed(signer_seeds)?;

        Ok(())
    }
}
