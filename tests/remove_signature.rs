use scrypto::prelude::*;
mod helper;
use helper::Helper;

#[test]
fn test_remove_signature_cancels_vote() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;

    // Alice signs
    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr)
        .expect_commit_success();

    // Alice withdraws
    env.remove_signature(
        &env.alice.clone(), "add dave", 0u8,
        None, None, None, Some(dave_addr), None, None,
    )
    .expect_commit_success();

    // Bob signs (1 effective vote – op still pending)
    env.mint_member_badge(&env.bob.clone(), "add dave", dave_addr)
        .expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 0, "Only 1 effective vote");

    // Carol makes it 2 → executes
    env.mint_member_badge(&env.carol.clone(), "add dave", dave_addr)
        .expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1);
}

#[test]
fn test_remove_signature_nonexistent_op_fails() {
    let mut env = Helper::new_2_of_3();
    env.remove_signature(
        &env.alice.clone(), "ghost op", 0u8,
        None, None, None, Some(env.dave.0), None, None,
    )
    .expect_commit_failure();
}

#[test]
fn test_remove_signature_by_non_signer_fails() {
    let mut env = Helper::new_2_of_3();
    let dave_addr = env.dave.0;
    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr)
        .expect_commit_success();

    // Carol was not a signer; this fails
    env.remove_signature(
        &env.carol.clone(), "add dave", 0u8,
        None, None, None, Some(dave_addr), None, None,
    )
    .expect_commit_failure();

    // Alice's vote is still there: bob co-signing executes the op
    env.mint_member_badge(&env.bob.clone(), "add dave", dave_addr)
        .expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1, "Op still executed normally");
}

