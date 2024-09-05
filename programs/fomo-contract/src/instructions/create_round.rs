use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_core::instructions::{CreateCollectionV2Cpi, CreateCollectionV2InstructionArgs};

use crate::{state::*, Config, SLOT_TO_CHANGE};

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct CreateRoundContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub collection: Signer<'info>,

    #[account(init, payer = authority,space = 8 + Round::INIT_SPACE, seeds = [b"round", seed.to_le_bytes().as_ref()],bump)]
    pub round_account: Box<Account<'info, Round>>,

    pub token_mint: Box<Account<'info, Mint>>,

    #[account(init,
        seeds = [b"team", round_account.key().as_ref()],
        bump,
        payer = authority,
        token::mint = token_mint,
        token::authority = round_account,
    )]
    pub team_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    /// The SPL Noop program.
    /// CHECK: Checked in mpl-core.
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// The MPL Core program.
    /// CHECK: Checked in mpl-core.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}
impl CreateRoundContext<'_> {
    pub fn create_round(
        ctx: Context<CreateRoundContext>,
        seed: u64,
        name: String,
        uri: String,
    ) -> Result<()> {
        let round_account = &mut ctx.accounts.round_account;

        let current_slot = Clock::get().unwrap().slot;
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"round",
            &round_account.seed.to_le_bytes(),
            &[round_account.bump],
        ]];
        round_account.create(RoundCreateArgs {
            seed,
            authority: ctx.accounts.authority.key(),
            bump: ctx.bumps.round_account,
            team_vault: ctx.accounts.team_vault.key(),
            treasure_vault: Pubkey::default(),
            pool_vault: Pubkey::default(),
            round_close_slot: current_slot + SLOT_TO_CHANGE,
            collection: ctx.accounts.collection.key(),
        });

        let config = Config::get_collection(round_account.key());
        // cpis into metaplex and makes the collection
        CreateCollectionV2Cpi {
            collection: &ctx.accounts.collection.as_ref(),
            payer: &ctx.accounts.authority.to_account_info(),
            update_authority: Some(ctx.accounts.round_account.to_account_info().as_ref()),
            system_program: &ctx.accounts.system_program.to_account_info(),
            __program: &ctx.accounts.mpl_core,
            __args: mpl_core::instructions::CreateCollectionV2InstructionArgs {
                name,
                uri,
                plugins: Some(config.plugins),
                external_plugin_adapters: None,
            },
        }
        .invoke_signed(signer_seeds)?;

        Ok(())
    }
}
