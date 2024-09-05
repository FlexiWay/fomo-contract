use anchor_lang::prelude::*;

#[account]
#[derive(Default, InitSpace)]
pub struct Round {
    // use for admin of the round
    pub authority: Pubkey,
    // unique seed for future rounds
    pub seed: u64,
    // counter for nft mint
    pub mint_counter: u64,
    // counter for nft burned
    pub nft_burn_counter: u64,
    // counter for 24 hours
    pub round_close_slot: u64,
    pub team_vault: Pubkey,
    pub pool_vault: Pubkey,
    pub treasure_vault: Pubkey,
    pub collection: Pubkey,
    pub bump: u8,
}

pub struct RoundCreateArgs {
    pub seed: u64,
    pub round_close_slot: u64,
    pub authority: Pubkey,
    pub bump: u8,
    pub team_vault: Pubkey,
    pub pool_vault: Pubkey,
    pub treasure_vault: Pubkey,
    pub collection: Pubkey,
}
pub struct RoundCreateReservesArgs {
    pub pool_vault: Pubkey,
    pub treasure_vault: Pubkey,
}

impl Round {
    pub fn create(&mut self, args: RoundCreateArgs) {
        self.seed = args.seed;
        self.authority = args.authority;
        self.team_vault = args.team_vault;
        self.pool_vault = args.pool_vault;
        self.treasure_vault = args.treasure_vault;
        self.bump = args.bump;
        self.mint_counter = 0;
        self.round_close_slot = args.round_close_slot;
        self.nft_burn_counter = 0;
        self.collection = args.collection;
    }
    pub fn create_reserves(&mut self, args: RoundCreateReservesArgs) {
        self.pool_vault = args.pool_vault;
        self.treasure_vault = args.treasure_vault;
    }
}
