// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Address, Env, String, Symbol};
use crate::schema::Course;

const COURSE_KEY: Symbol = symbol_short!("course");

pub fn is_course_creator(env: &Env, course_id: String, user: Address) -> bool {
    let key: (Symbol, String) = (COURSE_KEY, course_id.clone());
    let course: Course = env.storage().persistent().get(&key).unwrap();

    if course.creator == user {
        return true;
    }
    false
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::CourseRegistry;
    use crate::{schema::Course, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_is_cource_creator_success() {
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

        let is_creator = client.is_course_creator(&course.id, &creator);

        assert!(is_creator);
    }

    #[test]
    fn test_is_cource_creator_fail() {
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

        let is_creator = client.is_course_creator(&course.id, &impostor);

        assert!(!is_creator);
    }
}
