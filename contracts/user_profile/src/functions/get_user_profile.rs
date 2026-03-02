// SPDX-License-Identifier: MIT
// Copyright (c) 2025 SkillCert

use soroban_sdk::{Address, Env, Symbol, symbol_short};

use crate::schema::UserProfile;
use crate::error::{Error, handle_error};

const PROFILE_KEY: Symbol = symbol_short!("profile");

pub fn user_profile_get_user_profile(env: &Env, user_address: Address) -> UserProfile {
    // Get the user profile from storage with proper error handling
    match env
        .storage()
        .instance()
        .get::<(Symbol, Address), UserProfile>(&(PROFILE_KEY, user_address.clone()))
    {
        Some(profile) => profile,
        None => handle_error(env, Error::UserProfileNotFound),
    }
}
