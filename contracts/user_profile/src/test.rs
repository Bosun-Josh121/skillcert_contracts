// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol, symbol_short};

use crate::{UserProfileContract, UserProfileContractClient};
use crate::schema::UserProfile;

/// Helper function to create a test user profile
fn create_test_profile(env: &Env, address: Address) -> UserProfile {
    UserProfile {
        address: address.clone(),
        off_chain_ref_id: String::from_str(env, "usr-abc123-def456"),
        did_hash: Some(String::from_str(env, "did:example:abcdef1234567890")),
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
    }
}

/// Helper function to save a profile to storage
fn save_profile_to_storage(env: &Env, profile: &UserProfile) {
    let key: Symbol = symbol_short!("profile");
    env.storage()
        .instance()
        .set(&(key, profile.address.clone()), profile);
}

#[test]
fn test_get_user_profile_success() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user_address: Address = Address::generate(&env);
    let profile: UserProfile = create_test_profile(&env, user_address.clone());

    // Save profile to storage
    env.as_contract(&contract_id, || {
        save_profile_to_storage(&env, &profile);
    });

    // Test getting the profile
    let result: UserProfile = client.get_user_profile(&user_address);
    assert_eq!(result, profile);
}

#[test]
#[should_panic(expected = "escalating error to panic")]
fn test_get_user_profile_not_found() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user_address: Address = Address::generate(&env);

    // Try to get a profile that doesn't exist - should panic with UserProfileNotFound error (code 1)
    client.get_user_profile(&user_address);
}

#[test]
fn test_get_user_profile_with_privacy_returns_same_data() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user_address: Address = Address::generate(&env);
    let requester_address: Address = Address::generate(&env);
    let profile: UserProfile = create_test_profile(&env, user_address.clone());

    // Save profile to storage
    env.as_contract(&contract_id, || {
        save_profile_to_storage(&env, &profile);
    });

    // get_user_profile_with_privacy returns the same data
    // as get_user_profile since no PII is stored on-chain.
    let result: UserProfile = client.get_user_profile_with_privacy(&user_address, &requester_address);
    assert_eq!(result.address, profile.address);
    assert_eq!(result.off_chain_ref_id, profile.off_chain_ref_id);
    assert_eq!(result.did_hash, profile.did_hash);
}

#[test]
fn test_get_user_profile_with_privacy_same_user() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user_address: Address = Address::generate(&env);
    let profile: UserProfile = create_test_profile(&env, user_address.clone());

    // Save profile to storage
    env.as_contract(&contract_id, || {
        save_profile_to_storage(&env, &profile);
    });

    // Same user requesting their own profile
    let result: UserProfile = client.get_user_profile_with_privacy(&user_address, &user_address);
    assert_eq!(result, profile);
}

#[test]
fn test_get_user_profile_with_privacy_different_user() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user_address: Address = Address::generate(&env);
    let requester_address: Address = Address::generate(&env);
    let profile: UserProfile = create_test_profile(&env, user_address.clone());

    // Save profile to storage
    env.as_contract(&contract_id, || {
        save_profile_to_storage(&env, &profile);
    });

    // Different user requesting the profile — returns same data
    // since no PII is on-chain (privacy is handled off-chain)
    let result: UserProfile = client.get_user_profile_with_privacy(&user_address, &requester_address);
    assert_eq!(result.address, profile.address);
    assert_eq!(result.off_chain_ref_id, profile.off_chain_ref_id);
}

#[test]
#[should_panic(expected = "escalating error to panic")]
fn test_get_user_profile_with_privacy_not_found() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user_address: Address = Address::generate(&env);
    let requester_address: Address = Address::generate(&env);

    // Try to get a profile that doesn't exist - should panic with UserProfileNotFound error (code 1)
    client.get_user_profile_with_privacy(&user_address, &requester_address);
}

#[test]
fn test_profile_data_integrity() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user_address: Address = Address::generate(&env);
    let profile: UserProfile = UserProfile {
        address: user_address.clone(),
        off_chain_ref_id: String::from_str(&env, "usr-integrity-test"),
        did_hash: None, // Test with no DID hash
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
    };

    // Save profile to storage
    env.as_contract(&contract_id, || {
        save_profile_to_storage(&env, &profile);
    });

    // Test that all data is preserved correctly
    let result: UserProfile = client.get_user_profile(&user_address);
    assert_eq!(result.address, profile.address);
    assert_eq!(result.off_chain_ref_id, profile.off_chain_ref_id);
    assert_eq!(result.did_hash, profile.did_hash);
    assert_eq!(result.created_at, profile.created_at);
    assert_eq!(result.updated_at, profile.updated_at);
}

#[test]
fn test_multiple_users_profiles() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user1_address: Address = Address::generate(&env);
    let user2_address: Address = Address::generate(&env);

    let profile1: UserProfile = create_test_profile(&env, user1_address.clone());
    let profile2: UserProfile = UserProfile {
        address: user2_address.clone(),
        off_chain_ref_id: String::from_str(&env, "usr-jane-789xyz"),
        did_hash: Some(String::from_str(&env, "did:example:jane9876543210")),
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
    };

    // Save both profiles to storage
    env.as_contract(&contract_id, || {
        save_profile_to_storage(&env, &profile1);
        save_profile_to_storage(&env, &profile2);
    });

    // Test getting both profiles
    let result1: UserProfile = client.get_user_profile(&user1_address);
    let result2: UserProfile = client.get_user_profile(&user2_address);

    assert_eq!(result1, profile1);
    assert_eq!(result2, profile2);
    assert_ne!(result1, result2);
}

#[test]
fn test_profile_with_did_hash() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user_address: Address = Address::generate(&env);
    let profile: UserProfile = UserProfile {
        address: user_address.clone(),
        off_chain_ref_id: String::from_str(&env, "usr-did-test"),
        did_hash: Some(String::from_str(&env, "sha256:abcdef0123456789")),
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
    };

    // Save profile to storage
    env.as_contract(&contract_id, || {
        save_profile_to_storage(&env, &profile);
    });

    let result: UserProfile = client.get_user_profile(&user_address);
    assert_eq!(result.did_hash, Some(String::from_str(&env, "sha256:abcdef0123456789")));
}

#[test]
fn test_profile_without_did_hash() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(UserProfileContract, {});
    let client: UserProfileContractClient<'_> = UserProfileContractClient::new(&env, &contract_id);

    let user_address: Address = Address::generate(&env);
    let profile: UserProfile = UserProfile {
        address: user_address.clone(),
        off_chain_ref_id: String::from_str(&env, "usr-no-did"),
        did_hash: None,
        created_at: env.ledger().timestamp(),
        updated_at: env.ledger().timestamp(),
    };

    // Save profile to storage
    env.as_contract(&contract_id, || {
        save_profile_to_storage(&env, &profile);
    });

    let result: UserProfile = client.get_user_profile(&user_address);
    assert_eq!(result.did_hash, None);
    assert_eq!(result.off_chain_ref_id, String::from_str(&env, "usr-no-did"));
}
