use crate::error::{handle_error, Error};
use crate::functions::utils::u32_to_string;

use crate::schema::{Course, CourseFilters, MAX_EMPTY_CHECKS};
use soroban_sdk::{symbol_short, Env, Symbol, Vec, String};

const COURSE_KEY: Symbol = symbol_short!("course");

pub fn list_courses_with_filters(
    env: &Env,
    filters: CourseFilters,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Vec<Course> {
    // Validate pagination parameters to prevent abuse
    if let Some(l) = limit {
        if l > 100 {
            handle_error(env, Error::InvalidLimitValue)
        }
    }
    if let Some(o) = offset {
        if o > 10000 {
            handle_error(env, Error::InvalidOffsetValue)
        }
    }

    let mut results: Vec<Course> = Vec::new(env);
    let mut id: u128 = 1;
    let mut count: u32 = 0;
    let mut matched: u32 = 0;
    let mut empty_checks: u32 = 0;

    let offset_value: u32 = offset.unwrap_or(0);
    let limit_value: u32 = limit.unwrap_or(10);

    // Safety check for limit
    let max_limit: u32 = if limit_value > 20 { 20 } else { limit_value };

    loop {
        if id > crate::schema::MAX_SCAN_ID as u128
            || empty_checks > MAX_EMPTY_CHECKS as u32
        {
            break;
        }

        let course_id: String = u32_to_string(env, id as u32);
        let key: (Symbol, String) = (COURSE_KEY, course_id.clone());

        if !env.storage().persistent().has(&key) {
            empty_checks += 1;
            id += 1;
            continue;
        }

        empty_checks = 0;

        let course: Course = env.storage().persistent().get(&key).unwrap();

        // Skip archived or unpublished courses
        if course.is_archived || !course.published {
            id += 1;
            continue;
        }

        // Apply on-chain filters only (text search removed — title/description are off-chain)
        let passes_filters: bool = filters.min_price.map_or(true, |min| course.price >= min)
            && filters.max_price.map_or(true, |max| course.price <= max)
            && filters
                .category
                .as_ref()
                .is_none_or(|cat| course.category.as_ref() == Some(cat))
            && filters
                .level
                .as_ref()
                .map_or(true, |lvl| course.level.as_ref() == Some(lvl))
            && filters.min_duration.map_or(true, |min| {
                course.duration_hours.map_or(false, |d| d >= min)
            })
            && filters.max_duration.map_or(true, |max| {
                course.duration_hours.map_or(false, |d| d <= max)
            });

        if passes_filters {
            if matched >= offset_value {
                if count < max_limit {
                    results.push_back(course);
                    count += 1;
                } else {
                    break;
                }
            }
            matched += 1;
        }

        id += 1;
    }

    results
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{CourseRegistry, CourseRegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_empty_list_no_courses() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);

        let filters = CourseFilters {
            min_price: None,
            max_price: None,
            category: None,
            level: None,
            min_duration: None,
            max_duration: None,
        };

        let results = client.list_courses_with_filters(&filters, &None, &None);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_single_course_no_filters() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator = Address::generate(&env);

        let course = client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "abc123hash"),
            &100,
            &None,
            &None,
            &None,
            &None,
        );

        // Publish the course so it appears in filtered results
        use crate::schema::EditCourseParams;
        let params = EditCourseParams {
            new_content_hash: None,
            new_off_chain_ref_id: None,
            new_price: None,
            new_category: None,
            new_language: None,
            new_published: Some(true),
            new_level: None,
            new_duration_hours: None,
        };
        client.edit_course(&creator, &course.id, &params);

        let filters = CourseFilters {
            min_price: None,
            max_price: None,
            category: None,
            level: None,
            min_duration: None,
            max_duration: None,
        };

        let results = client.list_courses_with_filters(&filters, &None, &None);
        assert_eq!(results.len(), 1);
        assert_eq!(results.get(0).unwrap().price, 100);
    }

    #[test]
    fn test_price_filter_excludes_course() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator = Address::generate(&env);

        client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "abc123hash"),
            &100,
            &None,
            &None,
            &None,
            &None,
        );

        let filters = CourseFilters {
            min_price: Some(crate::schema::FILTER_MIN_PRICE),
            max_price: Some(crate::schema::DEFAULT_COURSE_PRICE),
            category: None,
            level: None,
            min_duration: None,
            max_duration: None,
        };

        let results = client.list_courses_with_filters(&filters, &None, &None);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_pagination_limit() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CourseRegistry, ());
        let client = CourseRegistryClient::new(&env, &contract_id);
        let creator = Address::generate(&env);

        client.create_course(
            &creator,
            &String::from_str(&env, "ref-001"),
            &String::from_str(&env, "abc123hash"),
            &100,
            &None,
            &None,
            &None,
            &None,
        );

        let filters = CourseFilters {
            min_price: None,
            max_price: None,
            category: None,
            level: None,
            min_duration: None,
            max_duration: None,
        };

        let results = client.list_courses_with_filters(&filters, &Some(0), &None);
        assert_eq!(results.len(), 0);
    }
}
