use anchor_lang::{prelude::*, system_program};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_core::{
    instructions::{CreateV2Cpi, CreateV2InstructionArgs, UpdatePluginV1Cpi},
    types::{DataState, FreezeDelegate},
};

use crate::{
    errors::CustomErrors, state::*, Config, BURN_FEE_BASIS_POINTS, POOL_FEE_BASIS_POINTS,
    SLOT_TO_CHANGE, TEAM_FEE_BASIS_POINTS, TREASURE_FEE_BASIS_POINTS,
};

#[derive(Accounts)]
pub struct CreateKeyContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut,constraint = authority_ata.mint ==  token_mint.key(), constraint = authority_ata.owner == authority.key())]
    pub authority_ata: Box<Account<'info, TokenAccount>>,
    /// The address of the new asset.
    #[account(mut)]
    pub asset: Signer<'info>,

    /// CHECK: checked
    #[account(mut, address = current_key_account.nft_mint)]
    pub current_asset: AccountInfo<'info>,

    #[account(mut, seeds = [b"round", round_account.seed.to_le_bytes().as_ref()],bump = round_account.bump )]
    pub round_account: Box<Account<'info, Round>>,

    /// The collection to which the asset belongs.
    /// CHECK: Checked in mpl-core.
    #[account(mut, address = round_account.collection.key())]
    pub collection: AccountInfo<'info>,

    #[account(init, payer = authority,space = 8 + NftKey::INIT_SPACE, seeds = [b"key",round_account.key().as_ref(),(round_account.mint_counter + 1).to_le_bytes().as_ref()],bump)]
    pub key_account: Box<Account<'info, NftKey>>,

    #[account(mut, seeds = [b"key",round_account.key().as_ref(),round_account.mint_counter.to_le_bytes().as_ref()],bump = current_key_account.bump)]
    pub current_key_account: Box<Account<'info, NftKey>>,

    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(mut,
        address = round_account.team_vault.key()
    )]
    pub team_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut,address = round_account.pool_vault.key() )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut,
           address = round_account.treasure_vault.key()
    )]
    pub treasure_vault: Box<Account<'info, TokenAccount>>,

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

impl CreateKeyContext<'_> {
    pub fn validate(&self) -> Result<()> {
        let current_slot = Clock::get().unwrap().slot;

        // check if round is over
        require_gt!(
            self.round_account.round_close_slot,
            current_slot,
            CustomErrors::RoundOver
        );

        Ok(())
    }

    #[access_control(ctx.accounts.validate())]
    pub fn create_key(ctx: Context<CreateKeyContext>) -> Result<()> {
        let key_account = &mut ctx.accounts.key_account;
        let round_account = &mut ctx.accounts.round_account;
        let current_counter = round_account.mint_counter + 1;
        let current_slot = Clock::get().unwrap().slot;
        let current_key_account = &mut ctx.accounts.current_key_account;

        key_account.create(CreateKeyArgs {
            nft_mint: ctx.accounts.asset.key(),
            bump: ctx.bumps.key_account,
            key_index: current_counter,
        });

        // assuming token as 6 decimal
        let total_amount_for_index = 1000_000 * 10 * current_counter;

        round_account.mint_counter = current_counter;
        round_account.round_close_slot = current_slot + SLOT_TO_CHANGE;

        // Burn Transfer Ix
        let transfer_instruction_burn = anchor_spl::token::Transfer {
            from: ctx.accounts.authority_ata.to_account_info(),
            to: ctx.accounts.team_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_ctx_burn = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_burn,
        );

        anchor_spl::token::transfer(
            cpi_ctx_burn,
            total_amount_for_index
                .checked_mul(BURN_FEE_BASIS_POINTS)
                .unwrap()
                .checked_div(10000)
                .unwrap(),
        )?;

        // Team Transfer Ix
        let transfer_instruction_team = anchor_spl::token::Transfer {
            from: ctx.accounts.authority_ata.to_account_info(),
            to: ctx.accounts.team_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_ctx_team = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_team,
        );

        anchor_spl::token::transfer(
            cpi_ctx_team,
            total_amount_for_index
                .checked_mul(TEAM_FEE_BASIS_POINTS)
                .unwrap()
                .checked_div(10000)
                .unwrap(),
        )?;

        // Pool Transfer Ix
        let transfer_instruction_pool = anchor_spl::token::Transfer {
            from: ctx.accounts.authority_ata.to_account_info(),
            to: ctx.accounts.pool_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_ctx_pool = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_pool,
        );

        anchor_spl::token::transfer(
            cpi_ctx_pool,
            total_amount_for_index
                .checked_mul(POOL_FEE_BASIS_POINTS)
                .unwrap()
                .checked_div(10000)
                .unwrap(),
        )?;

        // Treasure Transfer Ix
        let transfer_instruction_treasure = anchor_spl::token::Transfer {
            from: ctx.accounts.authority_ata.to_account_info(),
            to: ctx.accounts.treasure_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_ctx_treasure = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction_treasure,
        );

        anchor_spl::token::transfer(
            cpi_ctx_treasure,
            total_amount_for_index
                .checked_mul(TREASURE_FEE_BASIS_POINTS)
                .unwrap()
                .checked_div(10000)
                .unwrap(),
        )?;

        let config = Config::get_master(round_account.key());
        let seeds: &[&[&[u8]]] = &[&[
            b"round",
            &round_account.seed.to_le_bytes(),
            &[round_account.bump],
        ]];
        CreateV2Cpi {
            asset: &ctx.accounts.asset.to_account_info(),
            collection: Some(ctx.accounts.collection.as_ref()),
            authority: Some(ctx.accounts.authority.to_account_info().as_ref()),
            payer: &ctx.accounts.authority.to_account_info(),
            owner: Some(ctx.accounts.authority.as_ref()),
            update_authority: Some(round_account.to_account_info().as_ref()),
            system_program: &ctx.accounts.system_program.to_account_info(),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            __program: &ctx.accounts.mpl_core,
            __args: CreateV2InstructionArgs {
                name: config.name,
                uri: config.uri,
                data_state: DataState::AccountState,
                plugins: Some(config.plugins),
                external_plugin_adapters: None,
            },
        }
        .invoke_signed(seeds)?;

        UpdatePluginV1Cpi {
            asset: &ctx.accounts.current_asset,
            __program: &ctx.accounts.mpl_core,
            payer: &ctx.accounts.authority.to_account_info(),
            system_program: &ctx.accounts.system_program.to_account_info(),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            collection: Some(ctx.accounts.collection.as_ref()),
            authority: Some(round_account.to_account_info().as_ref()),
            __args: {
                mpl_core::instructions::UpdatePluginV1InstructionArgs {
                    plugin: mpl_core::types::Plugin::FreezeDelegate(FreezeDelegate {
                        frozen: false,
                    }),
                }
            },
        }
        .invoke_signed(seeds)?;

        Ok(())
    }
}
