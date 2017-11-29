// TODO: license

#![crate_name = "guessenclave"]
#![crate_type = "staticlib"]

#![no_std]

extern crate sgx_types;
extern crate sgx_tstd as std;
extern crate sgx_rand as rand;

use core::cmp::Ordering;
use core::result::Result;
use rand::Rng;

#[derive(Copy, Clone)]
pub enum State {
    Guessing { secret: i32 },
    Guessed,
}

#[derive(Copy, Clone)]
pub enum GuessFeedback {
    Higher,
    Lower,
    Win,
}

#[no_mangle]
pub extern "C" fn guess_enclave_init(result: *mut Result<State, &'static str>) {
    std::backtrace::enable_backtrace("enclave.signed.so", std::backtrace::PrintFormat::Short).expect("Couldn't enable backtrace");
    let result = unsafe { &mut *result };
    let secret: i32 = rand::thread_rng().gen_range(1, 101);
    *result = Ok(State::Guessing { secret });
}

#[no_mangle]
pub extern "C" fn guess_enclave_guess(result: *mut Result<(State, GuessFeedback), &'static str>, state: *const State, guess: i32) {
    let result = unsafe { &mut *result };
    let state = unsafe { &*state };
    *result = if let State::Guessing { secret } = *state {
        match secret.cmp(&guess) {
            Ordering::Greater => Ok((*state, GuessFeedback::Higher)),
            Ordering::Less => Ok((*state, GuessFeedback::Lower)),
            Ordering::Equal => Ok((State::Guessed, GuessFeedback::Win)),
        }
    } else {
        Err("Invalid state")
    };
}
