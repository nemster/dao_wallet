use scrypto::prelude::*;
mod helper;
use helper::Helper;
use helper::AccountBadgeReceiverHelper;

#[test]
fn test_transfer_account_badge() {
    let mut env = Helper::new_2_of_3();

    let receiver_helper = AccountBadgeReceiverHelper::new(&mut env);

    env.transfer_account_badge(&env.alice.clone(), "send account badge", receiver_helper.component)
        .expect_commit_success();
    env.transfer_account_badge(&env.bob.clone(), "send account badge", receiver_helper.component)
        .expect_commit_success();

    assert_eq!(
        env.ledger.get_component_balance(env.component, ACCOUNT_OWNER_BADGE),
        dec!("0"),
        "The account owner badge is still in the DAO wallet component"
    );

    assert_eq!(
        env.ledger.get_component_balance(receiver_helper.component, ACCOUNT_OWNER_BADGE),
        dec!("1"),
        "The account owner badge is not in the AccountBadgeReceiver"
    );
}
