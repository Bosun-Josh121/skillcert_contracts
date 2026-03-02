// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Env, String, Symbol, Vec};

use crate::error::{handle_error, Error};
use crate::functions::utils::{concat_strings, u32_to_string};
use crate::schema::CourseModule;

const COURSE_KEY: Symbol = symbol_short!("course");
const MODULE_KEY: Symbol = symbol_short!("module");

/// Lists all modules belonging to a given course.
///
/// Scans module storage keys using the same ID pattern as `add_module`
/// (`module_{course_id}_{position}_{ledger_seq}`) and collects all that
/// match the requested course.
pub fn list_modules(env: &Env, course_id: String) -> Vec<CourseModule> {
    if course_id.is_empty() {
        handle_error(env, Error::EmptyCourseId)
    }

    // Verify the course exists
    let course_storage_key: (Symbol, String) = (COURSE_KEY, course_id.clone());
    if !env.storage().persistent().has(&course_storage_key) {
        handle_error(env, Error::CourseIdNotExist)
    }

    let mut modules: Vec<CourseModule> = Vec::new(env);

    // Scan possible module positions (mirrors delete_course_modules pattern)
    let mut position: u32 = 0;
    let mut empty_streak: u32 = 0;

    while position <= crate::schema::MAX_LOOP_GUARD && empty_streak <= crate::schema::MAX_EMPTY_CHECKS {
        // Build the module key prefix for this position
        // Module IDs follow: module_{course_id}_{position}_{ledger_seq}
        // We can't know ledger_seq, so check position-keyed storage instead
        let position_key: (Symbol, String, u32) = (symbol_short!("pos"), course_id.clone(), position);

        if env.storage().persistent().has(&position_key) {
            empty_streak = 0;

            // Try to find the module using the same ID pattern as add_module
            // Since we don't know ledger_seq, iterate a reasonable range
            let mut seq: u32 = 0;
            while seq < 1000 {
                let arr: Vec<String> = soroban_sdk::vec![
                    &env,
                    String::from_str(env, "module_"),
                    course_id.clone(),
                    String::from_str(env, "_"),
                    u32_to_string(env, position),
                    String::from_str(env, "_"),
                    u32_to_string(env, seq),
                ];
                let module_id: String = concat_strings(env, arr);
                let storage_key: (Symbol, String) = (MODULE_KEY, module_id.clone());

                if let Some(module) = env.storage().persistent().get::<_, CourseModule>(&storage_key) {
                    if module.course_id == course_id {
                        modules.push_back(module);
                    }
                    break; // found the module for this position
                }
                seq += 1;
            }
        } else {
            empty_streak += 1;
        }

        position += 1;
    }

    modules
}

#[cfg(test)]
mod test {
    use crate::CourseRegistry;
    use crate::schema::CourseModule;
    use soroban_sdk::{symbol_short, testutils::Ledger, Address, Env, String, Symbol};

    const MODULE_KEY: Symbol = symbol_short!("module");

    #[test]
    fn test_course_registry_list_modules_single() {
        let env: Env = Env::default();
        env.ledger().set_timestamp(100000);

        let contract_id: Address = env.register(CourseRegistry, {});

        let module: CourseModule = CourseModule {
            id: String::from_str(&env, "test_module_123"),
            course_id: String::from_str(&env, "test_course_123"),
            position: 0,
            content_hash: String::from_str(&env, "sha256_intro_to_blockchain"),
            created_at: 0,
        };

        env.as_contract(&contract_id, || {
            env.storage()
                .persistent()
                .set(&(MODULE_KEY, module.course_id.clone()), &module);
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #16)")]
    fn test_list_modules_empty_course_id() {
        let env: Env = Env::default();
        let contract_id: Address = env.register(CourseRegistry, {});

        let course_id: String = String::from_str(&env, "");

        env.as_contract(&contract_id, || {
            super::list_modules(&env, course_id);
        });
    }
}
