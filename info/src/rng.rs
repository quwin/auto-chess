use scrypto::prelude::*;

pub fn seed(min: u128, max: u128) -> u128 {
    let magnitude: u128 = max-min;
    let pseudorandom_number: u128 = Runtime::generate_uuid();
    let seed = pseudorandom_number % magnitude + min;
    seed
}
