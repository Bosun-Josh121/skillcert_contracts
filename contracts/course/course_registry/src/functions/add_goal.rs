// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

use crate::error::{handle_error, Error};
use crate::functions::utils;
use crate::schema::{Course, CourseGoal, DataKey};

const COURSE_KEY: Symbol = symbol_short!("course");

const GOAL_ADDED_EVENT: Symbol = symbol_short!("goalAdded");

/// Add a goal to a course.
///
/// The caller provides a content_hash
/// representing the SHA-256 hash of the off-chain goal content for integrity verification.
pub fn add_goal(env: Env, creator: Address, course_id: String, content_hash: String) -> CourseGoal {
    creator.require_auth();

    // Validate input parameters
    if course_id.is_empty() {
        handle_error(&env, Error::EmptyCourseId);
    }

    // Validate content hash is provided
    if content_hash.is_empty() {
        handle_error(&env, Error::EmptyGoalContent);
    }

    // Check string lengths to prevent extremely long values
    if course_id.len() > 100 {
        handle_error(&env, Error::InvalidCourseId);
    }

    // Load course
    let storage_key: (Symbol, String) = (COURSE_KEY, course_id.clone());
    let course: Course = env
        .storage()
        .persistent()
        .get(&storage_key)
        .expect("Course not found");

    // Only creator can add goal (or later: check admin)
    if course.creator != creator {
        handle_error(&env, Error::OnlyCreatorCanAddGoals)
    }

    // Generate a unique goal ID
    let goal_id = utils::generate_unique_id(&env);

    // Create new goal — lean on-chain record
    let goal: CourseGoal = CourseGoal {
        course_id: course_id.clone(),
        goal_id: goal_id.clone(),
        content_hash: content_hash.clone(),
        created_by: creator.clone(),
        created_at: env.ledger().timestamp(),
    };

    // Save the new goal directly
    env.storage().persistent().set(
        &DataKey::CourseGoal(course_id.clone(), goal_id.clone()),
        &goal,
    );

    // Emit event — only essential blockchain data
    env.events().publish(
        (GOAL_ADDED_EVENT, course_id.clone(), goal_id.clone()),
        content_hash.clone(),
    );

    goal
}

#[cfg(test)]
mod test {
    use crate::schema::Course;
    use crate::{CourseRegistry, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn create_test_course<'a>(
        client: &CourseRegistryClient<'a>,
        creator: &Address,
    ) -> Course {
        client.create_course(
            creator,
            &String::from_str(&client.env, "off_chain_ref_001"),
            &String::from_str(&client.env, "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"),
            &1000_u128,
            &Some(String::from_str(&client.env, "category")),
            &Some(String::from_str(&client.env, "language")),
            &None,
            &None,
        )
    }

    #[test]
    fn test_add_goal_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let course: Course = create_test_course(&client, &creator);

        let content_hash = String::from_str(&env, "deadbeefdeadbeefdeadbeefdeadbeef");
        let goal = client.add_goal(&creator, &course.id, &content_hash);

        assert_eq!(goal.course_id, course.id);
        assert_eq!(goal.content_hash, content_hash);
        assert_eq!(goal.created_by, creator);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #1)")]
    fn test_add_goal_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let impostor: Address = Address::generate(&env);

        let course: Course = create_test_course(&client, &creator);

        let content_hash = String::from_str(&env, "deadbeefdeadbeefdeadbeefdeadbeef");
        client.add_goal(&impostor, &course.id, &content_hash);
    }

    #[test]
    #[should_panic(expected = "Course not found")]
    fn test_add_goal_course_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let fake_course_id = String::from_str(&env, "nonexistent_course");

        let content_hash = String::from_str(&env, "deadbeefdeadbeefdeadbeefdeadbeef");
        client.add_goal(&creator, &fake_course_id, &content_hash);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #2)")]
    fn test_add_goal_empty_content_hash() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let course: Course = create_test_course(&client, &creator);

        client.add_goal(&creator, &course.id, &String::from_str(&env, ""));
    }

    #[test]
    fn test_add_multiple_goals() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let course: Course = create_test_course(&client, &creator);

        let hash1 = String::from_str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1");
        let goal1 = client.add_goal(&creator, &course.id, &hash1);

        let hash2 = String::from_str(&env, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb2");
        let goal2 = client.add_goal(&creator, &course.id, &hash2);

        assert_eq!(goal1.course_id, course.id);
        assert_eq!(goal1.content_hash, hash1);
        assert_eq!(goal1.created_by, creator);

        assert_eq!(goal2.course_id, course.id);
        assert_eq!(goal2.content_hash, hash2);
        assert_eq!(goal2.created_by, creator);

        assert!(goal2.created_at >= goal1.created_at);
        assert_ne!(goal1.goal_id, goal2.goal_id);
    }
}
