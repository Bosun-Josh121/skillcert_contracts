// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

use crate::error::{handle_error, Error};
use crate::schema::Course;

const COURSE_KEY: Symbol = symbol_short!("course");

const ARCHIVED_COURSE_EVENT: Symbol = symbol_short!("archiveCs");

pub fn archive_course(env: &Env, creator: Address, course_id: String) -> Course {
    creator.require_auth();

    let key: (Symbol, String) = (COURSE_KEY, course_id.clone());
    let mut course: Course = env
        .storage()
        .persistent()
        .get(&key)
        .expect("Course not found");

    if course.creator != creator {
        handle_error(env, Error::OnlyCreatorCanArchive)
    }

    if course.is_archived {
        handle_error(env, Error::CourseAlreadyArchived)
    }
    course.is_archived = true;

    env.storage().persistent().set(&key, &course);
    
    env.events()
        .publish((ARCHIVED_COURSE_EVENT, course_id.clone()), course.clone());

    course
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CourseRegistry, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, testutils::Events, Address, Env};

    #[test]
    fn test_archive_course_success() {
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

        let archived_new_course = client.archive_course(&creator, &new_course.id.clone());
        assert!(archived_new_course.is_archived);
    }

    #[test]
    #[should_panic(expected = "Course not found")]
    fn test_archive_nonexistent_course() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let fake_course_id = String::from_str(&env, "nonexistent_course");

        client.archive_course(&creator, &fake_course_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #4)")]
    fn test_archive_course_by_non_creator() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let non_creator: Address = Address::generate(&env);

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

        client.archive_course(&non_creator, &new_course.id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_archive_already_archived_course() {
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

        let archived_course = client.archive_course(&creator, &new_course.id);
        assert!(archived_course.is_archived);

        client.archive_course(&creator, &new_course.id);
    }

    #[test]
    fn test_archive_course_event_published() {
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

        client.archive_course(&creator, &new_course.id);

        let events = env.events().all();
        assert!(!events.is_empty());
    }
}
