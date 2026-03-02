// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

use crate::error::{handle_error, Error};
use crate::schema::{Course, CourseGoal, DataKey};

const COURSE_KEY: Symbol = symbol_short!("course");

const GOAL_REMOVED_EVENT: Symbol = symbol_short!("goalRem");

pub fn remove_goal(env: Env, caller: Address, course_id: String, goal_id: String) {
    caller.require_auth();

    // Validate input
    if course_id.is_empty() {
        handle_error(&env, Error::EmptyCourseId)
    }
    if goal_id.is_empty() {
        handle_error(&env, Error::EmptyGoalId)
    }

    // Load course to verify it exists and check permissions
    let storage_key: (Symbol, String) = (COURSE_KEY, course_id.clone());
    let course: Course = env
        .storage()
        .persistent()
        .get(&storage_key)
        .expect("Course not found");

    // Only course creator or authorized admin can remove goals
    if course.creator != caller {
        handle_error(&env, Error::Unauthorized)
    }

    // Check if the goal exists
    let goal_storage_key: DataKey = DataKey::CourseGoal(course_id.clone(), goal_id.clone());
    let goal: CourseGoal = env
        .storage()
        .persistent()
        .get(&goal_storage_key)
        .expect("Goal not found");

    // Verify the goal belongs to the specified course
    if goal.course_id != course_id {
        handle_error(&env, Error::GoalCourseMismatch)
    }

    // Remove the goal from storage
    env.storage().persistent().remove(&goal_storage_key);

    // Emits an event for successful goal removal.
    env.events().publish(
        (GOAL_REMOVED_EVENT, course_id.clone(), goal_id.clone()),
        goal.content_hash.clone(),
    );
}

#[cfg(test)]
mod test {
    use crate::schema::Course;
    use crate::{CourseRegistry, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_remove_goal_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);

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

        let goal_content_hash = String::from_str(&env, "sha256_goal_basics_of_rust");
        let goal = client.add_goal(&creator, &course.id, &goal_content_hash);

        client.remove_goal(&creator, &course.id, &goal.goal_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_remove_goal_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let impostor: Address = Address::generate(&env);

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

        let goal_content_hash = String::from_str(&env, "sha256_goal_basics_of_rust");
        let goal = client.add_goal(&creator, &course.id, &goal_content_hash);

        client.remove_goal(&impostor, &course.id, &goal.goal_id);
    }

    #[test]
    #[should_panic(expected = "Course not found")]
    fn test_remove_goal_course_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let fake_course_id = String::from_str(&env, "nonexistent_course");
        let fake_goal_id = String::from_str(&env, "fake_goal_id");

        client.remove_goal(&creator, &fake_course_id, &fake_goal_id);
    }

    #[test]
    #[should_panic(expected = "Goal not found")]
    fn test_remove_goal_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);

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

        let fake_goal_id = String::from_str(&env, "nonexistent_goal");
        client.remove_goal(&creator, &course.id, &fake_goal_id);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #19)")]
    fn test_remove_goal_empty_goal_id() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);

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

        let empty_goal_id = String::from_str(&env, "");
        client.remove_goal(&creator, &course.id, &empty_goal_id);
    }

    #[test]
    fn test_remove_multiple_goals() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);

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

        let goal1 = client.add_goal(&creator, &course.id, &String::from_str(&env, "sha256_goal1"));
        let goal2 = client.add_goal(&creator, &course.id, &String::from_str(&env, "sha256_goal2"));
        let goal3 = client.add_goal(&creator, &course.id, &String::from_str(&env, "sha256_goal3"));

        client.remove_goal(&creator, &course.id, &goal2.goal_id);
        client.remove_goal(&creator, &course.id, &goal1.goal_id);
        client.remove_goal(&creator, &course.id, &goal3.goal_id);
    }
}
