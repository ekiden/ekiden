use std;

use ekiden_common::profile_block;

use super::db::Db;
use super::diffs;

#[no_mangle]
pub extern "C" fn db_state_diff(
    old: *const u8,
    old_length: usize,
    new: *const u8,
    new_length: usize,
    diff: *mut u8,
    diff_capacity: usize,
    diff_length: *mut usize,
) {
    profile_block!();

    let old = unsafe { std::slice::from_raw_parts(old, old_length) };
    let new = unsafe { std::slice::from_raw_parts(new, new_length) };

    // TODO: Error handling.
    let result = match diffs::diff(&old, &new) {
        Ok(result) => result,
        _ => panic!("Error while computing difference"),
    };

    // Copy back response.
    if result.len() <= diff_capacity {
        unsafe {
            for i in 0..result.len() as isize {
                std::ptr::write(diff.offset(i), result[i as usize]);
            }
            *diff_length = result.len();
        };
    }
}

#[no_mangle]
pub extern "C" fn db_state_apply(
    old: *const u8,
    old_length: usize,
    diff: *const u8,
    diff_length: usize,
    new: *mut u8,
    new_capacity: usize,
    new_length: *mut usize,
) {
    profile_block!();

    let old = unsafe { std::slice::from_raw_parts(old, old_length) };
    let diff = unsafe { std::slice::from_raw_parts(diff, diff_length) };

    // TODO: Error handling.
    let result = match diffs::apply(&old, &diff) {
        Ok(result) => result,
        _ => panic!("Error while applying diff"),
    };

    // Copy back response.
    if result.len() <= new_capacity {
        unsafe {
            for i in 0..result.len() as isize {
                std::ptr::write(new.offset(i), result[i as usize]);
            }
            *new_length = result.len();
        };
    }
}

#[no_mangle]
pub extern "C" fn db_state_set(state: *const u8, state_length: usize) {
    profile_block!();

    let state = unsafe { std::slice::from_raw_parts(state, state_length) };

    // TODO: Error handling.
    match Db::instance().import(state) {
        Ok(_) => {}
        _ => panic!("Error while importing state"),
    }
}

#[no_mangle]
pub extern "C" fn db_state_get(state: *mut u8, state_capacity: usize, state_length: *mut usize) {
    profile_block!();

    // TODO: Error handling.
    let result = match Db::instance().export() {
        Ok(state) => state,
        _ => panic!("Error while exporting state"),
    };

    // Copy back response.
    if result.len() <= state_capacity {
        unsafe {
            for i in 0..result.len() as isize {
                std::ptr::write(state.offset(i), result[i as usize]);
            }
            *state_length = result.len();
        };
    }
}
