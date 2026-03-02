// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use super::utils::u32_to_string;
use super::course_rate_limit_utils::check_course_creation_rate_limit;
use soroban_sdk::{symbol_short, Address, Env, String, Symbol, Vec};
use crate::error::{handle_error, Error};
use crate::schema::{Course, CourseLevel};

const COURSE_KEY: Symbol = symbol_short!("course");
const COURSE_ID: Symbol = symbol_short!("course");

const CREATE_COURSE_EVENT: Symbol = symbol_short!("crtCourse");
const GENERATE_COURSE_ID_EVENT: Symbol = symbol_short!("genCrsId");

pub fn create_course(
    env: Env,
    creator: Address,
    off_chain_ref_id: String,
    content_hash: String,
    price: u128,
    category: Option<String>,
    language: Option<String>,
    level: Option<CourseLevel>,
    duration_hours: Option<u32>,
) -> Course {
    creator.require_auth();

    // Check rate limiting before proceeding with course creation
    check_course_creation_rate_limit(&env, &creator);

    // Validate required on-chain fields
    if off_chain_ref_id.is_empty() {
        handle_error(&env, Error::OffChainRefIdRequired);
    }

    if content_hash.is_empty() {
        handle_error(&env, Error::ContentHashRequired);
    }

    // ensure the price is greater than 0
    if price == 0 {
        handle_error(&env, Error::InvalidPrice);
    }

    // Validate optional parameters
    if let Some(ref cat) = category {
        if cat.is_empty() || cat.len() > 100 {
            handle_error(&env, Error::EmptyCategory);
        }
    }

    if let Some(ref lang) = language {
        if lang.is_empty() || lang.len() > 50 {
            handle_error(&env, Error::InvalidLanguageLength);
        }
    }

    if let Some(duration) = duration_hours {
        if duration == 0 || duration > 8760 {
            // 8760 hours = 1 year, reasonable maximum
            handle_error(&env, Error::InvalidDurationValue);
        }
    }

    // generate the unique id
    let id: u128 = generate_course_id(&env);
    let converted_id: String = u32_to_string(&env, id as u32);

    let storage_key: (Symbol, String) = (COURSE_KEY, converted_id.clone());

    if env.storage().persistent().has(&storage_key) {
        handle_error(&env, Error::DuplicateCourseId)
    }

    // create a new course — lean on-chain record
    let new_course: Course = Course {
        id: converted_id.clone(),
        off_chain_ref_id: off_chain_ref_id.clone(),
        content_hash: content_hash.clone(),
        creator: creator.clone(),
        price,
        category: category.clone(),
        language: language.clone(),
        published: false,
        prerequisites: Vec::new(&env),
        is_archived: false,
        level: level.clone(),
        duration_hours,
    };

    // save to the storage
    env.storage().persistent().set(&storage_key, &new_course);

    // emit an event — only essential blockchain data
    env.events()
        .publish((CREATE_COURSE_EVENT,), (converted_id, creator, off_chain_ref_id, content_hash, price));

    new_course
}

pub fn generate_course_id(env: &Env) -> u128 {
    let current_id: u128 = env.storage().persistent().get(&COURSE_ID).unwrap_or(0);
    let new_id: u128 = current_id + 1;
    env.storage().persistent().set(&COURSE_ID, &new_id);

    // emit an event
    env.events()
        .publish((GENERATE_COURSE_ID_EVENT,), new_id);

    new_id
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::schema::Course;
    use crate::{CourseRegistry, CourseRegistryClient};
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_generate_course_id() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id: Address = env.register(CourseRegistry, {});

        env.as_contract(&contract_id, || {
            generate_course_id(&env);
            let id: u128 = generate_course_id(&env);
            assert_eq!(id, 2);
        });
    }

    #[test]
    fn test_add_course_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);

        let off_chain_ref_id = String::from_str(&env, "course_ref_001");
        let content_hash = String::from_str(&env, "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4");
        let price = 1000_u128;
        let category = Some(String::from_str(&env, "category"));
        let language = Some(String::from_str(&env, "language"));

        let course: Course = client.create_course(
            &creator,
            &off_chain_ref_id,
            &content_hash,
            &price,
            &category,
            &language,
            &None,
            &None,
        );

        let course = client.get_course(&course.id);
        assert_eq!(course.off_chain_ref_id, off_chain_ref_id);
        assert_eq!(course.content_hash, content_hash);
        assert_eq!(course.id, String::from_str(&env, "1"));
        assert_eq!(course.price, price);
        assert_eq!(course.category, category);
        assert_eq!(course.language, language);
        assert!(!course.published);
    }

    #[test]
    fn test_add_course_success_multiple() {
        let env: Env = Env::default();
        env.mock_all_auths();

        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let price: u128 = crate::schema::DEFAULT_COURSE_PRICE;
        let another_price: u128 = 2000;

        client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "ref_001"),
            &String::from_str(&env, "hash_aaaaaaaaaaaaaaaaaaaaaaaaaaaa01"),
            &price,
            &None,
            &None,
            &None,
            &None,
        );

        let course2 = client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "ref_002"),
            &String::from_str(&env, "hash_aaaaaaaaaaaaaaaaaaaaaaaaaaaa02"),
            &another_price,
            &None,
            &None,
            &None,
            &None,
        );

        let stored_course = client.get_course(&course2.id);

        assert_eq!(stored_course.off_chain_ref_id, String::from_str(&env, "ref_002"));
        assert_eq!(stored_course.id, String::from_str(&env, "2"));
        assert_eq!(stored_course.price, another_price);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #9)")]
    fn test_cannot_create_courses_with_zero_price() {
        let env: Env = Env::default();
        env.mock_all_auths();
        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let price: u128 = 0;

        client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "ref_001"),
            &String::from_str(&env, "hash_aaaaaaaaaaaaaaaaaaaaaaaaaaaa01"),
            &price,
            &None,
            &None,
            &None,
            &None,
        );
    }

    #[test]
    #[should_panic]
    fn test_cannot_create_course_with_empty_off_chain_ref_id() {
        let env: Env = Env::default();
        env.mock_all_auths();
        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, ""), // empty off_chain_ref_id
            &String::from_str(&env, "hash_aaaaaaaaaaaaaaaaaaaaaaaaaaaa01"),
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
    }

    #[test]
    #[should_panic]
    fn test_cannot_create_course_with_empty_content_hash() {
        let env: Env = Env::default();
        env.mock_all_auths();
        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "ref_001"),
            &String::from_str(&env, ""), // empty content_hash
            &crate::schema::DEFAULT_COURSE_PRICE,
            &None,
            &None,
            &None,
            &None,
        );
    }

    #[test]
    fn test_create_course_with_maximum_price() {
        let env: Env = Env::default();
        env.mock_all_auths();
        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let max_price: u128 = u128::MAX;

        let course = client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "premium_ref_001"),
            &String::from_str(&env, "hash_aaaaaaaaaaaaaaaaaaaaaaaaaaaa01"),
            &max_price,
            &None,
            &None,
            &None,
            &None,
        );
        assert_eq!(course.price, max_price);
        assert_eq!(course.id, String::from_str(&env, "1"));
    }

    #[test]
    fn test_create_course_with_all_optional_fields() {
        let env: Env = Env::default();
        env.mock_all_auths();
        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let price: u128 = 3000;
        let category: Option<String> = Some(String::from_str(&env, "Web Development"));
        let language: Option<String> = Some(String::from_str(&env, "Spanish"));
        let level: Option<CourseLevel> = Some(String::from_str(&env, "Intermediate"));
        let duration_hours: Option<u32> = Some(40);

        let course = client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "complete_ref_001"),
            &String::from_str(&env, "hash_complete_aaaaabbbbccccddddeee"),
            &price,
            &category,
            &language,
            &level,
            &duration_hours,
        );
        assert_eq!(course.off_chain_ref_id, String::from_str(&env, "complete_ref_001"));
        assert_eq!(course.price, price);
        assert_eq!(course.category, category);
        assert_eq!(course.language, language);
        assert_eq!(course.level, level);
        assert_eq!(course.duration_hours, duration_hours);
        assert!(!course.published);
    }

    #[test]
    fn test_create_course_with_partial_optional_fields() {
        let env: Env = Env::default();
        env.mock_all_auths();
        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let price: u128 = 1800;
        let category: Option<String> = Some(String::from_str(&env, "Data Science"));

        let course = client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "partial_ref_001"),
            &String::from_str(&env, "hash_partial_aaaaabbbbccccddddeeee"),
            &price,
            &category,
            &None,
            &None,
            &None,
        );
        assert_eq!(course.price, price);
        assert_eq!(course.category, category);
        assert_eq!(course.language, None);
    }

    #[test]
    fn test_create_multiple_courses_sequential_ids() {
        let env: Env = Env::default();
        env.mock_all_auths();
        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let price: u128 = crate::schema::DEFAULT_COURSE_PRICE;

        let course1 = client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "ref_001"),
            &String::from_str(&env, "hash_aaaaaaaaaaaaaaaaaaaaaaaaaaaa01"),
            &price,
            &None,
            &None,
            &None,
            &None,
        );

        let course2 = client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "ref_002"),
            &String::from_str(&env, "hash_aaaaaaaaaaaaaaaaaaaaaaaaaaaa02"),
            &price,
            &None,
            &None,
            &None,
            &None,
        );

        let course3 = client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "ref_003"),
            &String::from_str(&env, "hash_aaaaaaaaaaaaaaaaaaaaaaaaaaaa03"),
            &price,
            &None,
            &None,
            &None,
            &None,
        );

        assert_eq!(course1.id, String::from_str(&env, "1"));
        assert_eq!(course2.id, String::from_str(&env, "2"));
        assert_eq!(course3.id, String::from_str(&env, "3"));
    }

    #[test]
    fn test_create_course_with_unicode_ref_id() {
        let env: Env = Env::default();
        env.mock_all_auths();
        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let price: u128 = 2000;
        let language: Option<String> = Some(String::from_str(&env, "Español"));

        let course = client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "curso_programacion_espanol_001"),
            &String::from_str(&env, "hash_espanol_aaabbbbccccddddeeeeff"),
            &price,
            &None,
            &language,
            &None,
            &None,
        );
        assert_eq!(course.language, language);
        assert_eq!(course.price, price);
    }

    #[test]
    fn test_create_course_content_hash_is_stored() {
        let env: Env = Env::default();
        env.mock_all_auths();
        let contract_id: Address = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let expected_hash = String::from_str(&env, "deadbeef1234567890abcdef12345678");

        let course = client.create_course(
            &Address::generate(&env),
            &String::from_str(&env, "integrity_ref_001"),
            &expected_hash,
            &1000_u128,
            &None,
            &None,
            &None,
            &None,
        );

        let stored = client.get_course(&course.id);
        assert_eq!(stored.content_hash, expected_hash);
    }
}
