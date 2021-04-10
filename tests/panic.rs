#![deny(warnings)]

use near_env::PanicMessage;
use near_sdk::{serde::Serialize, test_utils::VMContextBuilder, testing_env, MockedBlockchain};

#[derive(Serialize, PanicMessage)]
#[serde(crate = "near_sdk::serde", tag = "errkey")]
enum P {
    #[panic_msg = "err no fields"]
    ErrNoFields,
    #[panic_msg = "err no fields 2"]
    ErrNoFields2,
    #[panic_msg = "err a with f1 {}"]
    ErrA { f1: u32 },
    #[panic_msg = "err b with f1 {} and f2 {}"]
    ErrB { f1: u32, f2: String },
}

fn init_context() {
    use std::convert::TryInto;

    let context = VMContextBuilder::new()
        .signer_account_id("bob_near".try_into().unwrap())
        .is_view(false)
        .build();
    testing_env!(context);
}

#[test]
#[should_panic(expected = "err no fields")]
fn err_no_fields_should_panic() {
    init_context();
    P::ErrNoFields.panic();
}

#[test]
#[should_panic(expected = "err no fields 2")]
fn err_no_fields2_should_panic() {
    init_context();
    P::ErrNoFields2.panic();
}

#[test]
#[should_panic(expected = "err a with f1 42")]
fn err_a_should_panic() {
    init_context();
    P::ErrA { f1: 42 }.panic();
}

#[test]
#[should_panic(expected = "err b with f1 42 and f2 f2 for err-b")]
fn err_b_should_panic() {
    init_context();
    P::ErrB {
        f1: 42,
        f2: "f2 for err-b".to_string(),
    }
    .panic();
}
