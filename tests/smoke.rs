use near_env::{near_envlog, near_envlog_skip_args};
use near_sdk::{
    env, near_bindgen,
    test_utils::{get_logs, VMContextBuilder},
    testing_env, MockedBlockchain,
};

#[near_bindgen]
pub struct Model {}

#[allow(dead_code)]
struct NoDisplay {}

#[near_envlog]
#[near_bindgen]
#[near_envlog(skip_args)]
#[near_envlog(only_pub)]
#[near_envlog(skip_args, only_pub)]
impl Model {
    pub fn self_fn_no_args(&self) -> u32 {
        1
    }

    pub fn self_fn_str_arg(&self, str_arg: String) -> String {
        str_arg
    }

    #[near_envlog_skip_args]
    fn self_fn_skip_args(&self, _no_display: NoDisplay) -> u32 {
        42
    }

    #[near_envlog_skip_args]
    #[near_envlog]
    pub fn self_fn_two_args(&self, an_arg: u32, another_arg: u16) -> u32 {
        2
    }

    #[payable]
    pub fn mut_self_fn_no_args(&mut self) -> u32 {
        self.priv_self(false);
        let a = env::attached_deposit();
        3 + a as u32
    }

    #[near_envlog]
    pub fn mut_self_fn_one_arg(&mut self, an_arg: bool) -> u32 {
        4
    }

    fn priv_self(&self, _an_arg: bool) -> u32 {
        4
    }
}

#[near_envlog]
fn fn_no_args() -> u32 {
    42
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

#[test]
fn works() {
    let context = VMContextBuilder::new()
        .signer_account_id("bob_near".to_string())
        .is_view(false)
        .build();
    testing_env!(context);

    let mut logs = Vec::new();

    logs.push("fn_no_args()");
    fn_no_args();

    logs.push("wrapped_function(first_param: 42, snd_param: 8)");
    let res = wrapped_function(42, 8);
    println!("res: {}", res);
    assert_eq!(res, 50);

    logs.push("free_standing_fn_skip_args");
    free_standing_fn_skip_args(1, 2);

    let mut model = Model {};

    logs.push("self_fn_no_args");
    logs.push("self_fn_no_args()");
    logs.push("self_fn_no_args");
    logs.push("self_fn_no_args()");
    let res = model.self_fn_no_args();
    assert_eq!(res, 1);

    logs.push("self_fn_str_arg");
    logs.push("self_fn_str_arg(str_arg: a value)");
    logs.push("self_fn_str_arg");
    logs.push("self_fn_str_arg(str_arg: a value)");
    let res = model.self_fn_str_arg("a value".to_string());
    assert_eq!(res, "a value");

    logs.push("self_fn_skip_args");
    logs.push("self_fn_skip_args");
    model.self_fn_skip_args(NoDisplay {});

    logs.push("self_fn_two_args(an_arg: 1, another_arg: 2)");
    logs.push("self_fn_two_args");
    logs.push("self_fn_two_args");
    logs.push("self_fn_two_args");
    logs.push("self_fn_two_args");
    model.self_fn_two_args(1, 2);

    logs.push("mut_self_fn_no_args");
    logs.push("mut_self_fn_no_args()pred: bob.near");
    logs.push("mut_self_fn_no_args");
    logs.push("mut_self_fn_no_args()pred: bob.near, deposit: 0");
    logs.push("priv_self");
    logs.push("priv_self(_an_arg: false)");
    model.mut_self_fn_no_args();

    logs.push("mut_self_fn_one_arg(an_arg: true), pred: bob.near");
    logs.push("mut_self_fn_one_arg");
    logs.push("mut_self_fn_one_arg(an_arg: true), pred: bob.near");
    logs.push("mut_self_fn_one_arg");
    logs.push("mut_self_fn_one_arg(an_arg: true), pred: bob.near");
    model.mut_self_fn_one_arg(true);

    for log_line in get_logs() {
        println!("{}", log_line);
    }

    assert_eq!(logs, get_logs());
}
