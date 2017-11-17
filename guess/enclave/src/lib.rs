// TODO: license

#![crate_name = "guessenclave"]
#![crate_type = "staticlib"]

#![no_std]

extern crate sgx_types;
extern crate sgx_tstd as std;
extern crate sgx_rand as rand;

use rand::Rng;
use std::cmp::Ordering;
use std::result::Result;

enum State {
    Uninitialized,
    Guessing { secret: i32 },
    Guessed,
}

pub enum GuessFeedback {
    Higher,
    Lower,
    Win,
}

static mut state: State = State::Uninitialized;

#[no_mangle]
pub extern "C" fn guess_enclave_init() -> Result<(), &'static str> {
    if let State::Uninitialized = state {
        let secret: i32 = rand::thread_rng().gen_range(1, 101);
        state = State::Guessing { secret };
        Result::Ok
    } else {
        Result::Err("Invalid state")
    }
}

#[no_mangle]
pub extern "C" fn guess_enclave_guess(guess: i32) -> Result<GuessFeedback, &'static str> {
    if let State::Guessing { secret } = state {
        match secret.cmp(guess) {
            Ordering::Greader => Ok(GuessFeedback::Higher),
            Ordering::Less => GuessFeedback::Lower,
            Ordering::Equal => {
                state = State::Guessed;
                GuessFeedback::Win
            },
        }
    } else {
        Err("Invalid state")
    }
}
