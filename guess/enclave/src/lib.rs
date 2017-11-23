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

pub enum State {
    Guessing { secret: i32 },
    Guessed,
}

pub enum GuessFeedback {
    Higher,
    Lower,
    Win,
}

#[no_mangle]
pub extern "C" fn guess_enclave_init() -> Result<State, &'static str> {
    let secret: i32 = rand::thread_rng().gen_range(1, 101);
    Ok(State::Guessing { secret })
}

#[no_mangle]
pub extern "C" fn guess_enclave_guess(state: State, guess: i32) -> Result<(State, GuessFeedback), &'static str> {
    if let State::Guessing { secret } = state {
        match secret.cmp(&guess) {
            Ordering::Greater => Ok((state, GuessFeedback::Higher)),
            Ordering::Less => Ok((state, GuessFeedback::Lower)),
            Ordering::Equal => Ok((State::Guessed, GuessFeedback::Win)),
        }
    } else {
        Err("Invalid state")
    }
}
