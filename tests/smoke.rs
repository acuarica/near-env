use near_env::{near_envlog, near_envlog_skip_args};

mod near_sdk {
    pub mod env {
        use std::io::{self, Write};

        pub fn log(message: &[u8]) {
            io::stdout().write_all(message).unwrap();
            println!("");
        }

        pub fn predecessor_account_id() -> String {
            "<pred>".to_string()
        }
    }
}

struct Model {}

#[allow(dead_code)]
struct NoDisplay {}

#[near_envlog]
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
    pub fn self_fn_skip_args(&self, _no_display: NoDisplay) -> u32 {
        42
    }

    #[near_envlog_skip_args]
    #[near_envlog]
    pub fn self_fn_two_args(&self, an_arg: u32, another_arg: u16) -> u32 {
        2
    }

    pub fn mut_self_fn_no_args(&mut self) -> u32 {
        self.priv_self(false);
        3
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
    // near_sdk::env::log(b"asdf");

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
