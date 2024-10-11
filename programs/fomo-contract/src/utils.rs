use anchor_lang::prelude::* ;
use mpl_core::types::{
    Creator,
    FreezeDelegate,
    Plugin,
    PluginAuthority,
    PluginAuthorityPair,
    Royalties,
    VerifiedCreators,
    VerifiedCreatorsSignature,
};

pub const MAIN_FEE_BASIS_POINTS: u64 = 7000;
pub const NFT_FEE_BASIS_POINTS: u64 = 2500;
pub const MINT_FEE_BASIS_POINTS: u64 = 500;
pub const TIME_TO_CHANGE: u64 = 3 * 60 * 60; // 86400
pub const INCREMENT_AMOUNT: u64 = 500_000; // 40000000

pub struct Config {
    pub name: String,
    pub uri: String,
    pub plugins: Vec<PluginAuthorityPair>,
}

impl Config {
    pub fn get_collection(round_account: Pubkey) -> Config {
        Config {
            name: String::from(""), // Not in use
            uri: String::from(""), // Not in use
            plugins: Vec::from([
                PluginAuthorityPair {
                    plugin: Plugin::Royalties(Royalties {
                        basis_points: 690,
                        creators: Vec::from([
                            Creator {
                                address: round_account,
                                percentage: 100,
                            },
                        ]),
                        rule_set: mpl_core::types::RuleSet::None,
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
            ]),
        }
    }
    pub fn get_master(round_account: Pubkey) -> Config {
        Config {
            name: String::from("Master Key"),
            uri: String::from("https://purple-quickest-catshark-409.mypinata.cloud/ipfs/QmXEFnXdMLeSCtzEk8gaiEcDq18DncZ8aRduSNmDgUa2kr"),
            plugins: Vec::from([
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
                PluginAuthorityPair {
                    plugin: Plugin::VerifiedCreators(VerifiedCreators {
                        signatures: Vec::from([
                            VerifiedCreatorsSignature {
                                verified: true,
                                address: round_account,
                            },
                        ]),
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
            ]),
        }
    }
    pub fn get_default(round_account: Pubkey) -> Config {
        Config {
            name: String::from("Collector Key"),
            uri: String::from("https://purple-quickest-catshark-409.mypinata.cloud/ipfs/QmQmaHT4SzGnRZHEf7YBRB37DCWmQvRaPgbkQmPhfKrT4i"),
            plugins: Vec::from([
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
                PluginAuthorityPair {
                    plugin: Plugin::VerifiedCreators(VerifiedCreators {
                        signatures: Vec::from([
                            VerifiedCreatorsSignature {
                                verified: true,
                                address: round_account,
                            },
                        ]),
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
            ]),
        }
    }
}

