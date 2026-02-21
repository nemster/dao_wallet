use scrypto::prelude::*;
use scrypto_test::prelude::*;
mod helper;
use helper::Helper;

#[test]
fn test_new_creates_component_and_badge_resource() {
    // Verified by construction; just check addresses are non-zero.
    let env = Helper::new_2_of_3();
    assert_ne!(env.component.as_node_id().0, [0u8; NodeId::LENGTH]);
    assert_ne!(env.dao_account.as_node_id().0, [0u8; NodeId::LENGTH]);
}

#[test]
fn test_new_distributes_one_badge_per_member() {
    let mut env = Helper::new_2_of_3();
    assert_eq!(env.nft_count(env.alice.0, env.member_badge), 1, "Alice");
    assert_eq!(env.nft_count(env.bob.0,   env.member_badge), 1, "Bob");
    assert_eq!(env.nft_count(env.carol.0, env.member_badge), 1, "Carol");
    assert_eq!(env.nft_count(env.dave.0,  env.member_badge), 0, "Dave (non-member)");
}

#[test]
fn test_new_rejects_zero_min_cosigners() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, addr) = ledger.new_allocated_account();
    let pkg = ledger.compile_and_publish(this_package!());
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(pkg, "DaoWallet", "new",
                    manifest_args!(indexset![addr], 0usize))
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit_failure();
}

#[test]
fn test_new_rejects_min_cosigners_of_one() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, addr) = ledger.new_allocated_account();
    let pkg = ledger.compile_and_publish(this_package!());
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(pkg, "DaoWallet", "new",
                    manifest_args!(indexset![addr], 1usize))
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit_failure();
}

#[test]
fn test_new_rejects_min_cosigners_above_member_count() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, addr) = ledger.new_allocated_account();
    let pkg = ledger.compile_and_publish(this_package!());
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(pkg, "DaoWallet", "new",
                    manifest_args!(indexset![addr], 5usize))
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit_failure();
}

#[test]
fn test_new_minimum_valid_2_of_2() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk_a, _, addr_a) = ledger.new_allocated_account();
    let (_,    _, addr_b) = ledger.new_allocated_account();
    let pkg = ledger.compile_and_publish(this_package!());
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(pkg, "DaoWallet", "new",
                    manifest_args!(indexset![addr_a, addr_b], 2usize))
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk_a)],
        )
        .expect_commit_success();
}

