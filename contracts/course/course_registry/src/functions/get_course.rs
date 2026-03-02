// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{Env, String, Symbol, symbol_short};

use crate::error::{handle_error, Error};
use crate::schema::Course;

const COURSE_KEY: Symbol = symbol_short!("course");

/// Retrieves a course by its ID.
///
/// Arguments:
/// - env: Soroban environment.
/// - course_id: unique identifier of the course.
///
/// Returns:
/// - Course: the course record associated with the given ID.
///
/// Errors:
/// - Panics with `"Course not found"` if the course does not exist.
/// - Returns `Error::CourseAlreadyArchived` if the course is archived.
///
/// Storage used (replace keys if your schema differs):
/// - (("course", id),) -> Course    // course record by id
pub fn get_course(env: &Env, course_id: String) -> Course {

    // Get the course from storage
    let course: Course = env
        .storage()
        .persistent()
        .get(&(COURSE_KEY, course_id.clone()))
        .expect("Course not found");

    match course.is_archived {
        true => handle_error(env, Error::CourseAlreadyArchived),
        false => course,
    }
}

#[cfg(test)]
mod test {
    use crate::{schema::Course, CourseRegistry, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_get_course_success() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let course = create_course(&client, &creator);
        let fetched_course = client.get_course(&course.id);
        assert_eq!(fetched_course, course);
    }

    #[test]
    #[should_panic(expected = "Course not found")]
    fn test_get_course_not_found() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        client.get_course(&String::from_str(&env, "1"));
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #5)")]
    fn test_get_archived_course() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator = Address::generate(&env);
        let course = create_course(&client, &creator);
        client.archive_course(&creator, &course.id);
        client.get_course(&course.id);
    }

    fn create_course<'a>(client: &CourseRegistryClient<'a>, creator: &Address) -> Course {
        let off_chain_ref_id = String::from_str(&client.env, "ref-001");
        let content_hash = String::from_str(&client.env, "abc123hash");
        let price = 1000_u128;
        client.create_course(
            &creator,
            &off_chain_ref_id,
            &content_hash,
            &price,
            &None,
            &None,
            &None,
            &None,
        )
    }
}
