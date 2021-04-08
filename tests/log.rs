use std::convert::TryInto;

use near_env::{near_envlog, near_envlog_skip_args};
use near_sdk::{
    near_bindgen,
    test_utils::{get_logs, VMContextBuilder},
    testing_env, MockedBlockchain,
};

#[near_bindgen]
pub struct Model {}

#[near_envlog]
impl Default for Model {
    fn default() -> Self {
        Model {}
    }
}

#[allow(dead_code)]
struct NoDisplay {}

#[near_envlog]
#[near_bindgen]
#[near_envlog(skip_args)]
#[near_envlog(only_pub)]
#[near_envlog(skip_args, only_pub)]
impl Model {
    #[init]
    pub fn new() -> Model {
        Model {}
    }

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

    pub fn self_fn_two_args(&self, an_arg: u32, another_arg: u16) -> u32 {
        2 + an_arg + another_arg as u32
    }

    #[near_envlog_skip_args]
    pub fn self_fn_skip_two_args(&self, an_arg: u32, another_arg: u16) -> u32 {
        2 + an_arg + another_arg as u32
    }

    #[payable]
    pub fn mut_self_fn_no_args(&mut self) -> u32 {
        self.priv_self(false);
        let a = near_sdk::env::attached_deposit();
        3 + a as u32
    }

    pub fn mut_self_fn_one_arg(&mut self, an_arg: bool) -> u32 {
        4
    }

    fn priv_self(&self, _an_arg: bool) -> u32 {
        4
    }
}

trait ModelTrait {
    fn trait_method(&self, arg: u32) -> u32;
}

#[near_envlog]
#[near_envlog(skip_args, only_pub)]
impl ModelTrait for Model {
    fn trait_method(&self, arg: u32) -> u32 {
        arg
    }
}

#[test]
fn test_logs() {
    let context = VMContextBuilder::new()
        .signer_account_id("bob_near".try_into().unwrap())
        .is_view(false)
        .build();
    testing_env!(context);

    let mut logs = Vec::new();

    let ver = env!("CARGO_PKG_VERSION");

    let l = format!("default() v{}", ver);
    logs.push(l.as_ref());
    Model::default();

    logs.push("new");
    logs.push("new()");
    logs.push("new");
    let l = format!("new() v{}", ver);
    logs.push(l.as_str());
    let mut model = Model::new();

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

    logs.push("self_fn_two_args");
    logs.push("self_fn_two_args(an_arg: 1, another_arg: 2)");
    logs.push("self_fn_two_args");
    logs.push("self_fn_two_args(an_arg: 1, another_arg: 2)");
    model.self_fn_two_args(1, 2);

    logs.push("mut_self_fn_no_args");
    logs.push("mut_self_fn_no_args() pred: bob.near");
    logs.push("mut_self_fn_no_args");
    logs.push("mut_self_fn_no_args() pred: bob.near, deposit: 0");
    logs.push("priv_self");
    logs.push("priv_self(_an_arg: false)");
    model.mut_self_fn_no_args();

    logs.push("mut_self_fn_one_arg");
    logs.push("mut_self_fn_one_arg(an_arg: true) pred: bob.near");
    logs.push("mut_self_fn_one_arg");
    logs.push("mut_self_fn_one_arg(an_arg: true) pred: bob.near");
    model.mut_self_fn_one_arg(true);

    logs.push("trait_method");
    logs.push("trait_method(arg: 2)");
    model.trait_method(2);

    let env_logs = get_logs();
    for log_line in &env_logs {
        println!("{}", log_line);
    }

    for i in 0..logs.len() {
        assert_eq!(logs[i], env_logs[i]);
    }

    assert_eq!(logs, env_logs);
}
