use anchor_lang::prelude::*;

#[account]
#[derive(Default, InitSpace)]
pub struct NftKey {
    pub nft_mint: Pubkey,
    pub key_index: u64,
    pub exited: u8,
    pub bump: u8,
}

pub struct CreateKeyArgs {
    pub nft_mint: Pubkey,
    pub key_index: u64,
    pub bump: u8,
}
impl NftKey {
    pub fn create(&mut self, args: CreateKeyArgs) {
        self.nft_mint = args.nft_mint;
        self.bump = args.bump;
        self.key_index = args.key_index;
        self.exited = 0;
    }
}
