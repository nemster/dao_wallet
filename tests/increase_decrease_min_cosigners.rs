use scrypto::prelude::*;
mod helper;
use helper::Helper;
use helper::has_event;

#[test]
fn test_increase_min_cosigners_2_to_3() {
    let mut env = Helper::new_2_of_3();

    // Both alice and bob sign (2 sigs needed)
    let r = env.increase_min_cosigners(&env.alice.clone(), "inc to 3");
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.increase_min_cosigners(&env.bob.clone(), "inc to 3");
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));

    // Now 3 sigs needed: alice+bob alone should NOT execute
    let dave_addr = env.dave.0;
    let r = env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.mint_member_badge(&env.bob.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(!has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 0, "Needs 3 sigs now");

    // Carol makes it 3 → executes
    let r = env.mint_member_badge(&env.carol.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1);
}

#[test]
fn test_increase_min_cosigners_when_already_at_max_fails() {
    // 3-of-3: guard is enabled_members(3) > min_cosigners(3) → false → panics
    let mut env = Helper::new_3_of_3();

    env.increase_min_cosigners(&env.alice.clone(), "increase min cosigners")
        .expect_commit_failure();
}

#[test]
fn test_decrease_min_cosigners_from_2_fails() {
    // Blueprint asserts min_cosigners > 2 before allowing decrease
    let mut env = Helper::new_2_of_3();

    env.decrease_min_cosigners(&env.alice.clone(), "decrease min cosigners")
        .expect_commit_failure();
}

#[test]
fn test_decrease_min_cosigners_3_to_2() {
    let mut env = Helper::new_3_of_3();

    // All 3 must sign the decrease (3-of-3)
    let r = env.decrease_min_cosigners(&env.alice.clone(), "dec to 2");
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.decrease_min_cosigners(&env.bob.clone(), "dec to 2");
    r.expect_commit_success();
    assert!(!has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.decrease_min_cosigners(&env.carol.clone(), "dec to 2");
    r.expect_commit_success();
    assert!(!has_event(&r, "NewOperationEvent"));
    assert!(has_event(&r, "OperationExecutedEvent"));

    // Now 2-of-3: alice+bob alone executes
    let dave_addr = env.dave.0;
    let r = env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.mint_member_badge(&env.bob.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1);
}

#[test]
fn test_increase_then_decrease_round_trip() {
    // Start 2-of-3, increase to 3-of-3, decrease back to 2-of-3
    let mut env = Helper::new_2_of_3();

    // Increase to 3 (needs 2 sigs at current threshold)
    let r = env.increase_min_cosigners(&env.alice.clone(), "inc");
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.increase_min_cosigners(&env.bob.clone(), "inc");
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));

    // Now 3-of-3. Decrease back to 2 (needs 3 sigs)
    let r = env.decrease_min_cosigners(&env.alice.clone(), "dec");
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.decrease_min_cosigners(&env.bob.clone(), "dec");
    r.expect_commit_success();
    assert!(!has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.decrease_min_cosigners(&env.carol.clone(), "dec");
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));

    // Back to 2-of-3: alice+bob execute
    let dave_addr = env.dave.0;
    let r = env.mint_member_badge(&env.alice.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "NewOperationEvent"));
    assert!(!has_event(&r, "OperationExecutedEvent"));
    let r = env.mint_member_badge(&env.bob.clone(), "add dave", dave_addr);
    r.expect_commit_success();
    assert!(has_event(&r, "OperationExecutedEvent"));
    assert_eq!(env.nft_count(dave_addr, env.member_badge), 1);
}

