// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{contracterror, panic_with_error, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    UserAlreadyHasAccess = 1,
    UserNoAccessCourse = 2,
    Unauthorized = 3,
    InvalidCourseId = 7,
    InvalidUser = 8,
    EmptyCourseId = 9,
    InvalidTransferData = 10,
    SameUserTransfer = 11,
    Initialized = 12,
    OffChainRefIdRequired = 13,
}

pub fn handle_error(env: &Env, error: Error) -> ! {
    panic_with_error!(env, error);
}
