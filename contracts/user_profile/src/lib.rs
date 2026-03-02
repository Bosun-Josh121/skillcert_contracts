// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

#![no_std]

/// Contract version for tracking deployments and upgrades
pub const VERSION: &str = "1.0.0";

pub mod functions;
pub mod error;
pub mod schema;

#[cfg(test)]
mod test;

use crate::schema::UserProfile;
use soroban_sdk::{contract, contractimpl, Address, Env};

/// User Profile Contract
///
/// This contract provides read-only access to on-chain user profile
/// information. All PII is stored off-chain; only the blockchain address,
/// an off-chain reference ID, and optional DID hash are stored on-chain.
#[contract]
pub struct UserProfileContract;

#[contractimpl]
impl UserProfileContract {
    /// Get a user profile by address.
    ///
    /// Retrieves the on-chain user profile (address, off_chain_ref_id,
    /// did_hash, timestamps) using the user's blockchain address.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `user_address` - The blockchain address of the user whose profile to retrieve
    ///
    /// # Returns
    ///
    /// Returns the `UserProfile` containing the user's on-chain data.
    pub fn get_user_profile(env: Env, user_address: Address) -> UserProfile {
        functions::get_user_profile::user_profile_get_user_profile(&env, user_address)
    }

    /// Get a user profile with requester context.
    ///
    /// This returns the same data
    /// as `get_user_profile`. Retained for API backward compatibility.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `user_address` - The blockchain address of the user whose profile to retrieve
    /// * `requester_address` - The address of the user requesting the profile (unused after refactor)
    ///
    /// # Returns
    ///
    /// Returns the `UserProfile` with minimal on-chain data.
    pub fn get_user_profile_with_privacy(
        env: Env,
        user_address: Address,
        _requester_address: Address,
    ) -> UserProfile {
        functions::get_user_profile::user_profile_get_user_profile(
            &env,
            user_address,
        )
    }
}
