// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Address, Env, Map, String, Symbol, Vec};

use crate::error::{handle_error, Error};
use crate::schema::{Course, DataKey};

const COURSE_KEY: Symbol = symbol_short!("course");

const PREREQ_CREATED_EVENT: Symbol = symbol_short!("prereqAdd");

pub fn add_prerequisite(env: Env, creator: Address, course_id: String, prerequisites: Vec<String>) {
    creator.require_auth();

    // Validate input parameters
    if course_id.is_empty() {
        handle_error(&env, Error::EmptyCourseId);
    }

    if course_id.len() > 100 {
        handle_error(&env, Error::EmptyCourseId);
    }

    // Validate prerequisites list
    if prerequisites.is_empty() {
        handle_error(&env, Error::EmptyPrerequisiteList);
    }

    // Check for reasonable limit on number of prerequisites
    if prerequisites.len() > 20 {
        handle_error(&env, Error::TooManyPrerequisites);
    }

    // Validate each prerequisite ID
    for prerequisite_id in prerequisites.iter() {
        if prerequisite_id.is_empty() {
            handle_error(&env, Error::EmptyPrerequisiteId);
        }

        if prerequisite_id.len() > 100 {
            handle_error(&env, Error::InvalidPrerequisiteId);
        }

        // Check for self-prerequisite
        if prerequisite_id == course_id {
            handle_error(&env, Error::SelfPrerequisite);
        }
    }

    let course_key: (Symbol, String) = (COURSE_KEY, course_id.clone());
    let course: Course = env
        .storage()
        .persistent()
        .get(&course_key)
        .expect("Course not found");

    if course.creator != creator {
        handle_error(&env, Error::OnlyCreatorCanEditPrereqs)
    }

    for prerequisite_id in prerequisites.iter() {
        let prereq_course_key: (Symbol, String) =
            (COURSE_KEY, prerequisite_id.clone());
        if !env.storage().persistent().has(&prereq_course_key) {
            handle_error(&env, Error::PrereqCourseNotFound)
        }
    }

    // Validate no duplicate prerequisites
    validate_no_duplicate_prerequisites(&env, &prerequisites);

    validate_no_circular_dependency(&env, &course_id, &prerequisites);

    env.storage().persistent().set(
        &DataKey::CoursePrerequisites(course_id.clone()),
        &prerequisites,
    );

    env.events()
        .publish((PREREQ_CREATED_EVENT, course_id), prerequisites.len());
}

fn validate_no_circular_dependency(env: &Env, course_id: &String, new_prerequisites: &Vec<String>) {
    // Check if course_id appears in new_prerequisites (direct circular dependency)
    for prerequisite_id in new_prerequisites.iter() {
        if prerequisite_id.eq(course_id) {
            handle_error(env, Error::SelfPrerequisite)
        }
    }

    // Check for indirect circular dependencies using DFS
    let mut visited = Map::new(env);
    let mut rec_stack = Map::new(env);

    for prerequisite_id in new_prerequisites.iter() {
        if has_cycle(
            env,
            &prerequisite_id,
            course_id,
            &mut visited,
            &mut rec_stack,
        ) {
            handle_error(env, Error::CircularDependency)
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
        testutils::{Address as TestAddress},
        Address, Env, String, Vec as SdkVec,
    };

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #56)")]
    fn test_add_prerequisite_duplicate_validation() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        // Create prerequisites with duplicate course2.id
        let mut prerequisites = SdkVec::new(&env);
        prerequisites.push_back(course2.id.clone());
        prerequisites.push_back(course2.id.clone()); // Duplicate

        client.add_prerequisite(&creator, &course1.id, &prerequisites);
    }

    #[test]
    fn test_add_prerequisite_no_duplicates_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        
        let course1 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        let course2 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        let course3 = client.create_course(
            &creator,
            &String::from_str(&env, "ref-003"),
            &String::from_str(&env, "hash003"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        // Create prerequisites without duplicates
        let mut prerequisites = SdkVec::new(&env);
        prerequisites.push_back(course2.id.clone());
        prerequisites.push_back(course3.id.clone());

        client.add_prerequisite(&creator, &course1.id, &prerequisites);

        // Verify prerequisites were saved correctly
        let stored_prerequisites: SdkVec<String> = env.as_contract(&contract_id, || {
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
