use scrypto::prelude::*;
mod helper;
use helper::Helper;
use helper::ValidatorHelper;

#[test]
fn test_stake_unstake() {
    let mut env = Helper::new_2_of_3();
    env.fund_dao();
    let v = ValidatorHelper::new(&mut env);

    env.dao_stake(&env.alice.clone(), "stake 300", dec!("300"), v.addr)
        .expect_commit_success();
    env.dao_stake(&env.bob.clone(), "stake 300", dec!("300"), v.addr)
        .expect_commit_success();

    let lsu_bal = env.ledger.get_component_balance(env.dao_account, v.lsu_resource);
    assert!(lsu_bal > dec!("0"), "No LSU found");

    env.dao_unstake(&env.alice.clone(), "unstake all", lsu_bal, v.addr)
        .expect_commit_success();
    env.dao_unstake(&env.bob.clone(), "unstake all", lsu_bal, v.addr)
        .expect_commit_success();

    let lsu_bal = env.ledger.get_component_balance(env.dao_account, v.lsu_resource);
    assert!(lsu_bal == dec!("0"), "The LSU are still there");

    let claim_ids = env.nft_ids(env.dao_account, v.claim_nft_resource);
    assert_eq!(claim_ids.len(), 1, "No Claim NFT found");
}
