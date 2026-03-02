// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{contracttype, Address, String, Vec};

/// Course registry defaults and limits
pub const DEFAULT_COURSE_PRICE: u128 = 1000;
pub const MAX_LOOP_GUARD: u32 = 1000;
pub const FILTER_MIN_PRICE: u128 = 500;
pub const MAX_SCAN_ID: u32 = 50;
pub const MAX_EMPTY_CHECKS: u32 = 10;

/// Rate limiting constants for course operations
pub const DEFAULT_COURSE_RATE_LIMIT_WINDOW: u64 = 3600; // 1 hour in seconds
pub const DEFAULT_MAX_COURSE_CREATIONS_PER_WINDOW: u32 = 3; // Max course creations per hour per address

/// Minimal on-chain course module reference.
///
/// Only stores the module's ID, course association,
/// position, a content hash for integrity verification, and a creation timestamp.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CourseModule {
    pub id: String,
    pub course_id: String,
    pub position: u32,
    /// SHA-256 hash of the off-chain module content (title, body, etc.)
    pub content_hash: String,
    pub created_at: u64,
}

/// Minimal on-chain course goal reference.
///
/// Only stores the goal's ID, course association,
/// a content hash for integrity verification, creator, and creation timestamp.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CourseGoal {
    pub goal_id: String,
    pub course_id: String,
    /// SHA-256 hash of the off-chain goal content
    pub content_hash: String,
    pub created_by: Address,
    pub created_at: u64,
}

/// Rate limiting configuration for course operations.
///
/// Tracks rate limiting settings for spam protection in course creation.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CourseRateLimitConfig {
    /// Time window for rate limiting in seconds
    pub window_seconds: u64,
    /// Maximum course creations allowed per window
    pub max_courses_per_window: u32,
}

/// Rate limiting tracking data for course operations per address.
///
/// Stores the current usage count and window start time for course rate limiting.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CourseRateLimitData {
    /// Current count of course creations in this window
    pub count: u32,
    /// Timestamp when the current window started
    pub window_start: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CourseCategory {
    pub id: u128,
    pub name: String,
    pub description: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum DataKey {
    Module(String),
    Courses,
    CourseGoalList(String),     // Optional: Keep a list of goal IDs per course
    CourseGoal(String, String), // (course_id, goal_id)
    CoursePrerequisites(String),
    CategorySeq,          // Sequence counter for category IDs
    CourseCategory(u128), // Course category by ID
    Admins,               // List of admin addresses
    /// Key for storing course rate limiting configuration
    CourseRateLimitConfig,
    /// Key for storing course rate limiting data per address: address -> CourseRateLimitData
    CourseRateLimit(Address),
}

/// Lean on-chain course record.
///
/// Title, description, and thumbnail_url have been moved off-chain.
/// The contract stores only data essential for verifiable credentials,
/// access control, and cryptographic proofs of course content integrity.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Course {
    pub id: String,
    /// Off-chain reference ID (UUID mapping to full course record in DB)
    pub off_chain_ref_id: String,
    /// SHA-256 hash of the off-chain course content (title, description, thumbnail, etc.)
    pub content_hash: String,
    pub creator: Address,
    pub price: u128,
    pub category: Option<String>,
    pub language: Option<String>,
    pub published: bool,
    pub prerequisites: Vec<CourseId>,
    pub is_archived: bool,
    pub level: Option<CourseLevel>,
    pub duration_hours: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CourseId {
    pub id: String,
    pub count: u128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    pub name: String,
    pub count: u128,
}

// Course level as string to avoid Soroban enum serialization issues
// Valid values: "Beginner", "Intermediate", "Advanced"
pub type CourseLevel = String;

/// Filters for querying courses.
///
/// Text search (search_text) removed since title/description are no longer on-chain.
/// Filtering is now limited to on-chain fields: price, category, level, duration.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CourseFilters {
    pub min_price: Option<u128>,
    pub max_price: Option<u128>,
    pub category: Option<String>,
    pub level: Option<CourseLevel>,
    pub min_duration: Option<u32>,
    pub max_duration: Option<u32>,
}

/// Parameters for editing an existing course.
///
/// Title, description, and thumbnail_url fields removed — update these off-chain.
/// The content_hash and off_chain_ref_id can be updated to reflect off-chain changes.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct EditCourseParams {
    /// New content hash (updated when off-chain content changes)
    pub new_content_hash: Option<String>,
    /// New off-chain reference ID
    pub new_off_chain_ref_id: Option<String>,
    pub new_price: Option<u128>,
    pub new_category: Option<Option<String>>,
    pub new_language: Option<Option<String>>,
    pub new_published: Option<bool>,
    pub new_level: Option<Option<CourseLevel>>,
    pub new_duration_hours: Option<Option<u32>>,
}

/// Backup data structure for course registry system.
///
/// Contains all course data, categories, modules, goals, and prerequisites
/// for backup and recovery operations.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CourseBackupData {
    /// All courses in the system
    pub courses: soroban_sdk::Map<String, Course>,
    /// All course categories
    pub categories: soroban_sdk::Map<u128, CourseCategory>,
    /// All course modules
    pub modules: soroban_sdk::Map<String, CourseModule>,
    /// All course goals mapped by (course_id, goal_id)
    pub goals: soroban_sdk::Map<String, soroban_sdk::Vec<CourseGoal>>,
    /// Course prerequisites mapping
    pub prerequisites: soroban_sdk::Map<String, soroban_sdk::Vec<CourseId>>,
    /// Category sequence counter
    pub category_seq: u128,
    /// List of admin addresses
    pub admins: soroban_sdk::Vec<Address>,
    /// Backup timestamp
    pub backup_timestamp: u64,
    /// Backup version for compatibility
    pub backup_version: String,
}
