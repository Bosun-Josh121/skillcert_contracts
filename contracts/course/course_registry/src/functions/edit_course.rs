// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

use crate::error::{handle_error, Error};
use crate::schema::{Course, EditCourseParams};

const COURSE_KEY: Symbol = symbol_short!("course");

const EDIT_COURSE_EVENT: Symbol = symbol_short!("editCours");

pub fn edit_course(
    env: Env,
    creator: Address,
    course_id: String,
    params: EditCourseParams,
) -> Course {
    creator.require_auth();

    // --- Load existing course ---
    let storage_key: (Symbol, String) = (COURSE_KEY, course_id.clone());
    let mut course: Course = env
        .storage()
        .persistent()
        .get(&storage_key)
        .expect("Course error: Course not found");

    // --- Permission: only creator can edit ---
    if creator != course.creator {
        handle_error(&env, Error::Unauthorized)
    }

    // --- Content hash update ---
    if let Some(ref hash) = params.new_content_hash {
        if hash.is_empty() {
            handle_error(&env, Error::ContentHashRequired);
        }
        course.content_hash = hash.clone();
    }

    // --- Off-chain ref ID update ---
    if let Some(ref ref_id) = params.new_off_chain_ref_id {
        if ref_id.is_empty() {
            handle_error(&env, Error::OffChainRefIdRequired);
        }
        course.off_chain_ref_id = ref_id.clone();
    }

    // --- Price (>0) ---
    if let Some(p) = params.new_price {
        if p == 0 {
            handle_error(&env, Error::InvalidPrice);
        }
        course.price = p;
    }

    // --- Optional fields: category / language ---
    if let Some(cat) = params.new_category {
        course.category = cat; // Some(value) sets; None clears
    }
    if let Some(lang) = params.new_language {
        course.language = lang;
    }

    // --- Published flag ---
    if let Some(p) = params.new_published {
        course.published = p;
    }

    // --- Level field ---
    if let Some(level) = params.new_level {
        course.level = level; // Some(value) sets; None clears
    }

    // --- Duration hours field ---
    if let Some(duration) = params.new_duration_hours {
        course.duration_hours = duration; // Some(value) sets; None clears
    }

    // --- Persist updated course ---
    env.storage().persistent().set(&storage_key, &course);

    // --- Emit event ---
    env.events()
        .publish((EDIT_COURSE_EVENT,), (creator, course_id));

    course
}

#[cfg(test)]
mod test {
    use crate::schema::{Course, EditCourseParams};
    use crate::{CourseRegistry, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn create_test_course<'a>(
        client: &CourseRegistryClient<'a>,
        creator: &Address,
        off_chain_ref_id: &str,
        content_hash: &str,
    ) -> Course {
        client.create_course(
            creator,
            &String::from_str(&client.env, off_chain_ref_id),
            &String::from_str(&client.env, content_hash),
            &1000_u128,
            &Some(String::from_str(&client.env, "original_category")),
            &Some(String::from_str(&client.env, "original_language")),
            &None,
            &None,
        )
    }

    #[test]
    fn test_edit_course_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator: Address = Address::generate(&env);

        let course: Course = create_test_course(
            &client,
            &creator,
            "original_ref_001",
            "hash_original_aabbccddeeff112233",
        );

        let params = EditCourseParams {
            new_content_hash: Some(String::from_str(&env, "hash_updated_aabbccddeeff998877")),
            new_off_chain_ref_id: Some(String::from_str(&env, "updated_ref_002")),
            new_price: Some(2000_u128),
            new_category: Some(Some(String::from_str(&env, "new_category"))),
            new_language: Some(Some(String::from_str(&env, "new_language"))),
            new_published: Some(true),
            new_level: None,
            new_duration_hours: None,
        };
        let edited_course = client.edit_course(&creator, &course.id, &params);

        assert_eq!(
            edited_course.content_hash,
            String::from_str(&env, "hash_updated_aabbccddeeff998877")
        );
        assert_eq!(
            edited_course.off_chain_ref_id,
            String::from_str(&env, "updated_ref_002")
        );
        assert_eq!(edited_course.price, 2000_u128);
        assert_eq!(
            edited_course.category,
            Some(String::from_str(&env, "new_category"))
        );
        assert_eq!(
            edited_course.language,
            Some(String::from_str(&env, "new_language"))
        );
        assert_eq!(edited_course.published, true);
        assert_eq!(edited_course.creator, creator);

        let retrieved_course = client.get_course(&course.id);
        assert_eq!(
            retrieved_course.content_hash,
            String::from_str(&env, "hash_updated_aabbccddeeff998877")
        );
        assert_eq!(
            retrieved_course.off_chain_ref_id,
            String::from_str(&env, "updated_ref_002")
        );
        assert_eq!(retrieved_course.price, 2000_u128);
        assert_eq!(retrieved_course.published, true);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #6)")]
    fn test_edit_course_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let impostor: Address = Address::generate(&env);

        let course: Course = create_test_course(
            &client,
            &creator,
            "original_ref_001",
            "hash_original_aabbccddeeff112233",
        );

        let params = EditCourseParams {
            new_content_hash: Some(String::from_str(&env, "hash_hacked_aabbccddeeff998877")),
            new_off_chain_ref_id: None,
            new_price: None,
            new_category: None,
            new_language: None,
            new_published: None,
            new_level: None,
            new_duration_hours: None,
        };
        client.edit_course(&impostor, &course.id, &params);
    }

    #[test]
    #[should_panic(expected = "Course error: Course not found")]
    fn test_edit_course_not_found() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);

        let creator: Address = Address::generate(&env);
        let fake_course_id = String::from_str(&env, "nonexistent_course");

        let params = EditCourseParams {
            new_content_hash: Some(String::from_str(&env, "hash_new_aabbccddeeff998877")),
            new_off_chain_ref_id: None,
            new_price: None,
            new_category: None,
            new_language: None,
            new_published: None,
            new_level: None,
            new_duration_hours: None,
        };
        client.edit_course(&creator, &fake_course_id, &params);
    }

    #[test]
    #[should_panic]
    fn test_edit_course_empty_content_hash() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator: Address = Address::generate(&env);

        let course: Course = create_test_course(
            &client,
            &creator,
            "original_ref_001",
            "hash_original_aabbccddeeff112233",
        );

        let params = EditCourseParams {
            new_content_hash: Some(String::from_str(&env, "")), // empty hash should panic
            new_off_chain_ref_id: None,
            new_price: None,
            new_category: None,
            new_language: None,
            new_published: None,
            new_level: None,
            new_duration_hours: None,
        };
        client.edit_course(&creator, &course.id, &params);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #9)")]
    fn test_edit_course_zero_price() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator: Address = Address::generate(&env);

        let course: Course = create_test_course(
            &client,
            &creator,
            "original_ref_001",
            "hash_original_aabbccddeeff112233",
        );

        let params = EditCourseParams {
            new_content_hash: None,
            new_off_chain_ref_id: None,
            new_price: Some(0_u128),
            new_category: None,
            new_language: None,
            new_published: None,
            new_level: None,
            new_duration_hours: None,
        };
        client.edit_course(&creator, &course.id, &params);
    }

    #[test]
    fn test_edit_course_partial_fields() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator: Address = Address::generate(&env);

        let course: Course = create_test_course(
            &client,
            &creator,
            "original_ref_001",
            "hash_original_aabbccddeeff112233",
        );

        let params = EditCourseParams {
            new_content_hash: Some(String::from_str(&env, "hash_updated_aabbccddeeff998877")),
            new_off_chain_ref_id: None, // not updating ref_id
            new_price: Some(2000_u128),
            new_category: None,
            new_language: None,
            new_published: None,
            new_level: None,
            new_duration_hours: None,
        };
        let edited_course = client.edit_course(&creator, &course.id, &params);

        assert_eq!(
            edited_course.content_hash,
            String::from_str(&env, "hash_updated_aabbccddeeff998877")
        );
        // off_chain_ref_id unchanged
        assert_eq!(
            edited_course.off_chain_ref_id,
            String::from_str(&env, "original_ref_001")
        );
        assert_eq!(edited_course.price, 2000_u128);
        assert_eq!(
            edited_course.category,
            Some(String::from_str(&env, "original_category"))
        );
        assert_eq!(
            edited_course.language,
            Some(String::from_str(&env, "original_language"))
        );
        assert_eq!(edited_course.published, false); // Default value, unchanged
    }

    #[test]
    fn test_edit_course_update_off_chain_ref_id_only() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, {});
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator: Address = Address::generate(&env);

        let course: Course = create_test_course(
            &client,
            &creator,
            "original_ref_001",
            "hash_original_aabbccddeeff112233",
        );

        let params = EditCourseParams {
            new_content_hash: None,
            new_off_chain_ref_id: Some(String::from_str(&env, "new_ref_v2_002")),
            new_price: None,
            new_category: None,
            new_language: None,
            new_published: None,
            new_level: None,
            new_duration_hours: None,
        };
        let edited_course = client.edit_course(&creator, &course.id, &params);

        assert_eq!(
            edited_course.off_chain_ref_id,
            String::from_str(&env, "new_ref_v2_002")
        );
        // content_hash should remain unchanged
        assert_eq!(
            edited_course.content_hash,
            String::from_str(&env, "hash_original_aabbccddeeff112233")
        );
    }
}
