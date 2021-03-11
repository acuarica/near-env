use near_env::{near_envlog, near_envlog_skip_args};
use near_sdk::{near_bindgen, testing_env, MockedBlockchain, VMContext};

#[near_bindgen]
struct Model {}

#[allow(dead_code)]
struct NoDisplay {}

#[near_bindgen]
#[near_envlog]
impl Model {
    pub fn self_fn_no_args(&self) -> u32 {
        1
    }

    pub fn self_fn_str_arg(&self, str_arg: String) -> String {
        str_arg
    }

    #[near_envlog_skip_args]
    pub fn self_fn_skip_args(&self, _no_display: NoDisplay) -> u32 {
        42
    }

    #[near_envlog_skip_args]
    #[near_envlog]
    pub fn self_fn_two_args(&self, an_arg: u32, another_arg: u16) -> u32 {
        2
    }

    pub fn mut_self_fn_no_args(&mut self) -> u32 {
        3
    }

    pub fn mut_self_fn_one_arg(&mut self, an_arg: bool) -> u32 {
        4
    }
}

#[near_envlog]
fn wrapped_function(first_param: u32, snd_param: u16) -> u32 {
    first_param + snd_param as u32
}

#[near_envlog]
#[near_envlog_skip_args]
fn free_standing_fn_skip_args(first_param: u32, snd_param: u16) -> u32 {
    first_param + snd_param as u32
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

    free_standing_fn_skip_args(1, 2);

    let mut model = Model {};
    model.self_fn_no_args();
    model.self_fn_str_arg("a value".to_string());
    model.self_fn_skip_args(NoDisplay {});
    model.self_fn_two_args(1, 2);
    model.mut_self_fn_no_args();
    model.mut_self_fn_one_arg(true);
}
