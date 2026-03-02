// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert
#![allow(clippy::too_many_arguments)]
#![no_std]

/// Contract version for tracking deployments and upgrades
pub const VERSION: &str = "1.0.0";

pub mod error;
pub mod functions;
pub mod schema;

#[cfg(test)]
mod test;

use crate::schema::{
    Course, CourseCategory, CourseFilters, CourseGoal, CourseLevel, CourseModule, EditCourseParams,
};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

/// Course Registry Contract
///
/// This contract manages the creation, modification, and querying of courses
/// in the SkillCert platform. It stores only lean on-chain data essential for
/// verifiable credentials and cryptographic proofs. All descriptive content
/// (titles, descriptions, thumbnails) is stored off-chain.
#[contract]
pub struct CourseRegistry;

#[contractimpl]
impl CourseRegistry {
    /// Create a new course in the registry.
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
        functions::create_course::create_course(
            env,
            creator,
            off_chain_ref_id,
            content_hash,
            price,
            category,
            language,
            level,
            duration_hours,
        )
    }

    /// Create a new course category.
    ///
    /// This function creates a new category that can be used to classify courses.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `caller` - The address of the user creating the category
    /// * `name` - The name of the category
    /// * `description` - Optional description of the category
    ///
    /// # Returns
    ///
    /// Returns the unique ID of the created category.
    ///
    /// # Panics
    ///
    /// * If category name is empty
    /// * If category with same name already exists
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Create a programming category
    /// let category_id = contract.create_course_category(
    ///     env.clone(),
    ///     admin_address,
    ///     "Programming".try_into().unwrap(),
    ///     Some("Computer programming courses".try_into().unwrap())
    /// );
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Duplicate names**: Cannot create categories with existing names
    /// * **Empty names**: Category name cannot be empty
    /// * **Unique IDs**: Each category gets a unique auto-generated ID
    pub fn create_course_category(
        env: Env,
        caller: Address,
        name: String,
        description: Option<String>,
    ) -> u128 {
        functions::create_course_category::create_course_category(env, caller, name, description)
    }

    /// Retrieve a course by its ID.
    ///
    /// This function fetches a course's complete information using its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `course_id` - The unique identifier of the course to retrieve
    ///
    /// # Returns
    ///
    /// Returns the `Course` object containing all course metadata.
    ///
    /// # Panics
    ///
    /// * If course with given ID doesn't exist
    /// * If course_id is invalid or empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Get course by ID
    /// let course = contract.get_course(env.clone(), "course_123".try_into().unwrap());
    /// println!("Course title: {}", course.title);
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Non-existent course**: Will panic if course ID doesn't exist
    /// * **Archived courses**: Still retrievable but marked as archived
    /// * **Public access**: Anyone can retrieve course information
    pub fn get_course(env: Env, course_id: String) -> Course {
        functions::get_course::get_course(&env, course_id)
    }

    /// Retrieve a course category by its ID.
    ///
    /// This function fetches a category's information using its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `category_id` - The unique identifier of the category to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(CourseCategory)` if found, `None` if the category doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Get category by ID
    /// if let Some(category) = contract.get_course_category(env.clone(), 1) {
    ///     println!("Category: {}", category.name);
    /// } else {
    ///     println!("Category not found");
    /// }
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Non-existent category**: Returns `None` instead of panicking
    /// * **Invalid ID**: Returns `None` for invalid category IDs
    /// * **Public access**: Anyone can retrieve category information
    pub fn get_course_category(env: Env, category_id: u128) -> Option<CourseCategory> {
        functions::get_course_category::get_course_category(&env, category_id)
    }

    /// Get all courses created by a specific instructor.
    ///
    /// This function retrieves all courses that were created by the specified instructor.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `instructor` - The address of the instructor to query courses for
    ///
    /// # Returns
    ///
    /// Returns a vector of `Course` objects created by the instructor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Get all courses by an instructor
    /// let instructor_courses = contract.get_courses_by_instructor(env.clone(), instructor_address);
    /// for course in instructor_courses {
    ///     println!("Course: {}", course.title);
    /// }
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **No courses**: Returns empty vector if instructor has no courses
    /// * **Archived courses**: Includes archived courses in results
    /// * **Public access**: Anyone can query instructor courses
    /// * **Invalid instructor**: Returns empty vector for non-existent instructors
    pub fn get_courses_by_instructor(env: Env, instructor: Address) -> Vec<Course> {
        functions::get_courses_by_instructor::get_courses_by_instructor(&env, instructor)
    }

    /// Remove a module from a course.
    ///
    /// This function removes a specific module from its associated course.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `module_id` - The unique identifier of the module to remove
    ///
    /// # Panics
    ///
    /// Remove a module from a course.
    ///
    /// This function removes a specific module from its associated course.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `module_id` - The unique identifier of the module to remove
    ///
    /// # Panics
    ///
    /// * If the module doesn't exist
    /// * If the module_id is invalid or empty
    /// * If module removal operation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Remove a module from a course
    /// contract.remove_module(env.clone(), "module_123".try_into().unwrap());
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Non-existent module**: Will panic if module ID doesn't exist
    /// * **Invalid ID**: Will panic for invalid or empty module IDs
    /// * **Course updates**: Automatically updates course module count
    ///
    /// Panics if the module removal fails or if the module doesn't exist.
    pub fn remove_module(env: Env, module_id: String) {
        functions::remove_module::remove_module(&env, module_id).unwrap_or_else(|e| panic!("{}", e))
    }

    /// Add a new module to a course.
    ///
    /// This function creates and adds a new module to the specified course
    /// at the given position.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `course_id` - The unique identifier of the course to add the module to
    /// * `position` - The position where the module should be inserted
    /// * `title` - The title of the new module
    ///
    /// # Returns
    ///
    /// Returns the created `CourseModule` object.
    ///
    /// # Panics
    ///
    /// * If course doesn't exist
    /// * If caller is not the course creator
    /// * If module title is empty
    /// * If position is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Add a module at position 1
    /// let module = contract.add_module(
    ///     env.clone(),
    ///     course_creator_address,
    ///     "course_123".try_into().unwrap(),
    ///     1,
    ///     "Introduction to Variables".try_into().unwrap()
    /// );
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Invalid position**: Position must be valid for the course
    /// * **Empty title**: Module title cannot be empty
    /// * **Creator only**: Only course creator can add modules
    /// * **Auto-generated ID**: Module gets unique auto-generated ID
    pub fn add_module(
        env: Env,
        caller: Address,
        course_id: String,
        position: u32,
        content_hash: String,
    ) -> CourseModule {
        functions::add_module::course_registry_add_module(env, caller, course_id, position, content_hash)
    }

    /// Delete a course from the registry.
    ///
    /// This function permanently removes a course from the registry.
    /// Only the course creator can delete their own courses.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `creator` - The address of the course creator
    /// * `course_id` - The unique identifier of the course to delete
    ///
    /// # Panics
    ///
    /// * If course doesn't exist
    /// * If creator is not the actual course creator
    /// * If course_id is invalid or empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Course creator deleting their course
    /// contract.delete_course(env.clone(), course_creator_address, "course_123".try_into().unwrap());
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Permission denied**: Only course creator can delete their courses
    /// * **Non-existent course**: Will panic if course doesn't exist
    /// * **Permanent deletion**: Course and all associated data are permanently removed
    /// * **Enrolled students**: Consider impact on enrolled students before deletion
    /// 
    /// Panics if the deletion fails or if the creator is not authorized.
    pub fn delete_course(env: Env, creator: Address, course_id: String) {
        functions::delete_course::delete_course(&env, creator, course_id)
            .unwrap_or_else(|e| panic!("{}", e))
    }

    /// Simple hello world function for testing.
    ///
    /// This is a basic function that returns a greeting message,
    /// primarily used for testing contract deployment and basic functionality.
    ///
    /// # Arguments
    ///
    /// * `_env` - The Soroban environment (unused)
    ///
    /// # Returns
    ///
    /// Returns a greeting string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Test contract deployment
    /// let greeting = contract.hello_world(env.clone());
    /// assert_eq!(greeting, "Hello from Web3 👋");
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Always succeeds**: This function never fails
    /// * **No dependencies**: Requires no external data or state
    /// * **Testing only**: Primarily intended for contract testing
    pub fn hello_world(_env: Env) -> String {
        String::from_str(&_env, "Hello from Web3 👋")
    }

    /// Edit an existing course goal.
    ///
    /// This function allows the course creator to modify the content of an existing goal.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `creator` - The address of the course creator
    /// * `course_id` - The unique identifier of the course
    /// * `goal_id` - The unique identifier of the goal to edit
    /// * `new_content` - The new content for the goal
    ///
    /// # Returns
    ///
    /// Returns the updated `CourseGoal` object.
    ///
    /// # Panics
    ///
    /// * If course doesn't exist
    /// * If goal doesn't exist
    /// * If creator is not the course creator
    /// * If new_content is empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Edit a course goal
    /// let updated_goal = contract.edit_goal(
    ///     env.clone(),
    ///     course_creator_address,
    ///     "course_123".try_into().unwrap(),
    ///     "goal_456".try_into().unwrap(),
    ///     "Updated learning objective".try_into().unwrap()
    /// );
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Empty content**: New content cannot be empty
    /// * **Creator only**: Only course creator can edit goals
    /// * **Non-existent goal**: Will panic if goal ID doesn't exist
    /// * **Content validation**: New content must meet validation requirements
    pub fn edit_goal(
        env: Env,
        creator: Address,
        course_id: String,
        goal_id: String,
        new_content_hash: String,
    ) -> CourseGoal {
        functions::edit_goal::edit_goal(env, creator, course_id, goal_id, new_content_hash)
    }

    /// Add a new goal to a course.
    pub fn add_goal(env: Env, creator: Address, course_id: String, content_hash: String) -> CourseGoal {
        functions::add_goal::add_goal(env, creator, course_id, content_hash)
    }

    /// Remove a goal from a course.
    ///
    /// This function removes a specific learning goal from the course.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `caller` - The address of the user requesting the removal
    /// * `course_id` - The unique identifier of the course
    /// * `goal_id` - The unique identifier of the goal to remove
    ///
    /// # Panics
    ///
    /// * If course doesn't exist
    /// * If goal doesn't exist
    /// * If caller is not the course creator
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Remove a goal from a course
    /// contract.remove_goal(
    ///     env.clone(),
    ///     course_creator_address,
    ///     "course_123".try_into().unwrap(),
    ///     "goal_456".try_into().unwrap()
    /// );
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Creator only**: Only course creator can remove goals
    /// * **Non-existent goal**: Will panic if goal ID doesn't exist
    /// * **Permanent removal**: Goal is permanently deleted from course
    /// * **Goal count**: Automatically updates course goal count
    pub fn remove_goal(env: Env, caller: Address, course_id: String, goal_id: String) {
        functions::remove_goal::remove_goal(env, caller, course_id, goal_id)
    }

    /// Edit an existing course.
    pub fn edit_course(
        env: Env,
        creator: Address,
        course_id: String,
        params: EditCourseParams,
    ) -> Course {
        functions::edit_course::edit_course(env, creator, course_id, params)
    }

    /// Archive a course.
    ///
    /// Returns the archived `Course` with `is_archived` set to `true`.
    pub fn archive_course(env: Env, creator: Address, course_id: String) -> Course {
        functions::archive_course::archive_course(&env, creator, course_id)
    }

    /// Check if a user is the course creator.
    pub fn is_course_creator(env: &Env, course_id: String, user: Address) -> bool {
        functions::is_course_creator::is_course_creator(env, course_id, user)
    }

    /// List all available course categories.
    ///
    /// This function retrieves all course categories that have been created
    /// in the system.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    ///
    /// Returns a vector of all available `Category` objects.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Get all categories
    /// let categories = contract.list_categories(env.clone());
    /// for category in categories {
    ///     println!("Category: {}", category.name);
    /// }
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **Empty system**: Returns empty vector if no categories exist
    /// * **Public access**: Anyone can list categories
    /// * **Order**: Categories are returned in creation order
    pub fn list_categories(env: Env) -> Vec<crate::schema::Category> {
        functions::list_categories::list_categories(&env)
    }

    /// List courses with filtering and pagination.
    ///
    /// This function retrieves courses based on the provided filters
    /// with optional pagination support.
    ///
    /// # Arguments
    ///
    /// * `env` - The Soroban environment
    /// * `filters` - Filtering criteria for courses
    /// * `limit` - Optional maximum number of courses to return
    /// * `offset` - Optional number of courses to skip for pagination
    ///
    /// # Returns
    ///
    /// Returns a vector of `Course` objects matching the filter criteria.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // List first 10 courses
    /// let courses = contract.list_courses_with_filters(
    ///     env.clone(),
    ///     CourseFilters::default(),
    ///     Some(10),
    ///     Some(0)
    /// );
    /// 
    /// // Filter by category
    /// let mut filters = CourseFilters::default();
    /// filters.category = Some("Programming".try_into().unwrap());
    /// let programming_courses = contract.list_courses_with_filters(
    ///     env.clone(),
    ///     filters,
    ///     Some(20),
    ///     None
    /// );
    /// ```
    ///
    /// # Edge Cases
    ///
    /// * **No matches**: Returns empty vector if no courses match filters
    /// * **Large limits**: Limit should be reasonable to avoid gas issues
    /// * **Public access**: Anyone can list courses
    /// * **Archived courses**: May or may not be included based on filter settings
    pub fn list_courses_with_filters(
        env: Env,
        filters: CourseFilters,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Vec<Course> {
        functions::list_courses_with_filters::list_courses_with_filters(
            &env, filters, limit, offset,
        )
    }

    /// List modules for a course.
    pub fn list_modules(env: Env, course_id: String) -> Vec<CourseModule> {
        functions::list_modules::list_modules(&env, course_id)
    }

    /// Add prerequisites to a course.
    pub fn add_prerequisite(
        env: Env,
        caller: Address,
        course_id: String,
        prerequisites: Vec<String>,
    ) {
        functions::create_prerequisite::add_prerequisite(env, caller, course_id, prerequisites)
    }

    /// Edit (replace) all prerequisites for a course.
    pub fn edit_prerequisite(
        env: Env,
        caller: Address,
        course_id: String,
        new_prerequisites: Vec<String>,
    ) {
        functions::edit_prerequisite::edit_prerequisite(env, caller, course_id, new_prerequisites)
    }

    /// Remove a prerequisite from a course.
    pub fn remove_prerequisite(
        env: Env,
        caller: Address,
        course_id: String,
        prereq_course_id: String,
    ) {
        functions::remove_prerequisite::remove_prerequisite(env, caller, course_id, prereq_course_id)
    }

    /// Get prerequisites for a course.
    pub fn get_prerequisites_by_course(env: Env, course_id: String) -> Vec<crate::schema::CourseId> {
        functions::get_prerequisites_by_course::get_prerequisites_by_course(&env, course_id)
    }

    /// Export all course data for backup purposes (admin only)
    ///
    /// This function exports all course data including courses, categories,
    /// modules, goals, and prerequisites for backup and recovery purposes.
    ///
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `caller` - Address performing the export (must be admin)
    ///
    /// # Returns
    /// * `CourseBackupData` - Complete backup data structure
    ///
    /// # Panics
    /// * If caller is not an admin
    pub fn export_course_data(env: Env, caller: Address) -> crate::schema::CourseBackupData {
        functions::backup_recovery::export_course_data(env, caller)
    }

    /// Import course data from backup (admin only)
    ///
    /// This function imports course data from a backup structure.
    /// Only admins can perform this operation. This will overwrite existing data.
    ///
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `caller` - Address performing the import (must be admin)
    /// * `backup_data` - Backup data structure to import
    ///
    /// # Returns
    /// * `u32` - Number of courses imported
    ///
    /// # Panics
    /// * If caller is not an admin
    /// * If backup data is invalid
    /// * If import operation fails
    pub fn import_course_data(env: Env, caller: Address, backup_data: crate::schema::CourseBackupData) -> u32 {
        functions::backup_recovery::import_course_data(env, caller, backup_data)
    }

    /// Get the current contract version
    ///
    /// Returns the semantic version of the current contract deployment.
    /// This is useful for tracking contract upgrades and compatibility.
    ///
    /// # Arguments
    /// * `_env` - The Soroban environment (unused)
    ///
    /// # Returns
    /// * `String` - The current contract version
    pub fn get_contract_version(_env: Env) -> String {
        String::from_str(&_env, VERSION)
    }

    /// Get contract version history
    ///
    /// Returns a list of all versions that have been deployed for this contract.
    /// This helps track the evolution of the contract over time.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `Vec<String>` - Vector of version strings in chronological order
    pub fn get_version_history(env: Env) -> Vec<String> {
        functions::contract_versioning::get_version_history(&env)
    }

    /// Check compatibility between contract versions
    ///
    /// Determines if data from one version can be safely used with another version.
    /// This is crucial for migration processes and backward compatibility.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `from_version` - The source version to check compatibility from
    /// * `to_version` - The target version to check compatibility to
    ///
    /// # Returns
    /// * `bool` - True if the versions are compatible, false otherwise
    pub fn is_version_compatible(env: Env, from_version: String, to_version: String) -> bool {
        functions::contract_versioning::is_version_compatible(&env, from_version, to_version)
    }

    /// Migrate course data between contract versions
    ///
    /// Performs data migration from one contract version to another.
    /// This function handles the transformation of course data structures
    /// when upgrading contract versions.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - The address performing the migration (must be course creator or admin)
    /// * `from_version` - The source version to migrate from
    /// * `to_version` - The target version to migrate to
    ///
    /// # Returns
    /// * `bool` - True if migration was successful, false otherwise
    ///
    /// # Events
    /// Emits a migration event upon successful completion
    pub fn migrate_course_data(env: Env, caller: Address, from_version: String, to_version: String) -> bool {
        functions::contract_versioning::migrate_course_data(&env, caller, from_version, to_version)
    }

    /// Get migration status for the current contract
    ///
    /// Returns information about the current migration status and any
    /// pending migrations that need to be completed.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// * `String` - Migration status information
    pub fn get_migration_status(env: Env) -> String {
        functions::contract_versioning::get_migration_status(&env)
    }

}
