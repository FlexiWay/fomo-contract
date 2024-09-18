use anchor_lang::prelude::*;

/// Struct representing a round in the system, containing various
/// parameters for tracking the state and operations within the round.
#[account]
#[derive(Default, InitSpace)]
pub struct Round {
    /// The authority or admin that has control over this round.
    pub authority: Pubkey,
    /// A unique seed used for generating new rounds.
    pub seed: u64,
    /// Counter for the number of NFTs minted in this round.
    pub mint_counter: u64,
    /// Counter for the number of NFTs burned in this round.
    pub nft_burn_counter: u64,
    /// A slot indicating when the round will close (24-hour window).
    pub round_close_timestamp: u64,
    /// The base fee required to mint an NFT during this round.
    pub round_basic_mint_fee: u64,
    /// The increment value for fees or other progressive parameters.
    pub round_increment: u64,
    /// Vault holding the main pool for this round.
    pub main_pool_vault: Pubkey,
    /// Vault holding NFTs associated with this round.
    pub nft_pool_vault: Pubkey,
    /// Vault for collecting minting fees in this round.
    pub mint_fee_vault: Pubkey,
    /// The public key of the NFT collection associated with this round.
    pub collection: Pubkey,
    /// The bump seed for the PDA (Program Derived Address) of this round.
    pub bump: u8,
}

/// Arguments for creating a new round. These are passed when initializing
/// a new round instance.
pub struct RoundCreateArgs {
    /// Unique seed for identifying the round.
    pub seed: u64,
    /// Admin authority for the round.
    pub authority: Pubkey,
    /// Bump seed for the PDA (Program Derived Address).
    pub bump: u8,
    /// Vault for collecting mint fees.
    pub mint_fee_vault: Pubkey,
    /// Vault for holding NFTs associated with the round.
    pub nft_pool_vault: Pubkey,
    /// Vault for the main pool associated with the round.
    pub main_pool_vault: Pubkey,
    /// The closing slot for the round (24-hour window).
    pub round_close_timestamp: u64,
    /// Increment value for progressive parameters (e.g., fees).
    pub round_increment: u64,
    /// Public key of the NFT collection associated with the round.
    pub collection: Pubkey,
}

pub struct RoundCreateVaultsArgs {
    /// Vault for collecting mint fees.
    pub mint_fee_vault: Pubkey,
    /// Vault for holding NFTs associated with the round.
    pub nft_pool_vault: Pubkey,
    /// Vault for the main pool associated with the round.
    pub main_pool_vault: Pubkey,
}

pub struct UpdateRoundIncrementArg {
    /// Increment value for progressive parameters (e.g., fees).
    pub round_increment: u64,
}

impl Round {
    /// Creates and initializes a new round with the provided arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments required to create a new round,
    ///            such as the seed, authority, vaults, and collection.
    pub fn create(&mut self, args: RoundCreateArgs) {
        self.seed = args.seed;
        self.authority = args.authority;
        self.bump = args.bump;
        self.mint_fee_vault = args.mint_fee_vault;
        self.nft_pool_vault = args.nft_pool_vault;
        self.main_pool_vault = args.main_pool_vault;
        self.round_close_timestamp = args.round_close_timestamp;
        self.round_increment = args.round_increment;
        self.mint_counter = 0; // Initialize mint counter to 0 for the new round
        self.nft_burn_counter = 0; // Initialize burn counter to 0 for the new round
        self.collection = args.collection;
    }

    pub fn create_vaults(&mut self, args: RoundCreateVaultsArgs) {
        self.mint_fee_vault = args.mint_fee_vault;
        self.nft_pool_vault = args.nft_pool_vault;
        self.main_pool_vault = args.main_pool_vault;
    }

    pub fn update_increment(&mut self, args: UpdateRoundIncrementArg) {
        self.round_increment = args.round_increment;
    }
}

/// Struct representing an NFT key, containing the mint information,
/// indexing details, and other metadata associated with the key.
#[account]
#[derive(Default, InitSpace)]
pub struct NftKey {
    /// The public key of the NFT mint associated with this key.
    pub nft_mint: Pubkey,
    /// Index of the key, used for uniquely identifying it within a collection or system.
    pub key_index: u64,
    /// A flag to indicate whether this key has exited or been deactivated (0 = active, 1 = exited).
    pub exited: u8,
    /// Bump seed for Program Derived Address (PDA) associated with this key.
    pub bump: u8,
}

/// Struct for arguments required to create a new `NftKey` instance.
pub struct CreateKeyArgs {
    /// Public key of the NFT mint.
    pub nft_mint: Pubkey,
    /// Unique index for this key.
    pub key_index: u64,
    /// Bump seed for the Program Derived Address (PDA).
    pub bump: u8,
}

impl NftKey {
    /// Initializes a new NFT key with the provided arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - Struct containing the necessary fields for initializing the NFT key.
    pub fn create(&mut self, args: CreateKeyArgs) {
        self.nft_mint = args.nft_mint; // Assign the NFT mint public key
        self.key_index = args.key_index; // Set the key index for this NFT key
        self.bump = args.bump; // Assign the bump seed for PDA derivation
        self.exited = 0; // Initialize the 'exited' flag as active (0)
    }
}
