// Sample implementation of a blueprint that can receive the account badge from the DAO wallet

use scrypto::prelude::*;

#[blueprint]
mod AccountBadgeReceiver {
    struct AccountBadgeReceiver {
        account_badge_vault: NonFungibleVault,
    }

    impl AccountBadgeReceiver {
        pub fn new(
        ) -> Global<AccountBadgeReceiver> {
            Self {
                account_badge_vault: NonFungibleVault::new(ACCOUNT_OWNER_BADGE),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }
        
        pub fn deposit_account_badge(
            &mut self,
            account_badge_bucket: NonFungibleBucket,
        ) {
            self.account_badge_vault.put(account_badge_bucket);
        }

    }
}
