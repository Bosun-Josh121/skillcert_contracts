// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Address, Env, Map, String, Symbol, Vec};

use crate::error::{handle_error, Error};
use crate::schema::{Course, DataKey};

const COURSE_KEY: Symbol = symbol_short!("course");

const PREREQ_UPDATED_EVENT: Symbol = symbol_short!("preqEdit");

pub fn edit_prerequisite(
    env: Env,
    creator: Address,
    course_id: String,
    new_prerequisites: Vec<String>,
) {
    creator.require_auth();

    // Load course to verify it exists and check authorization
    let course_key: (Symbol, String) = (COURSE_KEY, course_id.clone());
    let course: Course = env
        .storage()
        .persistent()
        .get(&course_key)
        .expect("Course not found");

    // Authorization: only creator can edit prerequisites
    if course.creator != creator {
        handle_error(&env, Error::Unauthorized)
    }

    // Validate that all prerequisite courses exist
    for prerequisite_id in new_prerequisites.iter() {
        let prereq_course_key: (Symbol, String) = (COURSE_KEY, prerequisite_id.clone());
        if !env.storage().persistent().has(&prereq_course_key) {
            handle_error(&env, Error::PrereqCourseNotFound)
        }
    }

    // Validate no duplicate prerequisites
    validate_no_duplicate_prerequisites(&env, &new_prerequisites);

    // Prevent circular dependencies
    validate_no_circular_dependency(&env, &course_id, &new_prerequisites);

    // Save updated prerequisites
    env.storage().persistent().set(
        &DataKey::CoursePrerequisites(course_id.clone()),
        &new_prerequisites,
    );

    // Emit event
    env.events().publish(
        (PREREQ_UPDATED_EVENT, course_id),
        new_prerequisites,
    );
}

fn validate_no_circular_dependency(env: &Env, course_id: &String, new_prerequisites: &Vec<String>) {
    // Check if course_id appears in new_prerequisites (direct circular dependency)
    for prerequisite_id in new_prerequisites.iter() {
        if prerequisite_id.eq(course_id) {
            handle_error(env, Error::SelfPrerequisite)
        }
    }

    // Check for indirect circular dependencies using DFS
    let mut visited: Map<String, bool> = Map::new(env);
    let mut rec_stack: Map<String, bool> = Map::new(env);

    for prerequisite_id in new_prerequisites.iter() {
        if has_cycle(
            env,
            &prerequisite_id,
            course_id,
            &mut visited,
            &mut rec_stack,
        ) {
            handle_error(env, Error::CircularDependency);
        }
    }
}

fn has_cycle(
    env: &Env,
    current_course: &String,
    target_course: &String,
    visited: &mut Map<String, bool>,
    rec_stack: &mut Map<String, bool>,
) -> bool {
    // If we've reached the target course, we found a cycle
    if current_course.eq(target_course) {
        return true;
    }

    // If already in recursion stack, we have a cycle
    if rec_stack.contains_key(current_course.clone()) {
        return true;
    }

    // If already visited and not in recursion stack, no cycle from this path
    if visited.contains_key(current_course.clone()) {
        return false;
    }

    // Mark as visited and add to recursion stack
    visited.set(current_course.clone(), true);
    rec_stack.set(current_course.clone(), true);

    // Get prerequisites for current course
    let prerequisites: Vec<String> = env
        .storage()
        .persistent()
        .get(&DataKey::CoursePrerequisites(current_course.clone()))
        .unwrap_or(Vec::new(env));

    // Recursively check all prerequisites
    for prerequisite in prerequisites.iter() {
        if has_cycle(env, &prerequisite, target_course, visited, rec_stack) {
            return true;
        }
    }

    // Remove from recursion stack before returning
    rec_stack.remove(current_course.clone());
    false
}

/// Validates that there are no duplicate prerequisites in the list
fn validate_no_duplicate_prerequisites(env: &Env, prerequisites: &Vec<String>) {
    let mut seen = Map::new(env);
    
    for prerequisite_id in prerequisites.iter() {
        if seen.contains_key(prerequisite_id.clone()) {
            handle_error(&env, Error::DuplicatePrerequisite);
        }
        seen.set(prerequisite_id.clone(), true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CourseRegistry;
    use crate::CourseRegistryClient;
    use soroban_sdk::{
        testutils::{Address as TestAddress, Events},
        Address, Env, String, Val,
    };

    #[test]
    fn test_edit_prerequisite_success() {
        let env: Env = Env::default();
        env.mock_all_auths();

        let contract_id: Address = env.register(CourseRegistry, ());
        let client: CourseRegistryClient<'_> = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let course1: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
        let course3 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-003"),
            &String::from_str(&env, "hash003"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );

        let mut prerequisites: Vec<String> = Vec::new(&env);
        prerequisites.push_back(course2.id.clone());
        prerequisites.push_back(course3.id.clone());

        client.edit_prerequisite(&creator, &course1.id.clone(), &prerequisites.clone());

        let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
        assert!(!events.is_empty());

        let stored_prerequisites: Vec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id.clone()))
                .unwrap()
        });

        assert_eq!(stored_prerequisites.len(), 2);
        assert_eq!(stored_prerequisites.get(0).unwrap(), course2.id);
        assert_eq!(stored_prerequisites.get(1).unwrap(), course3.id);
    }

    #[test]
    fn test_edit_prerequisite_replace_existing() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);
        
        // Initialize course rate limiting with permissive settings for testing
        env.as_contract(&contract_id, || {
            use crate::schema::{DataKey, CourseRateLimitConfig};
            let permissive_config = CourseRateLimitConfig {
                window_seconds: 3600,
                max_courses_per_window: 100,
            };
            let config_key = DataKey::CourseRateLimitConfig;
            env.storage().persistent().set(&config_key, &permissive_config);
        });

        let creator: Address = Address::generate(&env);
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
        let course3 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-003"),
            &String::from_str(&env, "hash003"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
        let course4 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-004"),
            &String::from_str(&env, "hash004"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );

        let mut initial_prerequisites = Vec::new(&env);
        initial_prerequisites.push_back(course2.id.clone());
        client.edit_prerequisite(&creator, &course1.id, &initial_prerequisites);

        let mut new_prerequisites = Vec::new(&env);
        new_prerequisites.push_back(course3.id.clone());
        new_prerequisites.push_back(course4.id.clone());
        client.edit_prerequisite(&creator, &course1.id, &new_prerequisites);

        let stored_prerequisites: Vec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id.clone()))
                .unwrap()
        });

        assert_eq!(stored_prerequisites.len(), 2);
        assert_eq!(stored_prerequisites.get(0).unwrap(), course3.id);
        assert_eq!(stored_prerequisites.get(1).unwrap(), course4.id);
    }

    #[test]
    fn test_edit_prerequisite_empty_list() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );

        let mut initial_prerequisites = Vec::new(&env);
        initial_prerequisites.push_back(course2.id.clone());
        client.edit_prerequisite(&creator, &course1.id, &initial_prerequisites);

        let empty_prerequisites = Vec::new(&env);
        client.edit_prerequisite(&creator, &course1.id.clone(), &empty_prerequisites);

        let stored_prerequisites: Vec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id.clone()))
                .unwrap()
        });

        assert_eq!(stored_prerequisites.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Course not found")]
    fn test_edit_prerequisite_course_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        client.edit_prerequisite(
            &Address::generate(&env),
            &String::from_str(&env, "404"),
            &Vec::new(&env),
        );
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #13)")]
    fn test_edit_prerequisite_invalid_prerequisite() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );

        let mut prerequisites = Vec::new(&env);
        prerequisites.push_back(String::from_str(&env, "404"));

        client.edit_prerequisite(&creator, &course1.id, &prerequisites);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #14)")]
    fn test_edit_prerequisite_direct_circular_dependency() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );

        let mut prerequisites = Vec::new(&env);
        prerequisites.push_back(course1.id.clone());

        client.edit_prerequisite(&creator, &course1.id, &prerequisites);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #15)")]
    fn test_edit_prerequisite_indirect_circular_dependency() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );
        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
        let course3 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-003"),
            &String::from_str(&env, "hash003"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );

        let mut prerequisites2 = Vec::new(&env);
        prerequisites2.push_back(course3.id.clone());
        client.edit_prerequisite(&creator, &course2.id, &prerequisites2);

        let mut prerequisites1 = Vec::new(&env);
        prerequisites1.push_back(course2.id.clone());
        client.edit_prerequisite(&creator, &course1.id, &prerequisites1);

        let mut prerequisites3 = Vec::new(&env);
        prerequisites3.push_back(course1.id.clone());
        client.edit_prerequisite(&creator, &course3.id, &prerequisites3);
    }

    #[test]
    fn test_edit_prerequisite_authorization() {
        let env = Env::default();

        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );
        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );

        let mut prerequisites = Vec::new(&env);
        prerequisites.push_back(course2.id.clone());

        client.edit_prerequisite(&creator, &course1.id, &prerequisites);

        let stored_prerequisites: Vec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id))
                .unwrap()
        });
        assert_eq!(stored_prerequisites.len(), 1);
    }

    #[test]
    fn test_edit_prerequisite_complex_chain() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        // Initialize course rate limiting with permissive settings for testing
        env.as_contract(&contract_id, || {
            use crate::schema::{DataKey, CourseRateLimitConfig};
            let permissive_config = CourseRateLimitConfig {
                window_seconds: 3600,
                max_courses_per_window: 100,
            };
            let config_key = DataKey::CourseRateLimitConfig;
            env.storage().persistent().set(&config_key, &permissive_config);
        });

        let creator: Address = Address::generate(&env);
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );
        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );
        let course3 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-003"),
            &String::from_str(&env, "hash003"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );
        let course4 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-004"),
            &String::from_str(&env, "hash004"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
        let course5 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-005"),
            &String::from_str(&env, "hash005"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );

        let mut prerequisites2 = Vec::new(&env);
        prerequisites2.push_back(course4.id.clone());
        client.edit_prerequisite(&creator, &course2.id, &prerequisites2);

        let mut prerequisites3 = Vec::new(&env);
        prerequisites3.push_back(course5.id.clone());
        client.edit_prerequisite(&creator, &course3.id, &prerequisites3);

        let mut prerequisites1 = Vec::new(&env);
        prerequisites1.push_back(course2.id.clone());
        prerequisites1.push_back(course3.id.clone());
        client.edit_prerequisite(&creator, &course1.id, &prerequisites1);

        let stored_prerequisites: Vec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id))
                .unwrap()
        });
        assert_eq!(stored_prerequisites.len(), 2);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #56)")]
    fn test_edit_prerequisite_duplicate_validation() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );

        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );

        // Try to edit with duplicate prerequisites
        let mut prerequisites = Vec::new(&env);
        prerequisites.push_back(course2.id.clone());
        prerequisites.push_back(course2.id.clone()); // Duplicate

        client.edit_prerequisite(&creator, &course1.id, &prerequisites);
    }

    #[test]
    fn test_edit_prerequisite_no_duplicates_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );

        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );

        let course3 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-003"),
            &String::from_str(&env, "hash003"),
            &1000,
            &None,
            &None,
            &None,
            &None,
        );

        // Edit with unique prerequisites
        let mut prerequisites = Vec::new(&env);
        prerequisites.push_back(course2.id.clone());
        prerequisites.push_back(course3.id.clone());

        client.edit_prerequisite(&creator, &course1.id, &prerequisites);

        // Verify prerequisites were saved correctly
        let stored_prerequisites: Vec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id.clone()))
                .unwrap()
        });

        assert_eq!(stored_prerequisites.len(), 2);
        assert_eq!(stored_prerequisites.get(0).unwrap(), course2.id);
        assert_eq!(stored_prerequisites.get(1).unwrap(), course3.id);
    }
}
