// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{contracttype, Address, String};

/// on-chain user profile.
///
/// Stores only the blockchain address and an off-chain reference ID
/// that maps to the full user record in the off-chain database.
/// All PII (name, email, country, etc.) is stored off-chain.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct UserProfile {
    /// User's blockchain address
    pub address: Address,
    /// Off-chain reference ID (UUID mapping to DB record)
    pub off_chain_ref_id: String,
    /// Optional DID hash for decentralized identity verification
    pub did_hash: Option<String>,
    /// Timestamp when the profile was created
    pub created_at: u64,
    /// Timestamp when the profile was last updated
    pub updated_at: u64,
}

/// Storage keys for user profile data.
///
/// This enum defines the keys used to store and retrieve
/// user profile data from the contract's persistent storage.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Key for storing user profiles: address -> UserProfile
    Profile(Address),
}
