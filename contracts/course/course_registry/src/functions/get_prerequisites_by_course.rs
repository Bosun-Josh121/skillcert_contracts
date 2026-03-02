// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{symbol_short, Env, String, Vec, Symbol};
use crate::schema::{Course, CourseId};

const COURSE_KEY: Symbol = symbol_short!("course");

pub fn get_prerequisites_by_course(env: &Env, course_id: String) -> Vec<CourseId> {
    let key: (Symbol, String) = (COURSE_KEY, course_id);

    match env.storage().persistent().get::<_, Course>(&key) {
        Some(course) => course.prerequisites,
        None => Vec::new(env),
    }
}
