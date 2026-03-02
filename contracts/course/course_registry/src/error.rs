// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{contracterror, panic_with_error, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    OnlyCreatorCanAddGoals = 1,
    EmptyGoalContent = 2,
    CourseIdNotExist = 3,
    OnlyCreatorCanArchive = 4,
    CourseAlreadyArchived = 5,
    Unauthorized = 6,
    /// Category name is required (used by create_course_category)
    NameRequired = 7,
    InvalidPrice = 9,
    DuplicateCourseId = 11,
    OnlyCreatorCanEditPrereqs = 12,
    PrereqCourseNotFound = 13,
    SelfPrerequisite = 14,
    CircularDependency = 15,
    EmptyCourseId = 16,
    CourseNotFound = 17,
    EmptyNewGoalContent = 18,
    EmptyGoalId = 19,
    GoalCourseMismatch = 20,
    ModuleNotFound = 21,
    UnauthorizedCaller = 401,
    UnauthorizedCourseAccess = 402,
    InvalidAdminOperation = 403,
    DuplicateModulePosition = 405,
    EmptyModuleId = 22,
    PrereqNotInList = 23,
    InvalidModulePosition = 24,
    InvalidCategoryName = 27,
    EmptyCategory = 28,
    InvalidLanguageLength = 43,
    InvalidDurationValue = 45,
    InvalidLimitValue = 46,
    InvalidOffsetValue = 47,
    InvalidGoalContent = 48,
    InvalidPrerequisiteId = 49,
    EmptyPrerequisiteList = 50,
    TooManyPrerequisites = 51,
    EmptyPrerequisiteId = 52,
    InvalidCourseId = 53,
    InvalidPrice100 = 54,
    AlreadyInitialized = 55,
    DuplicatePrerequisite = 56,
    // Rate limiting errors
    CourseRateLimitExceeded = 57,
    CourseRateLimitNotConfigured = 58,
    /// Content hash is required for on-chain integrity verification
    ContentHashRequired = 59,
    /// Off-chain reference ID is required
    OffChainRefIdRequired = 60,
}

pub fn handle_error(env: &Env, error: Error) -> ! {
    panic_with_error!(env, error);
}
