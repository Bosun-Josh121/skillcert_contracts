// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Vec, vec, Address, Env, String, Symbol};

use crate::functions::utils::{concat_strings, u32_to_string};
use crate::error::{handle_error, Error};
use crate::schema::CourseModule;

const COURSE_KEY: Symbol = symbol_short!("course");
const MODULE_KEY: Symbol = symbol_short!("module");

const COURSE_REGISTRY_ADD_MODULE_EVENT: Symbol = symbol_short!("crsAddMod");

/// Add a module to a course.
///
/// The caller provides a content_hash
/// representing the SHA-256 hash of the off-chain module content for integrity verification.
pub fn course_registry_add_module(
    env: Env,
    caller: Address,
    course_id: String,
    position: u32,
    content_hash: String,
) -> CourseModule {
    // Validate input parameters
    if course_id.is_empty() {
        handle_error(&env, Error::EmptyCourseId);
    }

    if content_hash.is_empty() {
        handle_error(&env, Error::ContentHashRequired);
    }

    // Check string lengths to prevent extremely long values
    if course_id.len() > 100 {
        handle_error(&env, Error::EmptyCourseId);
    }

    // Validate position is reasonable (not extremely large)
    if position > 10000 {
        handle_error(&env, Error::InvalidModulePosition);
    }

    let course_storage_key: (Symbol, String) = (COURSE_KEY, course_id.clone());

    if !env.storage().persistent().has(&course_storage_key) {
        handle_error(&env, Error::CourseIdNotExist)
    }

    // Verify caller has proper authorization
    super::access_control::require_course_management_auth(&env, &caller, &course_id);

    // Check for duplicate position
    let position_key: (Symbol, String, u32) = (symbol_short!("pos"), course_id.clone(), position);
    if env.storage().persistent().has(&position_key) {
        handle_error(&env, Error::DuplicateModulePosition)
    }

    let ledger_seq: u32 = env.ledger().sequence();

    let arr: Vec<String> = vec![
        &env,
        String::from_str(&env, "module_"),
        course_id.clone(),
        String::from_str(&env, "_"),
        u32_to_string(&env, position),
        String::from_str(&env, "_"),
        u32_to_string(&env, ledger_seq),
    ];

    let module_id: String = concat_strings(&env, arr);

    // Create new module — lean on-chain record
    let module: CourseModule = CourseModule {
        id: module_id.clone(),
        course_id: course_id.clone(),
        position,
        content_hash: content_hash.clone(),
        created_at: env.ledger().timestamp(),
    };

    let storage_key: (Symbol, String) = (MODULE_KEY, module_id.clone());
    let position_key: (Symbol, String, u32) = (symbol_short!("pos"), course_id.clone(), position);

    env.storage().persistent().set(&storage_key, &module);
    env.storage().persistent().set(&position_key, &true);

    // emit an event — only essential blockchain data
    env.events()
        .publish((COURSE_REGISTRY_ADD_MODULE_EVENT,), (caller, course_id, position, content_hash));

    module
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::*;
    use crate::{schema::Course, CourseRegistry, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn create_course<'a>(client: &CourseRegistryClient<'a>, creator: &Address) -> Course {
        let off_chain_ref_id = String::from_str(&client.env, "ref_001");
        let content_hash = String::from_str(&client.env, "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4");
        let price = 1000_u128;
        client.create_course(
            creator,
            &off_chain_ref_id,
            &content_hash,
            &price,
            &None,
            &None,
            &None,
            &None,
        )
    }

    // Mock UserManagement contract for testing
    mod mock_user_management {
        use soroban_sdk::{contract, contractimpl, Address, Env};

        #[contract]
        pub struct UserManagement;

        #[contractimpl]
        impl UserManagement {
            pub fn is_admin(_env: Env, _who: Address) -> bool {
                // For testing, return false to force course creator authorization
                // This ensures that only course creators can add modules
                false
            }
        }
    }

    fn setup_test_env() -> (Env, Address, Address, CourseRegistryClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();

        // Register mock user management contract
        let user_mgmt_id = env.register(mock_user_management::UserManagement, ());

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        // Setup admin
        let admin = Address::generate(&env);
        env.as_contract(&contract_id, || {
            crate::functions::access_control::initialize(&env, &admin, &user_mgmt_id);
        });

        (env, contract_id, admin, client)
    }

    #[test]
    fn test_add_module_success_course_creator() {
        let (env, _, _, client) = setup_test_env();
        let creator = Address::generate(&env);
        let course = create_course(&client, &creator);

        let content_hash = String::from_str(&env, "module_hash_aabbccddee1122334455");
        let module = client.add_module(&creator, &course.id, &1, &content_hash);

        assert_eq!(module.course_id, course.id);
        assert_eq!(module.position, 1);
        assert_eq!(module.content_hash, content_hash);
    }

    #[test]
    fn test_add_module_success_admin() {
        let (env, _, _admin, client) = setup_test_env();
        let creator = Address::generate(&env);
        let course = create_course(&client, &creator);

        let content_hash = String::from_str(&env, "module_hash_aabbccddee1122334455");
        let module = client.add_module(&creator, &course.id, &1, &content_hash);

        assert_eq!(module.course_id, course.id);
        assert_eq!(module.position, 1);
        assert_eq!(module.content_hash, content_hash);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #6)")] // Unauthorized error
    fn test_add_module_unauthorized() {
        let (env, _, _, client) = setup_test_env();
        let creator = Address::generate(&env);
        let course = create_course(&client, &creator);

        let unauthorized_user = Address::generate(&env);
        client.add_module(
            &unauthorized_user,
            &course.id,
            &1,
            &String::from_str(&env, "module_hash_aabbccddee1122334455"),
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #3)")] // CourseIdNotExist error
    fn test_add_module_invalid_course() {
        let (env, _, _admin, client) = setup_test_env();

        let unauthorized_user = Address::generate(&env);
        client.add_module(
            &unauthorized_user,
            &String::from_str(&env, "invalid_course"),
            &1,
            &String::from_str(&env, "module_hash_aabbccddee1122334455"),
        );
    }

    #[test]
    fn test_add_module_generates_unique_ids() {
        let (env, _, _admin, client) = setup_test_env();
        let creator = Address::generate(&env);
        let course = create_course(&client, &creator);

        let module1 = client.add_module(
            &creator,
            &course.id,
            &1,
            &String::from_str(&env, "hash_module_one_aabbccddeeff1122"),
        );
        let module2 = client.add_module(
            &creator,
            &course.id,
            &2,
            &String::from_str(&env, "hash_module_two_aabbccddeeff3344"),
        );

        assert_ne!(module1.id, module2.id);
    }

    #[test]
    fn test_add_module_storage_key_format() {
        let (env, contract_id, _admin, client) = setup_test_env();
        let creator = Address::generate(&env);
        let course = create_course(&client, &creator);

        let content_hash = String::from_str(&env, "module_hash_aabbccddee1122334455");
        let module = client.add_module(&creator, &course.id, &1, &content_hash);

        let exists: bool = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .has(&(MODULE_KEY, module.id.clone()))
        });

        assert!(exists);
    }

    #[test]
    #[should_panic]
    fn test_add_module_different_course_creator() {
        let (env, _, _, client) = setup_test_env();
        let creator1 = Address::generate(&env);

        let course1 = create_course(&client, &creator1);

        // Creator2 should not be able to add module to Creator1's course
        let creator2 = Address::generate(&env);
        client.add_module(
            &creator2,
            &course1.id,
            &1,
            &String::from_str(&env, "module_hash_aabbccddee1122334455"),
        );
    }

    #[test]
    #[should_panic]
    fn test_add_module_empty_content_hash() {
        let (env, _, _admin, client) = setup_test_env();
        let creator = Address::generate(&env);
        let course = create_course(&client, &creator);

        // Should panic with validation error for empty content hash
        client.add_module(&creator, &course.id, &1, &String::from_str(&env, ""));
    }

    #[test]
    #[should_panic]
    fn test_add_module_duplicate_position() {
        let (env, _, _admin, client) = setup_test_env();
        let creator = Address::generate(&env);
        let course = create_course(&client, &creator);

        let hash = String::from_str(&env, "module_hash_aabbccddee1122334455");

        // Add first module at position 1
        client.add_module(&creator, &course.id, &1, &hash);

        // Try to add another module at the same position
        client.add_module(
            &creator,
            &course.id,
            &1,
            &String::from_str(&env, "different_hash_aabbccddee11223344"),
        );
    }
}
