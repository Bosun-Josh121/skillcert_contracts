// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, vec, Address, Env, String, Symbol, Vec};

use crate::error::{handle_error, Error};
use crate::schema::{Course, CourseModule};
use crate::functions::utils::{concat_strings, u32_to_string};

const COURSE_KEY: Symbol = symbol_short!("course");
const MODULE_KEY: Symbol = symbol_short!("module");

const DELETE_COURSE_EVENT: Symbol = symbol_short!("delCourse");

pub fn delete_course(env: &Env, creator: Address, course_id: String) -> Result<(), &'static str> {
    creator.require_auth();

    if course_id.is_empty() {
        handle_error(env, Error::EmptyCourseId)
    }

    let course_storage_key: (Symbol, String) = (COURSE_KEY, course_id.clone());

    if !env.storage().persistent().has(&course_storage_key) {
        handle_error(env, Error::CourseNotFound)
    }

    let course: Course = env
        .storage()
        .persistent()
        .get(&course_storage_key)
        .ok_or("Course not found")?;

    if course.creator != creator {
        handle_error(env, Error::Unauthorized)
    }

    delete_course_modules(env, &course_id);

    env.storage().persistent().remove(&course_storage_key);

    // emit an event
    env.events()
        .publish((DELETE_COURSE_EVENT,), (creator, course_id));

    Ok(())
}

fn delete_course_modules(env: &Env, course_id: &String) {
    let mut modules_to_delete: Vec<String> = Vec::new(env);

    let mut counter = 0u32;
    loop {
        let arr = vec![
            &env,
            String::from_str(env, "module_"),
            course_id.clone(),
            String::from_str(env, "_"),
            u32_to_string(env, counter),
            String::from_str(env, "_0"),
        ];

        let module_id = concat_strings(env, arr);
        let key = (MODULE_KEY, module_id.clone());
        if env.storage().persistent().has(&key) {
            if let Some(module) = env.storage().persistent().get::<_, CourseModule>(&key) {
                if module.course_id == *course_id {
                    modules_to_delete.push_back(module_id);
                }
            }
        } else {
            break;
        }
        counter += 1;
        if counter > crate::schema::MAX_LOOP_GUARD {
            break;
        }
    }

    for id in modules_to_delete.iter() {
        env.storage().persistent().remove(&(MODULE_KEY, id.clone()));
        env.events().publish((id.clone(),), "module_deleted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Course;
    use crate::{CourseRegistry, CourseRegistryClient};
    use soroban_sdk::testutils::Address as _;

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
    use soroban_sdk::{Env, String};

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_delete_course_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let impostor: Address = Address::generate(&env);

        let new_course: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "abc123hash"),
            &1000_u128,
            &Some(String::from_str(&env, "category")),
            &Some(String::from_str(&env, "language")),
            &None,
            &None,
        );

        client.delete_course(&impostor, &new_course.id.clone());
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_impostor_cannot_delete_course() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let actual_creator: Address = Address::generate(&env);
        let someone_else: Address = Address::generate(&env);

        let course: Course = client.create_course(
            &actual_creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "abc123hash"),
            &500_u128,
            &Some(String::from_str(&env, "security")),
            &Some(String::from_str(&env, "english")),
            &None,
            &None,
        );

        let retrieved_course = client.get_course(&course.id);
        assert_eq!(retrieved_course.creator, actual_creator);

        client.delete_course(&someone_else, &course.id);
    }

    #[test]
    fn test_delete_course_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);

        let new_course: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "abc123hash"),
            &1000_u128,
            &Some(String::from_str(&env, "category")),
            &Some(String::from_str(&env, "language")),
            &None,
            &None,
        );

        assert_eq!(new_course, client.get_course(&new_course.id.clone()));
        assert_eq!(
            new_course.creator,
            client.get_course(&new_course.id.clone()).creator
        );

        client.delete_course(&creator, &new_course.id.clone());

        let exists: bool = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .has(&(COURSE_KEY, new_course.id.clone()))
        });
        assert!(!exists);
    }

    #[test]
    fn test_delete_course_with_modules() {
        let (env, contract_id, client) = setup_test_env();

        let creator: Address = Address::generate(&env);

        let new_course: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "abc123hash"),
            &1000_u128,
            &Some(String::from_str(&env, "category")),
            &Some(String::from_str(&env, "language")),
            &None,
            &None,
        );

        let module = client.add_module(
            &creator,
            &new_course.id,
            &0,
            &String::from_str(&env, "module_content_hash_001"),
        );

        let module_exists: bool = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .has(&(MODULE_KEY, module.id.clone()))
        });
        assert!(module_exists);

        client.delete_course(&creator, &new_course.id.clone());

        let course_exists: bool = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .has(&(COURSE_KEY, new_course.id.clone()))
        });
        assert!(!course_exists);

        let module_exists: bool = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .has(&(MODULE_KEY, module.id.clone()))
        });
        assert!(!module_exists);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #17)")]
    fn test_delete_course_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let fake_id = String::from_str(&env, "not_found");

        client.delete_course(&creator, &fake_id);
    }

    #[test]
    fn test_delete_course_preserves_others() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);

        let course1: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000_u128,
            &Some(String::from_str(&env, "category1")),
            &Some(String::from_str(&env, "language1")),
            &None,
            &None,
        );

        let course2: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000_u128,
            &Some(String::from_str(&env, "category2")),
            &Some(String::from_str(&env, "language2")),
            &None,
            &None,
        );

        client.delete_course(&creator, &course1.id.clone());

        let course1_exists: bool = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .has(&(COURSE_KEY, course1.id.clone()))
        });
        assert!(!course1_exists);

        let course2_exists: bool = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .has(&(COURSE_KEY, course2.id.clone()))
        });
        assert!(course2_exists);
    }
}
