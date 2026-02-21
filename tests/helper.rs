#![allow(dead_code)]

use scrypto::prelude::*;
use scrypto::prelude::Runtime;
use scrypto_test::prelude::*;

pub struct Helper {
    pub ledger:       DefaultLedgerSimulator,

    pub alice: (ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
    pub bob:   (ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
    pub carol: (ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
    /// Dave is not initially a member.
    pub dave:  (ComponentAddress, Secp256k1PublicKey),

    pub package:      PackageAddress,
    pub component:    ComponentAddress,
    pub member_badge: ResourceAddress,
    /// The internal Account that holds the DAO treasury.
    pub dao_account:  ComponentAddress,
}

impl Helper {
    pub fn new_2_of_2() -> Self { Self::new(2, 2) }
    pub fn new_2_of_3() -> Self { Self::new(2, 3) }
    pub fn new_3_of_3() -> Self { Self::new(3, 3) }

    pub fn new(
        min_cosigners: usize,
        members_number: usize,
    ) -> Self {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let (alice_pk, _, alice_addr) = ledger.new_allocated_account();
        let (bob_pk,   _, bob_addr)   = ledger.new_allocated_account();
        let (carol_pk, _, carol_addr) = ledger.new_allocated_account();
        let (dave_pk,  _, dave_addr)  = ledger.new_allocated_account();

        let package = ledger.compile_and_publish(this_package!());
        
        let members = match members_number {
            2 => indexset![alice_addr, bob_addr],
            3 => indexset![alice_addr, bob_addr, carol_addr],
            _ => Runtime::panic("choose 2 or 3 memebers".to_string())
        };

        // Instantiate DaoWallet.
        // Blueprint: new(members: IndexSet<Global<Account>>, min_cosigners: usize)
        // Returns: (Global<DaoWallet>, ResourceAddress, Global<Account>)
        // In manifests IndexSet<Global<Account>> serialises as Array<Address>.
        let receipt = ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    package,
                    "DaoWallet",
                    "new",
                    manifest_args!(
                        members,
                        min_cosigners
                    ),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&alice_pk)],
        );
        receipt.expect_commit_success();

        let commit = receipt.expect_commit(true);
        let component    = commit.new_component_addresses()[0];
        let dao_account  = commit.new_component_addresses()[1];
        let member_badge = commit.new_resource_addresses()[0];

        // Badge assignment is REVERSED (IndexSet::pop removes last-inserted):
        //   Insertion order: alice(0), bob(1), carol(2)
        //   pop() order:     carol(#1), bob(#2), alice(#3)
        Helper {
            ledger,
            alice: (alice_addr, alice_pk, NonFungibleLocalId::integer(3)),
            bob:   (bob_addr,   bob_pk,   NonFungibleLocalId::integer(2)),
            carol: (carol_addr, carol_pk, NonFungibleLocalId::integer(1)),
            dave:  (dave_addr,  dave_pk),
            package,
            component,
            member_badge,
            dao_account,
        }
    }

    fn signer(pk: &Secp256k1PublicKey) -> Vec<NonFungibleGlobalId> {
        vec![NonFungibleGlobalId::from_public_key(pk)]
    }

    pub fn mint_member_badge(
        &mut self,
        caller:     &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:       &str,
        new_member: ComponentAddress,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "mint_member_badge", |l| {
                (l.proof("p"), desc.to_owned(), new_member)
            })
            .build();
        self.ledger.execute_manifest(manifest, Self::signer(&caller.1))
    }

    pub fn disable_member_badge(
        &mut self,
        caller:    &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:      &str,
        target_id: NonFungibleLocalId,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "disable_member_badge", |l| {
                (l.proof("p"), desc.to_owned(), target_id)
            })
            .build();
        self.ledger.execute_manifest(manifest, Self::signer(&caller.1))
    }

    pub fn enable_member_badge(
        &mut self,
        caller:    &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:      &str,
        target_id: NonFungibleLocalId,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "enable_member_badge", |l| {
                (l.proof("p"), desc.to_owned(), target_id)
            })
            .build();
        self.ledger.execute_manifest(manifest, Self::signer(&caller.1))
    }

    pub fn increase_min_cosigners(
        &mut self,
        caller: &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:   &str,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "increase_min_cosigners", |l| {
                (l.proof("p"), desc.to_owned())
            })
            .build();
        self.ledger.execute_manifest(manifest, Self::signer(&caller.1))
    }

    pub fn decrease_min_cosigners(
        &mut self,
        caller: &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:   &str,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "decrease_min_cosigners", |l| {
                (l.proof("p"), desc.to_owned())
            })
            .build();
        self.ledger.execute_manifest(manifest, Self::signer(&caller.1))
    }

    pub fn send_fungibles(
        &mut self,
        caller:    &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:      &str,
        resource:  ResourceAddress,
        amount:    Decimal,
        recipient: ComponentAddress,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "send_fungibles", |l| {
                (l.proof("p"), desc.to_owned(), resource, amount, recipient)
            })
            .build();
        self.ledger.execute_manifest(manifest, Self::signer(&caller.1))
    }

    pub fn send_non_fungibles(
        &mut self,
        caller:    &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:      &str,
        resource:  ResourceAddress,
        ids:       Vec<NonFungibleLocalId>,
        recipient: ComponentAddress,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "send_non_fungibles", |l| {
                (l.proof("p"), desc.to_owned(), resource, ids, recipient)
            })
            .build();
        self.ledger.execute_manifest(manifest, Self::signer(&caller.1))
    }

    pub fn remove_signature(
        &mut self,
        caller:              &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:                &str,
        op_type:             u8,
        resource_address:    Option<ResourceAddress>,
        fungibles_amount:    Option<Decimal>,
        nft_ids:             Option<Vec<NonFungibleLocalId>>,
        recipient_account:   Option<ComponentAddress>,
        recipient_component: Option<ComponentAddress>,
        validator:           Option<ComponentAddress>,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "remove_signature", |l| {
                (
                    l.proof("p"),
                    desc.to_owned(),
                    op_type,
                    resource_address,
                    fungibles_amount,
                    nft_ids,
                    recipient_account,
                    recipient_component,
                    validator,
                )
            })
            .build();
        self.ledger.execute_manifest(manifest, Self::signer(&caller.1))
    }

    pub fn dao_stake(
        &mut self,
        caller:    &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:      &str,
        amount:    Decimal,
        validator: ComponentAddress,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "stake", |l| {
                (l.proof("p"), desc.to_owned(), amount, validator)
            })
            .build();
        self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&caller.1)])
    }

    pub fn dao_unstake(
        &mut self,
        caller:    &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:      &str,
        amount:    Decimal,
        validator: ComponentAddress,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "unstake", |l| {
                (l.proof("p"), desc.to_owned(), amount, validator)
            })
            .build();
        self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&caller.1)])
    }

    pub fn dao_claim_unstaked_xrd(
        &mut self,
        caller:      &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:        &str,
        claim_ids:   Vec<NonFungibleLocalId>,
        validator:   ComponentAddress,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "claim_unstaked_xrd", |l| {
                (l.proof("p"), desc.to_owned(), claim_ids, validator)
            })
            .build();
        self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&caller.1)])
    }

    pub fn transfer_account_badge(
        &mut self,
        caller:      &(ComponentAddress, Secp256k1PublicKey, NonFungibleLocalId),
        desc:        &str,
        component:   ComponentAddress,
    ) -> TransactionReceiptV1 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                caller.0, self.member_badge, indexset![caller.2.clone()],
            )
            .pop_from_auth_zone("p")
            .call_method_with_name_lookup(self.component, "transfer_account_badge", |l| {
                (l.proof("p"), desc.to_owned(), component)
            })
            .build();
        self.ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&caller.1)])
    }

    pub fn nft_count(&mut self, account: ComponentAddress, resource: ResourceAddress) -> usize {
        self.nft_ids(account, resource).len()
    }

    pub fn nft_ids(
        &mut self,
        account:  ComponentAddress,
        resource: ResourceAddress,
    ) -> Vec<NonFungibleLocalId> {
        let receipt = self.ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(account, "non_fungible_local_ids", manifest_args!(resource, 200u32))
                .build(),
            vec![],
        );
        receipt
            .expect_commit(true)
            .outcome
            .expect_success()
            .get(1)
            .and_then(|output| match output {
                InstructionOutput::CallReturn(bytes) =>
                    scrypto_decode::<Vec<NonFungibleLocalId>>(bytes).ok(),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn xrd_balance(&mut self, account: ComponentAddress) -> Decimal {
        self.ledger.get_component_balance(account, XRD)
    }

    /// Deposit XRD from the faucet into the DAO's internal Account.
    pub fn fund_dao(&mut self) {
        self.ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .try_deposit_entire_worktop_or_abort(self.dao_account, None)
                .build(),
            vec![],
        )
        .expect_commit_success();
    }
}

pub fn has_event(receipt: &TransactionReceiptV1, name: &str) -> bool {
    receipt
        .expect_commit(true)
        .application_events
        .iter()
        .any(|(id, _)| id.1.as_str().contains(name))
}

pub struct ValidatorHelper {
    /// ComponentAddress of the Validator component
    pub addr:             ComponentAddress,
    /// Public key of the validator operator account
    pub owner_pk:         Secp256k1PublicKey,
    /// Account that holds the validator owner badge
    pub owner_addr:       ComponentAddress,
    /// ResourceAddress of the Liquid Staking Units minted on stake()
    pub lsu_resource:     ResourceAddress,
    /// ResourceAddress of the claim NFTs minted on unstake()
    pub claim_nft_resource: ResourceAddress,
}

impl ValidatorHelper {
    pub fn new(env: &mut Helper) -> Self {
        let (owner_pk, _, owner_addr) = env.ledger.new_allocated_account();

        let receipt = env.ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .get_free_xrd_from_faucet()
                .take_from_worktop(XRD, dec!("2000"), "xrd")
                .call_method_with_name_lookup(CONSENSUS_MANAGER, "create_validator", |l| {
                    (Secp256k1PublicKey([42u8; 33]), dec!("0.01"), l.bucket("xrd"))
                })
                .try_deposit_entire_worktop_or_abort(owner_addr, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&owner_pk)],
        );
        receipt.expect_commit_success();
        let validator_addr = receipt.expect_commit(true).new_component_addresses()[0];

        let owner_badge_ids = env.nft_ids(owner_addr, VALIDATOR_OWNER_BADGE);
        assert!(!owner_badge_ids.is_empty(), "owner badge must be minted");
        let owner_badge_id = owner_badge_ids[0].clone();

        env.ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_non_fungibles(
                    owner_addr, VALIDATOR_OWNER_BADGE, indexset![owner_badge_id],
                )
                .call_method(
                    validator_addr,
                    "update_accept_delegated_stake",
                    manifest_args!(true),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&owner_pk)],
        )
        .expect_commit_success();

        let pool_unit_metadata = env.ledger.get_metadata(validator_addr.into(), "pool_unit").unwrap();
        let lsu_global: GlobalAddress = match pool_unit_metadata {
            GenericMetadataValue::GlobalAddress(ga) => ga,
            _ => Runtime::panic("Wrong metadata type".to_string()),
        };
        let lsu_resource = ResourceAddress::try_from(lsu_global).unwrap();

        let claim_nft_metadata = env.ledger.get_metadata(validator_addr.into(), "claim_nft").unwrap();
        let claim_global: GlobalAddress = match claim_nft_metadata {
            GenericMetadataValue::GlobalAddress(ga) => ga,
            _ => Runtime::panic("Wrong metadata type".to_string()),
        };
        let claim_nft_resource = ResourceAddress::try_from(claim_global).unwrap();

        ValidatorHelper {
            addr: validator_addr,
            owner_pk,
            owner_addr,
            lsu_resource,
            claim_nft_resource,
        }
    }
}

pub struct AccountBadgeReceiverHelper {
    // Public key of the owner account
    pub owner_pk: Secp256k1PublicKey,
    // Account that holds the owner badge
    pub owner_addr: ComponentAddress,
    // Package containing the AccountBadgeReceiver blueprint
    pub package: PackageAddress,
    // AccountBadgeReceiver component address
    pub component: ComponentAddress,
}

impl AccountBadgeReceiverHelper {
    pub fn new(env: &mut Helper) -> Self {
        let (owner_pk, _, owner_addr) = env.ledger.new_allocated_account();

        let package = env.ledger.compile_and_publish("account_badge_receiver");

        let receipt = env.ledger.execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    package,
                    "AccountBadgeReceiver",
                    "new",
                    manifest_args!(),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&owner_pk)],
        );
        let component = receipt.expect_commit(true).new_component_addresses()[0];

        AccountBadgeReceiverHelper {
            owner_pk: owner_pk,
            owner_addr: owner_addr,
            package: package,
            component: component,
        }
    }
}
