use near_env::near_envlog;
use near_sdk::{near_bindgen, testing_env, MockedBlockchain, VMContext};

#[near_envlog]
fn wrapped_function(first_param: u32, snd_param: u16) -> u32 {
    first_param + snd_param as u32
}

#[near_bindgen]
struct A {}

#[near_bindgen]
#[near_envlog]
impl A {
    pub fn mmm(&mut self) -> u32 {
        64
    }

    pub fn mm(&mut self, fp: bool) -> u32 {
        64
    }

    pub fn m(&self) -> u32 {
        42
    }

    #[near_envlog]
    pub fn self_fn(&self, first_param: u32, snd_param: u16) -> u32 {
        self.m();
        first_param + snd_param as u32
    }
}

fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
    VMContext {
        current_account_id: "alice_near".to_string(),
        signer_account_id: "bob_near".to_string(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id: "carol_near".to_string(),
        input,
        block_index: 0,
        block_timestamp: 0,
        account_balance: 0,
        account_locked_balance: 0,
        storage_usage: 0,
        attached_deposit: 0,
        prepaid_gas: 10u64.pow(18),
        random_seed: vec![0, 1, 2],
        is_view,
        output_data_receivers: vec![],
        epoch_height: 0,
    }
}

#[test]
fn works() {
    let context = get_context(vec![], false);
    testing_env!(context);

    let res = wrapped_function(42, 8);
    println!("res: {}", res);

    A {}.self_fn(1, 2);
    A {}.mm(true);
    A {}.mmm();
}
