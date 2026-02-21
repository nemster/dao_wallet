use scrypto::prelude::*;
use scrypto_test::prelude::*;
mod helper;
use helper::Helper;
use helper::has_event;

#[test]
fn test_mint_badge_2_of_3_first_sig_is_pending() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;
    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr)
        .expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 0, "Should be pending");
}

#[test]
fn test_mint_badge_2_of_3_second_sig_executes() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;
    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr)
        .expect_commit_success();
    env.mint_member_badge(&env.bob.clone(), "add dave", dave_addr)
        .expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1);
}

#[test]
fn test_mint_badge_double_sign_same_member_fails() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;
    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr)
        .expect_commit_success();
    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr)
        .expect_commit_failure();
}

#[test]
fn test_mint_badge_description_mismatch_creates_separate_op() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;
    // Two ops with different descriptions → each needs its own 2nd sig
    env.mint_member_badge(&env.alice.clone(), "add dave A", dave_addr)
        .expect_commit_success();
    env.mint_member_badge(&env.bob.clone(), "add dave B", dave_addr)
        .expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 0);
}

#[test]
fn test_3_of_3_requires_all_three_sigs() {
    let mut env = Helper::new_3_of_3();
    let dave_addr = env.dave.0;

    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr).expect_commit_success();
    env.mint_member_badge(&env.bob.clone(),   "add dave", dave_addr).expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 0, "2 of 3 not enough");

    env.mint_member_badge(&env.carol.clone(), "add dave", dave_addr).expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1);
}

#[test]
fn test_2_of_3_any_two_members_can_execute() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;
    // Carol + Bob (not alice)
    env.mint_member_badge(&env.carol.clone(), "add dave", dave_addr).expect_commit_success();
    env.mint_member_badge(&env.bob.clone(),   "add dave", dave_addr).expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1);
}

#[test]
fn test_new_member_can_cosign_after_badge_mint() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;
    let dave_pk   = env.dave.1;

    // Mint Dave's badge (alice + bob cosign)
    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr).expect_commit_success();
    env.mint_member_badge(&env.bob.clone(),   "add dave", dave_addr).expect_commit_success();

    let dave_ids = env.nft_ids(dave_addr, env.member_badge);
    assert_eq!(dave_ids.len(), 1);
    let dave_id = dave_ids[0].clone();

    // Dave proposes an operation
    env.fund_dao();
    let member_badge = env.member_badge;
    let component    = env.component;
    let alice_addr   = env.alice.0;

    let receipt = env.ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                dave_addr, member_badge, indexset![dave_id],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(component, "send_fungibles", |l| {
                (l.proof("p"), "dave pays alice".to_owned(), XRD, dec!("10"), alice_addr)
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&dave_pk)],
    );
    receipt.expect_commit_success();
    assert!(has_event(&receipt, "NewOperationEvent"));
}

