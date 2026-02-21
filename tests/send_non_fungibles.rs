use scrypto::prelude::*;
mod helper;
use helper::Helper;

#[test]
fn test_send_non_fungibles_empty_id_list_fails() {
    let mut env = Helper::new_2_of_3();

    env.send_non_fungibles(
        &env.alice.clone(), "empty", env.member_badge, vec![], env.dave.0,
    )
    .expect_commit_failure();
}

#[test]
fn test_send_non_fungibles_success() {
    let mut env = Helper::new_2_of_3();

    let nft_resource = env.ledger.create_non_fungible_resource(env.dao_account);
    let nft_id       = NonFungibleLocalId::integer(1);

    // Looks like env.ledger.create_non_fungible_resource() created 3 NFTs. Why this?
    let created = env.nft_count(env.dao_account, nft_resource);

    let dave_addr = env.dave.0;

    env.send_non_fungibles(
        &env.alice.clone(), "send nft to dave",
        nft_resource, vec![nft_id.clone()], dave_addr,
    )
    .expect_commit_success();

    env.send_non_fungibles(
        &env.bob.clone(), "send nft to dave",
        nft_resource, vec![nft_id.clone()], dave_addr,
    )
    .expect_commit_success();

    assert_eq!(
        env.nft_ids(dave_addr, nft_resource),
        vec![nft_id.clone()],
        "Dave did not receive the NFT"
    );
    assert_eq!(
        env.nft_ids(env.dao_account, nft_resource).len(),
        created - 1,
        "DAO wallet still owns the NFT"
    );
}

#[test]
fn test_send_non_fungibles_wrong_id_fails() {
    let mut env = Helper::new_2_of_3();

    let nft_resource = env.ledger.create_non_fungible_resource(env.dao_account);
    let wrong_id     = NonFungibleLocalId::integer(99);

    env.send_non_fungibles(
        &env.alice.clone(), "send non existing NFT",
        nft_resource, vec![wrong_id], env.dave.0,
    )
    .expect_commit_failure();
}

