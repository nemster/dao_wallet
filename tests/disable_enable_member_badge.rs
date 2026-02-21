use scrypto::prelude::*;
mod helper;
use helper::Helper;
use helper::has_event;

#[test]
fn test_disable_badge_2_of_3_executes_with_two_sigs() {
    let mut env = Helper::new_2_of_3();
    let bob_id = env.bob.2.clone();

    let r = env.disable_member_badge(&env.alice.clone(), "disable bob", bob_id.clone());
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));

    let r = env.disable_member_badge(&env.bob.clone(), "disable bob", bob_id.clone());
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));

    // Bob's badge has been disabled, he can no longer sign operations
    let r = env.enable_member_badge(&env.bob.clone(), "enable bob", bob_id);
    r.expect_commit_failure();
}

#[test]
fn test_disable_badge_two_members() {
    let mut env = Helper::new_2_of_2();
    let bob_id = env.bob.2.clone();

    // If only two members exist, it's not possible to disable one of them
    env.disable_member_badge(&env.alice.clone(), "disable bob", bob_id.clone())
        .expect_commit_failure();
}

#[test]
fn test_disable_nonexistent_badge_fails() {
    let mut env = Helper::new_2_of_3();
    env.disable_member_badge(
        &env.alice.clone(), "ghost", NonFungibleLocalId::integer(999),
    )
    .expect_commit_failure();
}

#[test]
fn test_disabled_member_vote_is_ignored() {
    // Sequence that proves disabled badges votes are removed
    //  1. Bob signs add-dave first (1 vote: bob)
    //  2. Alice + Carol disable Bob (2-of-3; bob's badge is now disabled)
    //  3. Carol signs add-dave → retain() strips Bob's vote → 1 active vote (carol)
    //  4. Alice signs add-dave → 2nd active vote → executes
    let mut env = Helper::new_2_of_3();
    let bob_id    = env.bob.2.clone();
    let dave_addr = env.dave.0;

    // Step 1: Bob proposes add-dave
    env.mint_member_badge(&env.bob.clone(), "add dave", dave_addr)
        .expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 0, "Still pending");

    // Step 2: Alice and Carol disable Bob (alice first, carol second)
    env.disable_member_badge(&env.alice.clone(), "disable bob", bob_id.clone())
        .expect_commit_success();
    env.disable_member_badge(&env.carol.clone(), "disable bob", bob_id)
        .expect_commit_success();

    // Step 3: Carol co-signs add-dave; retain() removes Bob's vote → still only 1 active
    env.mint_member_badge(&env.carol.clone(), "add dave", dave_addr)
        .expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 0, "Bob purged; only Carol active");

    // Step 4: Alice makes it 2 active votes → executes
    env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr)
        .expect_commit_success();
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1);
}

#[test]
fn test_enable_badge_after_disable() {
    let mut env = Helper::new_2_of_3();
    let bob_id = env.bob.2.clone();

    // Disable (2 sigs needed)
    let r = env.disable_member_badge(&env.alice.clone(), "disable bob", bob_id.clone());
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.disable_member_badge(&env.bob.clone(), "disable bob", bob_id.clone());
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));

    // Re-enable (2 sigs needed)
    let r = env.enable_member_badge(&env.alice.clone(), "enable bob", bob_id.clone());
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.enable_member_badge(&env.carol.clone(), "enable bob", bob_id);
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));
}

#[test]
fn test_enable_already_active_badge_fails() {
    let mut env = Helper::new_2_of_3();
    let bob_id = env.bob.2.clone();

    env.enable_member_badge(&env.alice.clone(), "enable active bob", bob_id)
        .expect_commit_failure();
}
