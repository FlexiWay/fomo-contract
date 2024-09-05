use anchor_lang::{prelude::Pubkey, Key};
use mpl_core::types::{
    Creator, FreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair, Royalties,
    VerifiedCreators, VerifiedCreatorsSignature,
};

pub const TEAM_FEE_BASIS_POINTS: u64 = 420;
pub const POOL_FEE_BASIS_POINTS: u64 = 690;
pub const BURN_FEE_BASIS_POINTS: u64 = 990;
pub const TREASURE_FEE_BASIS_POINTS: u64 = 7900;
pub const SLOT_TO_CHANGE: u64 = 200000; // 200000

pub struct Config {
    pub name: String,
    pub uri: String,
    pub plugins: Vec<PluginAuthorityPair>,
}

impl Config {
    pub fn get_collection(round_account: Pubkey) -> Config {
        Config {
            name: String::from(""), // Not in use
            uri: String::from(""),  // Not in use
            plugins: Vec::from([PluginAuthorityPair {
                plugin: Plugin::Royalties(Royalties {
                    basis_points: 690,
                    creators: Vec::from([Creator {
                        address: round_account,
                        percentage: 100,
                    }]),
                    rule_set: mpl_core::types::RuleSet::None,
                }),
                authority: Some(PluginAuthority::UpdateAuthority),
            }]),
        }
    }
    pub fn get_master(round_account: Pubkey) -> Config {
        Config {
            name: String::from("Master Key"),
            uri: String::from("http://"),
            plugins: Vec::from([
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
                PluginAuthorityPair {
                    plugin: Plugin::VerifiedCreators(VerifiedCreators {
                        signatures: Vec::from([VerifiedCreatorsSignature {
                            verified: true,
                            address: round_account,
                        }]),
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
            ]),
        }
    }
    pub fn get_default(round_account: Pubkey) -> Config {
        Config {
            name: String::from("Key"),
            uri: String::from("http://"),
            plugins: Vec::from([
                PluginAuthorityPair {
                    plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
                PluginAuthorityPair {
                    plugin: Plugin::VerifiedCreators(VerifiedCreators {
                        signatures: Vec::from([VerifiedCreatorsSignature {
                            verified: true,
                            address: round_account,
                        }]),
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
            ]),
        }
    }
}
