// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Address, Env, String, Symbol, Vec};

use crate::error::{handle_error, Error};
use crate::schema::{Course, DataKey};

const COURSE_KEY: Symbol = symbol_short!("course");

const PREREQ_REMOVED_EVENT: Symbol = symbol_short!("prereqRmv");

pub fn remove_prerequisite(
    env: Env,
    creator: Address,
    course_id: String,
    prerequisite_course_id: String,
) {
    creator.require_auth();

    // Load course
    let course_key: (Symbol, String) = (COURSE_KEY, course_id.clone());
    let course: Course = env
        .storage()
        .persistent()
        .get(&course_key)
        .expect("Course not found");

    // Authorization: only creator can remove prerequisites
    if course.creator != creator {
        handle_error(&env, Error::Unauthorized)
    }

    // Load current list of prerequisites
    let mut prerequisites: Vec<String> = env
        .storage()
        .persistent()
        .get(&DataKey::CoursePrerequisites(course_id.clone()))
        .unwrap_or(Vec::new(&env));

    // Find and remove the prerequisite
    let index: Option<usize> = prerequisites
        .iter()
        .position(|id| id.eq(&prerequisite_course_id));

    match index {
        Some(i) => {
            prerequisites.remove(i as u32);
        }
        None => handle_error(&env, Error::PrereqNotInList),
    }

    // Save updated prerequisites
    env.storage().persistent().set(
        &DataKey::CoursePrerequisites(course_id.clone()),
        &prerequisites,
    );

    // Emits an event for successful prerequisite removal.
    env.events()
        .publish((PREREQ_REMOVED_EVENT, course_id), prerequisite_course_id);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::schema::Course;
    use crate::{CourseRegistry, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec as SdkVec};

    #[test]
    fn test_remove_prerequisite_success() {
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
            &None,
            &None,
            &None,
            &None,
        );

        let course2: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        let prerequisites = SdkVec::from_array(&env, [course2.id.clone()]);
        client.edit_prerequisite(&creator, &course1.id, &prerequisites);

        let stored_prerequisites: SdkVec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id.clone()))
                .unwrap_or(SdkVec::new(&env))
        });
        assert_eq!(stored_prerequisites.len(), 1);
        assert_eq!(stored_prerequisites.get(0).unwrap(), course2.id);

        client.remove_prerequisite(&creator, &course1.id, &course2.id);

        let stored_prerequisites: SdkVec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id.clone()))
                .unwrap_or(SdkVec::new(&env))
        });
        assert_eq!(stored_prerequisites.len(), 0);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_remove_prerequisite_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let impostor: Address = Address::generate(&env);

        let course1: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "hash001"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        let course2: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        let prerequisites = SdkVec::from_array(&env, [course2.id.clone()]);
        client.edit_prerequisite(&creator, &course1.id, &prerequisites);

        client.remove_prerequisite(&impostor, &course1.id, &course2.id);
    }

    #[test]
    #[should_panic(expected = "Course not found")]
    fn test_remove_prerequisite_course_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let fake_course_id = String::from_str(&env, "nonexistent_course");
        let fake_prereq_id = String::from_str(&env, "nonexistent_prereq");

        client.remove_prerequisite(&creator, &fake_course_id, &fake_prereq_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #23)")]
    fn test_remove_prerequisite_not_in_list() {
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
            &None,
            &None,
            &None,
            &None,
        );

        let course2: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        client.remove_prerequisite(&creator, &course1.id, &course2.id);
    }

    #[test]
    fn test_remove_one_of_multiple_prerequisites() {
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
            &None,
            &None,
            &None,
            &None,
        );

        let course2: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-002"),
            &String::from_str(&env, "hash002"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        let course3: Course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-003"),
            &String::from_str(&env, "hash003"),
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        let prerequisites = SdkVec::from_array(&env, [course2.id.clone(), course3.id.clone()]);
        client.edit_prerequisite(&creator, &course1.id, &prerequisites);

        let stored_prerequisites: SdkVec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id.clone()))
                .unwrap_or(SdkVec::new(&env))
        });
        assert_eq!(stored_prerequisites.len(), 2);
        assert_eq!(stored_prerequisites.get(0).unwrap(), course2.id);
        assert_eq!(stored_prerequisites.get(1).unwrap(), course3.id);

        client.remove_prerequisite(&creator, &course1.id, &course2.id);

        let stored_prerequisites: SdkVec<String> = env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .get(&DataKey::CoursePrerequisites(course1.id.clone()))
                .unwrap_or(SdkVec::new(&env))
        });
        assert_eq!(stored_prerequisites.len(), 1);
        assert_eq!(stored_prerequisites.get(0).unwrap(), course3.id);
    }
}
