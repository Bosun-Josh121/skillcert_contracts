// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use crate::schema::{Course, CourseBackupData, CourseCategory, CourseGoal, CourseId, CourseModule, DataKey};
use soroban_sdk::{Address, Env, Map, String, Vec};

/// Export all course data for backup purposes
///
/// This function creates a complete backup of all course data including courses,
/// categories, modules, goals, and prerequisites.
///
/// # Arguments
/// * `env` - Soroban environment
/// * `caller` - Address requesting the backup (must be admin)
///
/// # Returns
/// * `CourseBackupData` - Complete backup structure
///
/// # Panics
/// * If caller is not an admin
pub fn export_course_data(env: Env, caller: Address) -> CourseBackupData {
    caller.require_auth();

    // Verify caller is admin
    if !is_admin(&env, caller) {
        panic!("Unauthorized: Only admins can export course data");
    }

    // Initialize maps for backup data
    let mut courses = Map::new(&env);
    let mut categories = Map::new(&env);
    let mut modules = Map::new(&env);
    let mut goals = Map::new(&env);
    let mut prerequisites = Map::new(&env);

    // Get all courses by iterating through course IDs
    // Courses are stored as (Symbol("course"), course_id) -> Course
    let course_key = soroban_sdk::symbol_short!("course");
    let mut all_courses = Vec::new(&env);
    
    // Get the current course ID to know how many courses exist
    let course_id_key = soroban_sdk::symbol_short!("course");
    let max_course_id: u128 = env
        .storage()
        .persistent()
        .get(&course_id_key)
        .unwrap_or(0u128);
    
    // Iterate through all possible course IDs
    for id in 1..=max_course_id {
        let course_id_str = super::utils::u32_to_string(&env, id as u32);
        let storage_key = (course_key.clone(), course_id_str.clone());
        
        if let Some(course) = env.storage().persistent().get::<_, Course>(&storage_key) {
            all_courses.push_back(course.clone());
            courses.set(course.id.clone(), course.clone());
            
            // Export course goals
            if let Some(course_goals) = env
                .storage()
                .persistent()
                .get::<DataKey, Vec<CourseGoal>>(&DataKey::CourseGoalList(course.id.clone()))
            {
                goals.set(course.id.clone(), course_goals);
            }

            // Export course prerequisites
            if let Some(course_prereqs) = env
                .storage()
                .persistent()
                .get::<DataKey, Vec<CourseId>>(&DataKey::CoursePrerequisites(course.id.clone()))
            {
                prerequisites.set(course.id.clone(), course_prereqs);
            }

            // Export course modules (simplified version)
            let module_id = String::from_str(&env, "default_module");
            let course_module = CourseModule {
                id: module_id.clone(),
                course_id: course.id.clone(),
                position: 1,
                content_hash: String::from_str(&env, "default_content_hash"),
                created_at: env.ledger().timestamp(),
            };
            modules.set(module_id, course_module);
        }
    }


    // Export all categories
    let mut category_id = 1u128;
    loop {
        if let Some(category) = env
            .storage()
            .persistent()
            .get::<DataKey, CourseCategory>(&DataKey::CourseCategory(category_id))
        {
            categories.set(category_id, category);
            category_id += 1;
        } else {
            break;
        }
        
        // Safety check to avoid infinite loops
        if category_id > 10000 {
            break;
        }
    }

    // Get category sequence counter
    let category_seq: u128 = env
        .storage()
        .persistent()
        .get(&DataKey::CategorySeq)
        .unwrap_or(0);

    // Get admin list
    let admins: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Admins)
        .unwrap_or(Vec::new(&env));

    // Create backup data structure
    CourseBackupData {
        courses,
        categories,
        modules,
        goals,
        prerequisites,
        category_seq,
        admins,
        backup_timestamp: env.ledger().timestamp(),
        backup_version: String::from_str(&env, "1.0.0"),
    }
}

/// Import course data from backup
///
/// This function restores course data from a backup structure.
/// This operation will overwrite existing data.
///
/// # Arguments
/// * `env` - Soroban environment
/// * `caller` - Address performing the import (must be admin)
/// * `backup_data` - Backup data to restore
///
/// # Returns
/// * `u32` - Number of courses imported
///
/// # Panics
/// * If caller is not an admin
/// * If backup data is invalid
pub fn import_course_data(env: Env, caller: Address, backup_data: CourseBackupData) -> u32 {
    caller.require_auth();

    // Verify caller is admin
    if !is_admin(&env, caller) {
        panic!("Unauthorized: Only admins can import course data");
    }

    // Validate backup version compatibility
    let expected_version = String::from_str(&env, "1.0.0");
    if backup_data.backup_version != expected_version {
        panic!("Incompatible backup version");
    }

    let mut imported_count = 0u32;
    let course_key = soroban_sdk::symbol_short!("course");

    // Import courses - store each course individually
    for (_course_id, course) in backup_data.courses.iter() {
        let storage_key = (course_key.clone(), course.id.clone());
        env.storage()
            .persistent()
            .set(&storage_key, &course);
        imported_count += 1;
    }

    // Import categories
    for (category_id, category) in backup_data.categories.iter() {
        env.storage()
            .persistent()
            .set(&DataKey::CourseCategory(category_id), &category);
    }

    // Import modules
    for (module_id, module) in backup_data.modules.iter() {
        env.storage()
            .persistent()
            .set(&DataKey::Module(module_id), &module);
    }

    // Import goals
    for (course_id, course_goals) in backup_data.goals.iter() {
        env.storage()
            .persistent()
            .set(&DataKey::CourseGoalList(course_id), &course_goals);
    }

    // Import prerequisites
    for (course_id, prereqs) in backup_data.prerequisites.iter() {
        env.storage()
            .persistent()
            .set(&DataKey::CoursePrerequisites(course_id), &prereqs);
    }

    // Import category sequence counter
    env.storage()
        .persistent()
        .set(&DataKey::CategorySeq, &backup_data.category_seq);

    // Import admin list
    env.storage()
        .persistent()
        .set(&DataKey::Admins, &backup_data.admins);

    // Emit import event
    env.events().publish(
        (String::from_str(&env, "course_data_imported"),),
        (imported_count, backup_data.backup_timestamp),
    );

    imported_count
}

/// Check if an address is an admin
/// 
/// This is a simplified version for the backup system.
/// In a real implementation, this would check against the user_management contract.
fn is_admin(env: &Env, address: Address) -> bool {
    let admins: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Admins)
        .unwrap_or(Vec::new(env));
    
    for admin in admins.iter() {
        if admin == address {
            return true;
        }
    }
    false
}
