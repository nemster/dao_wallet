use scrypto::prelude::*;
mod helper;
use helper::Helper;

#[test]
fn test_send_fungibles_2_of_3_first_sig_pending() {
    let mut env = Helper::new_2_of_3();
    env.fund_dao();
    let dave_addr = env.dave.0;
    let before = env.xrd_balance(dave_addr);

    env.send_fungibles(&env.alice.clone(), "pay dave", XRD, dec!("100"), dave_addr)
        .expect_commit_success();
    assert_eq!(env.xrd_balance(dave_addr), before, "Not yet executed");
}

#[test]
fn test_send_fungibles_2_of_3_executes_on_second_sig() {
    let mut env = Helper::new_2_of_3();
    env.fund_dao();
    let dave_addr = env.dave.0;
    let before = env.xrd_balance(dave_addr);

    env.send_fungibles(&env.alice.clone(), "pay dave", XRD, dec!("100"), dave_addr)
        .expect_commit_success();
    env.send_fungibles(&env.bob.clone(), "pay dave", XRD, dec!("100"), dave_addr)
        .expect_commit_success();

    assert_eq!(env.xrd_balance(dave_addr) - before, dec!("100"));
}

#[test]
fn test_send_fungibles_insufficient_balance_fails() {
    let mut env = Helper::new_2_of_3();
    env.fund_dao();
    // check_availability will panic: "Not enough funds!"
    env.send_fungibles(&env.alice.clone(), "drain", XRD, dec!("10001"), env.dave.0)
        .expect_commit_failure();
}

#[test]
fn test_send_fungibles_zero_amount_fails() {
    let mut env = Helper::new_2_of_3();
    env.fund_dao();
    // check_availability: "Amount must be positive"
    env.send_fungibles(&env.alice.clone(), "zero", XRD, dec!("0"), env.dave.0)
        .expect_commit_failure();
}

#[test]
fn test_send_fungibles_two_concurrent_ops() {
    let mut env = Helper::new_2_of_3();
    env.fund_dao();
    let dave_addr = env.dave.0;
    let before = env.xrd_balance(dave_addr);

    // Two independent ops (different descriptions), each signed by alice first
    env.send_fungibles(&env.alice.clone(), "pay A", XRD, dec!("100"), dave_addr)
        .expect_commit_success();
    env.send_fungibles(&env.alice.clone(), "pay B", XRD, dec!("200"), dave_addr)
        .expect_commit_success();

    // Complete both
    env.send_fungibles(&env.bob.clone(), "pay A", XRD, dec!("100"), dave_addr)
        .expect_commit_success();
    env.send_fungibles(&env.bob.clone(), "pay B", XRD, dec!("200"), dave_addr)
        .expect_commit_success();

    assert_eq!(env.xrd_balance(dave_addr) - before, dec!("300"));
}

