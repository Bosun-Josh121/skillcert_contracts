// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use crate::{schema::Category, CourseRegistry, CourseRegistryClient};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String, Vec};

use crate::{
    functions::{
        get_prerequisites_by_course::get_prerequisites_by_course,
        list_categories::list_categories,
    },
    schema::Course,
};

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
    
    // Register mock user management contract
    let user_mgmt_id = env.register(mock_user_management::UserManagement, ());
    
    let contract_id = env.register(CourseRegistry, ());
    let client = CourseRegistryClient::new(&env, &contract_id);

    // Setup admin
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
    let new_module = client.add_module(&creator, &course.id, &0, &String::from_str(&env, "module_content_hash_1"));

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
fn test_remove_multiple_different_modules() {
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
    let module1 = client.add_module(&creator, &course.id, &0, &String::from_str(&env, "module_hash_1"));
    let module2 = client.add_module(&creator, &course.id, &1, &String::from_str(&env, "module_hash_2"));

    client.remove_module(&module1.id.clone());
    client.remove_module(&module2.id.clone());

    let module1_exists: bool = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .has(&(symbol_short!("module"), module1.id.clone()))
    });
    assert!(!module1_exists);

    let module2_exists: bool = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .has(&(symbol_short!("module"), module2.id.clone()))
    });
    assert!(!module2_exists);
}

#[test]
fn test_remove_module_storage_isolation() {
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
    let module1 = client.add_module(&creator, &course.id, &0, &String::from_str(&env, "module_hash_1"));
    let module2 = client.add_module(&creator, &course.id, &1, &String::from_str(&env, "module_hash_2"));

    client.remove_module(&module1.id.clone());

    let module1_exists: bool = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .has(&(symbol_short!("module"), module1.id.clone()))
    });
    assert!(!module1_exists);

    let module2_exists: bool = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .has(&(symbol_short!("module"), module2.id.clone()))
    });
    assert!(module2_exists);
}

#[test]
fn test_get_course_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CourseRegistry, ());
    let client = CourseRegistryClient::new(&env, &contract_id);

    let creator: Address = Address::generate(&env);
    let course = client.create_course(
        &creator,
        &String::from_str(&env, "ref-001"),
        &String::from_str(&env, "abc123hash"),
        &crate::schema::DEFAULT_COURSE_PRICE,
        &None,
        &None,
        &None,
        &None,
    );

    let retrieved = client.get_course(&course.id);

    assert_eq!(retrieved.id, course.id);
    assert_eq!(retrieved.off_chain_ref_id, course.off_chain_ref_id);
    assert_eq!(retrieved.content_hash, course.content_hash);
    assert_eq!(retrieved.creator, course.creator);
    assert_eq!(retrieved.published, course.published);
}

#[test]
#[should_panic(expected = "Course not found")]
fn test_get_course_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CourseRegistry, {});
    let client = CourseRegistryClient::new(&env, &contract_id);

    let fake_id = String::from_str(&env, "not_found");
    client.get_course(&fake_id);
}

#[test]
fn test_get_courses_by_instructor_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CourseRegistry, {});
    let client = CourseRegistryClient::new(&env, &contract_id);

    let courses = client.get_courses_by_instructor(&Address::generate(&env));

    assert_eq!(courses.len(), 0);
}

#[test]
fn test_get_courses_by_instructor_found() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CourseRegistry, {});
    let client = CourseRegistryClient::new(&env, &contract_id);

    let creator: Address = Address::generate(&env);
    let course = client.create_course(
        &creator,
        &String::from_str(&env, "ref-001"),
        &String::from_str(&env, "abc123hash"),
        &crate::schema::DEFAULT_COURSE_PRICE,
        &None,
        &None,
        &None,
        &None,
    );

    let results = client.get_courses_by_instructor(&creator);

    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().id, course.id);
}

#[test]
fn test_get_prerequisites_by_course_id() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CourseRegistry, {});
    let client = CourseRegistryClient::new(&env, &contract_id);

    let creator: Address = Address::generate(&env);
    let course = client.create_course(
        &creator,
        &String::from_str(&env, "ref-001"),
        &String::from_str(&env, "abc123hash"),
        &crate::schema::DEFAULT_COURSE_PRICE,
        &None,
        &None,
        &None,
        &None,
    );

    let prerequisites = env.as_contract(&contract_id, || {
        get_prerequisites_by_course(&env, course.id.clone())
    });
    assert!(prerequisites.is_empty());
}

#[test]
fn test_list_categories_counts() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CourseRegistry, ());
    let client = CourseRegistryClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    client.create_course(
        &creator,
        &String::from_str(&env, "ref-A"),
        &String::from_str(&env, "hashA"),
        &10,
        &Some(String::from_str(&env, "Programming")),
        &None,
        &None,
        &None,
    );
    client.create_course(
        &creator,
        &String::from_str(&env, "ref-B"),
        &String::from_str(&env, "hashB"),
        &10,
        &Some(String::from_str(&env, "Data")),
        &None,
        &None,
        &None,
    );
    client.create_course(
        &creator,
        &String::from_str(&env, "ref-C"),
        &String::from_str(&env, "hashC"),
        &10,
        &Some(String::from_str(&env, "Programming")),
        &None,
        &None,
        &None,
    );

    // Call the function to list categories
    let cats = client.list_categories();
    assert_eq!(cats.len(), 2); // Should have 2 unique categories

    // Verify counts without assuming order
    let mut prog = 0u128;
    let mut data = 0u128;
    for c in cats.iter() {
        let cname = c.name.clone();
        if cname == String::from_str(&env, "Programming") {
            prog = c.count;
        }
        if cname == String::from_str(&env, "Data") {
            data = c.count;
        }
    }
    assert_eq!(prog, 2);
    assert_eq!(data, 1);
}

#[test]
fn test_list_categories_empty() {
    let env: Env = Env::default();
    let contract_id: Address = env.register(CourseRegistry, ());
    // No courses created, should return empty vector
    let cats: Vec<Category> = env.as_contract(&contract_id, || list_categories(&env));
    assert_eq!(cats.len(), 0);
}

#[test]
fn test_list_categories_ignores_none() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let contract_id: Address = env.register(CourseRegistry, ());
    let client = CourseRegistryClient::new(&env, &contract_id);
    let creator: Address = Address::generate(&env);

    client.create_course(
        &creator,
        &String::from_str(&env, "ref-B"),
        &String::from_str(&env, "hashB"),
        &10,
        &Some(String::from_str(&env, "Programming")),
        &None,
        &None,
        &None,
    );

    let cats = client.list_categories();
    assert_eq!(cats.len(), 1); // Only "Programming" should be returned
    let c = cats.get(0).unwrap();
    assert_eq!(c.name, String::from_str(&env, "Programming"));
    assert_eq!(c.count, 1);
}

#[test]
fn test_list_categories_with_id_gaps() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CourseRegistry, ());
    let client = CourseRegistryClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    client.create_course(
        &creator,
        &String::from_str(&env, "ref-1"),
        &String::from_str(&env, "hash1"),
        &10,
        &Some(String::from_str(&env, "Programming")),
        &None,
        &None,
        &None,
    );
    client.create_course(
        &creator,
        &String::from_str(&env, "ref-2"),
        &String::from_str(&env, "hash2"),
        &10,
        &Some(String::from_str(&env, "Data")),
        &None,
        &None,
        &None,
    );

    // Manually delete course 2 to create an ID gap
    env.as_contract(&contract_id, || {
        let key = (symbol_short!("course"), String::from_str(&env, "2"));
        env.storage().persistent().remove(&key);
    });

    // Create course 3 (this will still have ID 3)
    client.create_course(
        &creator,
        &String::from_str(&env, "ref-3"),
        &String::from_str(&env, "hash3"),
        &10,
        &Some(String::from_str(&env, "Programming")),
        &None,
        &None,
        &None,
    );

    // Call the function - it should skip missing ID 2 but still count 1 and 3
    let cats = client.list_categories();
    let mut prog = 0u128;
    let mut data = 0u128;
    for c in cats.iter() {
        if c.name == String::from_str(&env, "Programming") {
            prog = c.count;
        }
        if c.name == String::from_str(&env, "Data") {
            data = c.count;
        }
    }
    assert_eq!(prog, 2); // Course 1 and Course 3
    assert_eq!(data, 0); // Course 2 was deleted
}

#[test]
fn test_course_backup_and_recovery_system() {
    let env = Env::default();
    let contract_id: Address = env.register(CourseRegistry, {});
    let client = CourseRegistryClient::new(&env, &contract_id);
    
    let admin: Address = Address::generate(&env);
    let instructor: Address = Address::generate(&env);
    
    env.mock_all_auths();

    // Create test courses
    let _course1 = client.create_course(
        &instructor,
        &String::from_str(&env, "ref-rust-prog"),
        &String::from_str(&env, "hash_rust_basics"),
        &1000_u128,
        &Some(String::from_str(&env, "Programming")),
        &None,
        &None,
        &None,
    );

    let _course2 = client.create_course(
        &instructor,
        &String::from_str(&env, "ref-adv-rust"),
        &String::from_str(&env, "hash_adv_rust"),
        &1500_u128,
        &Some(String::from_str(&env, "Programming")),
        &None,
        &None,
        &None,
    );

    // Set up admin first (add to admin list) - use contract context
    env.as_contract(&contract_id, || {
        let mut admin_list = Vec::new(&env);
        admin_list.push_back(admin.clone());
        env.storage()
            .persistent()
            .set(&crate::schema::DataKey::Admins, &admin_list);
    });

    // Test backup export
    let backup_data = client.export_course_data(&admin);
    assert!(backup_data.courses.len() >= 2);
    
    // Test backup data structure
    assert!(backup_data.courses.len() >= 2);
    
    // Test backup import (this would overwrite existing data)
    let imported_count = client.import_course_data(&admin, &backup_data);
    assert!(imported_count >= 2);
}
