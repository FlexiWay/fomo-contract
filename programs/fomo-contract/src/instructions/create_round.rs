use anchor_lang::{ prelude::*, system_program };
use anchor_spl::token::Token;
use mpl_core::instructions::{ CreateCollectionV2Cpi, CreateCollectionV2InstructionArgs };
use crate::{ state::*, Config, SLOT_TO_CHANGE, INCREMENT_AMOUNT };

/// Accounts structure for the `create_round` instruction
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct CreateRoundContext<'info> {
    /// Authority account, typically the user initiating the transaction
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The collection account, representing the NFT collection authority
    #[account(mut)]
    pub collection: Signer<'info>,

    /// The `Round` account initialized by this instruction, used to store round details
    #[account(
        init,
        payer = authority,
        space = 8 + Round::INIT_SPACE,
        seeds = [b"round", seed.to_le_bytes().as_ref()],
        bump
    )]
    pub round_account: Box<Account<'info, Round>>,

    /// The SPL token program
    pub token_program: Program<'info, Token>,

    /// The MPL Core program
    /// CHECK: This is the MPL Core program, and is checked for validity in mpl-core.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,

    /// The Solana system program
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl CreateRoundContext<'_> {
    /// Creates a new round within the application
    pub fn create_round(
        ctx: Context<CreateRoundContext>,
        seed: u64,
        name: String,
        uri: String
    ) -> Result<()> {
        let round_account = &mut ctx.accounts.round_account;
        // msg!("round_account: {}", round_account.key());

        // Get the current slot to calculate when the round will close
        let current_slot: u64 = Clock::get()?.slot;

        let round_bump: u8 = *ctx.bumps.get("round_account").unwrap();

        // Create signer seeds for signing instructions with the program-derived address (PDA)
        let signer_seeds: &[&[&[u8]]] = &[
            &[b"round", &seed.to_le_bytes(), &[round_bump]],
        ];

        // Populate the round account with initial data
        round_account.create(RoundCreateArgs {
            seed,
            authority: ctx.accounts.authority.key(),
            bump: round_bump,
            mint_fee_vault: Pubkey::default(),
            nft_pool_vault: Pubkey::default(),
            main_pool_vault: Pubkey::default(),
            round_close_slot: current_slot + SLOT_TO_CHANGE,
            round_increment: INCREMENT_AMOUNT,
            collection: ctx.accounts.collection.key(),
        });

        // Fetch the collection configuration for creating the collection on-chain
        let config = Config::get_collection(round_account.key());

        // Call the Metaplex Core CPI to create a new NFT collection
        (CreateCollectionV2Cpi {
            collection: &ctx.accounts.collection.as_ref(),
            payer: &ctx.accounts.authority.to_account_info(),
            update_authority: Some(ctx.accounts.round_account.to_account_info().as_ref()),
            system_program: &ctx.accounts.system_program.to_account_info(),
            __program: &ctx.accounts.mpl_core,
            __args: CreateCollectionV2InstructionArgs {
                name,
                uri,
                plugins: Some(config.plugins),
                external_plugin_adapters: None,
            },
        }).invoke_signed(signer_seeds)?;

        Ok(())
    }
}
