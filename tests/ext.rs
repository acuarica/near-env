#![deny(warnings)]

use near_env::near_ext;
use near_sdk::ext_contract;

#[near_ext]
#[ext_contract(a)]
trait I {
    fn m(&self) -> u32;
}

struct A {}

impl I for A {
    fn m(&self) -> u32 {
        42
    }
}

#[test]
fn trait_a_should_be_defined() {
    assert_eq!(A {}.m(), 42);
}
