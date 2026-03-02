// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Env, String, Symbol};

use crate::error::{handle_error, Error};
use crate::schema::CourseModule;

const MODULE_KEY: Symbol = symbol_short!("module");

const REMOVE_MODULE_EVENT: Symbol = symbol_short!("remModule");

pub fn remove_module(env: &Env, module_id: String) -> Result<(), &'static str> {
    if module_id.is_empty() {
        handle_error(env, Error::EmptyModuleId)
    }

    // Try to get the module data to verify it exists and is a valid CourseModule
    let module: Option<CourseModule> = env
        .storage()
        .persistent()
        .get(&(MODULE_KEY, module_id.clone()));

    // Validate that the module exists and is a valid CourseModule
    if module.is_none() {
        handle_error(env, Error::ModuleNotFound)
    }

    // Delete the CourseModule directly from persistent storage using its key.
    env.storage()
        .persistent()
        .remove(&(MODULE_KEY, module_id.clone()));

    // Emits an event to indicate the module has been removed.
    env.events().publish((REMOVE_MODULE_EVENT,), module_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{schema::Course, CourseRegistry, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    // Mock UserManagement contract for testing
    mod mock_user_management {
        use soroban_sdk::{contract, contractimpl, Address, Env};

        #[contract]
        pub struct UserManagement;

        #[contractimpl]
        impl UserManagement {
            pub fn is_admin(_env: Env, _who: Address) -> bool {
                true
            }
        }
    }

    fn setup_test_env() -> (Env, Address, CourseRegistryClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();

        let user_mgmt_id = env.register(mock_user_management::UserManagement, ());

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        env.as_contract(&contract_id, || {
            crate::functions::access_control::initialize(&env, &admin, &user_mgmt_id);
        });

        (env, contract_id, client)
    }

    #[test]
    fn test_remove_module_success() {
        let (env, contract_id, client) = setup_test_env();

        let creator = Address::generate(&env);
        let course: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "abc123hash"),
            &1000_u128,
            &Some(String::from_str(&env, "category")),
            &Some(String::from_str(&env, "language")),
            &None,
            &None,
        );
        let new_module = client.add_module(
            &creator,
            &course.id,
            &0,
            &String::from_str(&env, "module_content_hash_001"),
        );

        let exists: bool = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .has(&(symbol_short!("module"), new_module.id.clone()))
        });
        assert!(exists);

        client.remove_module(&new_module.id);
        let exists: bool = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .has(&(symbol_short!("module"), new_module.id.clone()))
        });
        assert!(!exists);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #22)")]
    fn test_remove_module_with_empty_id() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        client.remove_module(&String::from_str(&env, ""));
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #21)")]
    fn test_remove_module_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        client.remove_module(&String::from_str(&env, "non_existent_module"));
    }
}

