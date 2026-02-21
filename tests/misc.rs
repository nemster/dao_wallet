use scrypto::prelude::*;
use scrypto_test::prelude::*;
mod helper;
use helper::Helper;
use helper::has_event;

#[test]
fn test_replay_attack() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;

    let r = env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));

    let r = env.mint_member_badge(&env.bob.clone(),   "add dave", dave_addr);
    r.expect_commit_success();
    assert!(!has_event(&r, "NewOperationEvent"));
    assert!(has_event(&r, "OperationExecutedEvent"));

    // After execution, carol signs. This causes a new round, the operation is not executed again 
    let r = env.mint_member_badge(&env.carol.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(!has_event(&r, "OperationExecutedEvent"));
}

#[test]
fn test_double_signature() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;

    let r = env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));

    // Alice tries to sign again the same operation
    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr).expect_commit_failure();
}

#[test]
fn test_wrong_badge_resource_rejected() {
    let mut env = Helper::new_2_of_3();

    let fake_badge = env.ledger.create_non_fungible_resource(env.alice.0);
    let fake_ids   = env.nft_ids(env.alice.0, fake_badge);
    let fake_id    = fake_ids[0].clone();

    let alice_pk   = env.alice.1;
    let alice_addr = env.alice.0;
    let component  = env.component;

    env.ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_non_fungibles(
                    alice_addr, fake_badge, indexset![fake_id],
                )
                .pop_from_auth_zone("p")
                .call_method_with_name_lookup(component, "increase_min_cosigners", |l| {
                    (l.proof("p"), "hack".to_owned())
                })
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&alice_pk)],
        )
        .expect_commit_failure();
}

#[test]
fn test_two_independent_ops_coexist_and_both_execute() {
    let mut env = Helper::new_2_of_3();
    env.fund_dao();
    let dave_addr = env.dave.0;
    let dave_initial_balance = env.xrd_balance(dave_addr);

    // Two independent ops pending simultaneously
    let r = env.send_fungibles(&env.alice.clone(), "send xrd", XRD, dec!("100"), dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    
    // Complete both
    let r = env.send_fungibles(&env.bob.clone(), "send xrd", XRD, dec!("100"), dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));
    let r = env.mint_member_badge(&env.bob.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));

    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1);
    assert_eq!(env.xrd_balance(dave_addr) - dave_initial_balance, dec!("100"));
}

#[test]
fn test_different_descriptions_are_independent_ops() {
    let mut env = Helper::new_2_of_3();
    env.fund_dao();
    let dave_addr = env.dave.0;
    let dave_initial_balance = env.xrd_balance(dave_addr);

    // Two ops identical except for description
    let r = env.send_fungibles(&env.alice.clone(), "payment A", XRD, dec!("100"), dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));

    let r = env.send_fungibles(&env.alice.clone(), "payment B", XRD, dec!("100"), dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));

    // Bob cosigns only payment-A
    let r = env.send_fungibles(&env.bob.clone(), "payment A", XRD, dec!("100"), dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));

    // Only payment-A executed
    assert_eq!(env.xrd_balance(dave_addr) - dave_initial_balance, dec!("100"));
}

#[test]
fn test_same_description_different_recipient_is_different_op() {
    let mut env = Helper::new_2_of_3();
    env.fund_dao();

    let alice_addr = env.alice.0;
    let dave_addr  = env.dave.0;

    let alice_initial_balance = env.xrd_balance(alice_addr);
    let dave_initial_balance = env.xrd_balance(alice_addr);
    let dao_initial_balance = env.xrd_balance(env.dao_account);

    // Same description + amount, different recipient → different op hash
    let r = env.send_fungibles(&env.alice.clone(), "pay 100", XRD, dec!("100"), alice_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.send_fungibles(&env.alice.clone(), "pay 100", XRD, dec!("100"), dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));

    // Bob cosigns the alice-recipient op
    let r = env.send_fungibles(&env.bob.clone(), "pay 100", XRD, dec!("100"), alice_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));

    // Only payment to Alice has been executed
    assert!(dao_initial_balance - env.xrd_balance(env.dao_account) == dec!("100"), "Only one payment executed");
    assert!(env.xrd_balance(dave_addr) == dave_initial_balance, "Dave received no XRD");
    assert!(env.xrd_balance(alice_addr) - alice_initial_balance == dec!("100"), "Alice got 100 XRD");
}

