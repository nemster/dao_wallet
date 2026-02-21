use scrypto::prelude::*;
use crate::cosigner::*;
use crate::operation::*;
use crate::member::*;

/* A DaoWallet is a wrapper around an Account that implements M-of-N multisignature.
 * A single operation must be signed by min_cosigners members to be executed; each member executes
 * a transaction specifying the same parameters.
 */
#[blueprint]
#[types(
    Operation,
    Cosigners,
)]
#[events(
    NewOperationEvent,
    OperationExecutedEvent,
)]
mod dao_wallet {
    struct DaoWallet {

        // ResourceManager to mint member badges
        member_badges_resource_manager: NonFungibleResourceManager,

        // The minimum number of cosigners to execute an operation
        min_cosigners: usize,

        // The account containing the DAO treasure
        account: Global<Account>,

        // The vault containing the owner badge of the account containing the DAO treasure
        account_badge: NonFungibleVault,

        // Container of the past and pending operations
        operations: KeyValueStore<Operation, Cosigners>,

        // The numeric non fungible id of the next member badge to mint
        next_badge_id: u64,
    }

    impl DaoWallet {

        /* Instantiate a DaoWallet component, mint member badges, send them to the member
         * accounts, set the number of signatures needed for a transaction to be executed, create
         * an account to hold the DAO treasury and a badge to manage it.
         * Input parameters:
         * - members: list of the accounts that will receive the member badges
         * - min_cosigners: the number of different member badges needed to sign an operation
         * Outputs:
         * - the DaoWallet component
         * - the resource address of the account badges
         * - the account to contain the DAO treasure
         */
        pub fn new(
            mut members: IndexSet<Global<Account>>,
            min_cosigners: usize,
        ) -> (
            Global<DaoWallet>,
            ResourceAddress,
            Global<Account>,
        ) {

            // Verify that input parameters make sense
            assert!(
                min_cosigners > 1,
                "Do not set less than one cosigner"
            );
            assert!(
                min_cosigners <= members.len(),
                "Not enough members"
            );

            // Reserve a component address to set permissions
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(DaoWallet::blueprint_id());

            // Prepare the list of member badges to create
            let mut member_badges_specification = vec![];
            for n in 1..=members.len() {
                member_badges_specification.push(
                    (
                        IntegerNonFungibleLocalId::from(n as u64),
                        Member::new()
                    )
                );
            }

            // Create member badges
            let mut member_badges_bucket = ResourceBuilder::new_integer_non_fungible::<Member>(
                OwnerRole::None
            )
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(deny_all);
                        metadata_setter_updater => rule!(deny_all);
                        metadata_locker => rule!(deny_all);
                        metadata_locker_updater => rule!(deny_all);
                    },
                    init {
                        "name" => "DAO wallet badge", locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                ))
                // Member badges are non transferable (soul bound)
                .withdraw_roles(withdraw_roles!(
                    withdrawer => rule!(deny_all);
                    withdrawer_updater => rule!(deny_all);
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(require(global_caller(component_address)));
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .mint_initial_supply(member_badges_specification);

            // Get the ResourceAddressand the ResourceManager of the member badges
            let member_badges_address = member_badges_bucket.resource_address();
            let member_badges_resource_manager =
                NonFungibleResourceManager::from(member_badges_address);

            // Get ready to mint more member badges
            let next_badge_id = members.len() as u64 + 1;

            // Send badges to the members accounts
            let mut id: u64 = 1;
            while let Some(mut member) = members.pop() {
                member.try_deposit_or_abort(
                    member_badges_bucket.take_non_fungible(
                        &NonFungibleLocalId::Integer(id.into())
                    ).into(),
                    None
                );

                id += 1;
            }

            // Burn the empty bucket
            member_badges_bucket.burn();

            // Create an Account to hold the DAO treasury and a badge to manage it
            let (account, account_badge) = Blueprint::<Account>::create();

            // Instantiate the DaoWallet component
            let dao_wallet = Self {
                member_badges_resource_manager: member_badges_resource_manager,
                min_cosigners: min_cosigners,
                account: account,
                account_badge: NonFungibleVault::with_bucket(account_badge),
                operations: KeyValueStore::new_with_registered_type(),
                next_badge_id: next_badge_id,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .globalize();

            // Return all information
            (dao_wallet, member_badges_address, account)
        }

        /* Private method to validate a member badge proof and get the numeric id of the used
         * member badge.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * Outputs:
         * - the non fungible id of the member badge used to create the proof
         */
        fn get_badge_id(
            &self,
            member_badge_proof: Proof,
        ) -> NonFungibleLocalId {

            // Verify the ResourceAddress and find the member badge
            let non_fungible = member_badge_proof.check_with_message(
                self.member_badges_resource_manager.address(),
                "Incorrect proof",
            )
                .as_non_fungible()
                .non_fungible::<Member>();

            // Verify that the member badge is enabled
            assert!(
                non_fungible.data().is_enabled(),
                "Disabled badge"
            );

            // Return the non fungible local id of the badge
            non_fungible.local_id().clone()
        }

        /* Private method to add a cosigner to the list of those who signed an operation.
         * Returns true if min_cosigners has been reached (the operation can be executed).
         * Input parameters:
         * - operation: the operation to sign
         * - badge_id: the non fungible id of the member badge signing the operation
         * Outputs:
         * - true if enough signatures exist, false otherwise
         */
        fn add_cosigner(
            &mut self,
            operation: Operation,
            badge_id: NonFungibleLocalId,
        ) -> bool {

            // Create the cosigner
            let cosigner = Cosigner::new(badge_id.clone());

            // Check if the operation already exists
            let mut opt_cosigners = self.operations.get_mut(&operation);
            match opt_cosigners {

                // If not
                None => {

                    // Emit the NewOperationEvent
                    operation.emit_new_event(badge_id);

                    // Avoid multiple borrows error
                    drop(opt_cosigners);

                    // Add the new operation, with a single signer, to the list
                    self.operations.insert(
                        operation,
                        indexset!(cosigner),
                    );
                },

                // If yes
                Some(ref mut cosigners) => {

                    // Remove eventual cosigners whose badge has been disabled in the meantime
                    cosigners.retain(
                        |cosigner: &Cosigner| {
                            cosigner.is_enabled(&self.member_badges_resource_manager)
                        }
                    );

                    // Add the new cosigner to the list, fail if it was already there
                    assert!(
                        cosigners.insert(cosigner),
                        "You already signed this operation"
                    );
            
                    // If the operation has enough cosigners emit the OperationExecutedEvent,
                    // reset the list of cosigners and return true
                    if cosigners.len() >= self.min_cosigners {
                        operation.emit_executed_event(cosigners.clone());
                        cosigners.clear();
                        return true;
                    }
                },
            }

            // Return false
            false
        }

        /* Internal method to verify that it is possible to withdraw the specified resource from
         * the account.
         * There's no output, the method panics if the operation is impossible.
         * Input parameters:
         * - resource: the address of the resource to withdraw
         * - amount: the amount to withdraw if resource is fungible, None otherways
         * - non_fungible_ids: the list of non fungibles to withdraw or None
         */
        fn check_availability(
            &self,
            resource: ResourceAddress,
            amount: Option<Decimal>,
            non_fungible_ids: Option<&Vec<NonFungibleLocalId>>,
        ) {

            // The account badge is needed in order to withdraw anything
            assert!(
                !self.account_badge.is_empty(),
                "The account badge is missing"
            );

            // Fungibles/non fungible specific checks
            match resource.is_fungible() {
                true => {
                    let amount = amount.expect("No amount specified");

                    assert!(
                        amount > Decimal::ZERO,
                        "Amount must be positive"
                    );
                    assert!(
                        self.account.balance(resource) >= amount,
                        "Not enough funds!"
                    );
                    assert!(
                        non_fungible_ids.is_none(),
                        "Fungible/non fungible mismatch"
                    );
                },

                false => {
                    let non_fungible_ids = non_fungible_ids.expect("No ids specified");

                    assert!(
                        non_fungible_ids.len() > 0,
                        "I'm not sending zero NFTs"
                    );
                    for non_fungible_id in non_fungible_ids.iter() {
                        assert!(
                            self.account.has_non_fungible(
                                resource,
                                non_fungible_id.clone()
                            ),
                            "We don't have that NFT"
                        );
                    }
                    assert!(
                        amount.is_none(),
                        "Fungible/non fungible mismatch"
                    );
                }
            }
        }

        /* A member can invoke this method to create/sign an operation to mint a new member badge
         * and send it to the specified account.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - recipient: the account address to send the new member badge to
         */
        pub fn mint_member_badge(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            mut recipient: Global<Account>,
        ) {

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::MintBadge,
                None,
                None,
                None,
                Some(recipient),
                None,
                None,
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, create a new member badge
                let member_badge_bucket = self.member_badges_resource_manager.mint_non_fungible(
                    &NonFungibleLocalId::Integer(self.next_badge_id.into()),
                    Member::new()
                );

                // Send the new member badge to the specified account
                recipient.try_deposit_or_abort(
                    member_badge_bucket.into(),
                    None
                );

                // Get ready for minting the next member badge
                self.next_badge_id += 1;
            }
        }

        /* A member can invoke this method to create/sign an operation to disable a member badge.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - non_fungible_id: the id of the member badge to disable
         */
        pub fn disable_member_badge(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            non_fungible_id: NonFungibleLocalId,
        ) {

            // Make sure that the specified member badge is currently enabled
            assert!(
                Member::is_badge_enabled(
                    &non_fungible_id,
                    &self.member_badges_resource_manager,
                ),
                "Member badge already disabled"
            );

            // Make sure that a sufficient number of member badges will be enabled after the
            // operation
            let enabled_members = Member::count_enabled_members(&self.member_badges_resource_manager);
            assert!(
                enabled_members > self.min_cosigners,
                "Not enough enabled members remaining"
            );

            // Get the id of the signing member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::DisableBadge,
                None,
                None,
                Some(vec![non_fungible_id.clone()]),
                None,
                None,
                None,
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, disable the specified member badge
                Member::enable(
                    &non_fungible_id,
                    false,
                    &self.member_badges_resource_manager,
                );
            }
        }

        /* A member can invoke this method to create/sign an operation to enable a member badge
         * that has been previously disabled.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - non_fungible_id: the id of the member badge to enable
         */
        pub fn enable_member_badge(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            non_fungible_id: NonFungibleLocalId,
        ) {

            // Make sure that the specified member badge is currently disabled
            assert!(
                !Member::is_badge_enabled(
                    &non_fungible_id,
                    &self.member_badges_resource_manager,
                ),
                "Member badge already enabled"
            );

            // Get the id of the signing member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::EnableBadge,
                None,
                None,
                Some(vec![non_fungible_id.clone()]),
                None,
                None,
                None,
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, enable the specified member badge
                Member::enable(
                    &non_fungible_id,
                    true,
                    &self.member_badges_resource_manager,
                );
            }
        }

        /* A member can invoke this method to create/sign an operation to increase the minimum
         * number of cosigners for future operations.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         */
        pub fn increase_min_cosigners(
            &mut self,
            member_badge_proof: Proof,
            description: String,
        ) {

            // Make sure that enough members exist to sign future operatons after the threshold increase
            let enabled_members = Member::count_enabled_members(&self.member_badges_resource_manager);
            assert!(
                enabled_members > self.min_cosigners,
                "Not enough enabled members"
            );

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::IncreaseMinCosigners,
                None,
                None,
                None,
                None,
                None,
                None,
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, increase the number of required
                // cosigners
                self.min_cosigners += 1;
            }
        }

        /* A member can invoke this method to create/sign an operation to decrease the minimum
         * number of cosigners for future operations.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         */
        pub fn decrease_min_cosigners(
            &mut self,
            member_badge_proof: Proof,
            description: String,
        ) {

            // Do not allow the number of cosigners be 1 or less
            assert!(
                self.min_cosigners > 2,
                "Remaining cosigners must be more than 1"
            );

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::DecreaseMinCosigners,
                None,
                None,
                None,
                None,
                None,
                None,
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, decrease the number of required
                // cosigners
                self.min_cosigners -= 1;
            }
        }

        /* A member can invoke this method to create/sign an operation to send a bucket of
         * fungibles to an account.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - resource: resource address of the fungibles to send
         * - amount: the amount of fungibles to send
         * - recipient: the account address to send the fungibles to
         */
        pub fn send_fungibles(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            resource: ResourceAddress,
            amount: Decimal,
            mut recipient: Global<Account>,
        ) {

            // Verify that the operation is possible
            self.check_availability(
                resource,
                Some(amount),
                None,
            );

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::SendFungibles,
                Some(resource),
                Some(amount),
                None,
                Some(recipient),
                None,
                None,
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, use the account badge to withdraw
                // the specified fungibles from the DAO treasury and put them in a bucket
                let coin_bucket = self.account_badge.authorize_with_non_fungibles(
                    &self.account_badge.non_fungible_local_ids(1),
                    || self.account.withdraw(
                        resource,
                        amount
                    )
                );

                // Try to send the bucket to the specified account
                recipient.try_deposit_or_abort(
                    coin_bucket,
                    None
                );
            }
        }

        /* A member can invoke this method to create/sign an operation to send a bucket of
         * non fungibles to an account.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - resource: resource address of the non fungibles to send
         * - non_fungible_ids: the list of non fungible ids to send
         * - recipient: the account address to send the non fungibles to
         */
        pub fn send_non_fungibles(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            resource: ResourceAddress,
            non_fungible_ids: Vec<NonFungibleLocalId>,
            mut recipient: Global<Account>,
        ) {

            // Verify that the operation is possible
            self.check_availability(
                resource,
                None,
                Some(&non_fungible_ids),
            );

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::SendNonFungibles,
                Some(resource),
                None,
                Some(non_fungible_ids.clone()),
                Some(recipient),
                None,
                None,
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, use the account badge to withdraw
                // the specified non fungibles from the DAO treasury and put them in a bucket
                let non_fungibles_bucket = self.account_badge.authorize_with_non_fungibles(
                    &self.account_badge.non_fungible_local_ids(1),
                    || self.account.withdraw_non_fungibles(
                        resource,
                        non_fungible_ids
                    )
                );

                // Try to send the bucket to the specified account
                recipient.try_deposit_or_abort(
                    non_fungibles_bucket.into(),
                    None
                );
            }
        }

        /* A member can invoke this method to remove his signature from an operation.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - operation_type: numeric identifier of the operation type
         * - resource: the address of the resource to send or None
         * - amount: the amount of fungibles to send of None
         * - non_fungible_ids: the list of id of the non fungibles to send or None
         * - recipient: address of the account to send the resources to or None
         * - component: address of the component to send the account badge to or None
         */
        pub fn remove_signature(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            operation_type: u8,
            resource: Option<ResourceAddress>,
            amount: Option<Decimal>,
            non_fungible_ids: Option<Vec<NonFungibleLocalId>>,
            recipient: Option<Global<Account>>,
            component: Option<Global<AnyComponent>>,
            validator: Option<Global<Validator>>,
        ) {

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                operation_type.into(),
                resource,
                amount,
                non_fungible_ids,
                recipient,
                component,
                validator,
            );

            // Create the cosigner
            let cosigner = Cosigner::new(badge_id);

            // Remove the cosigner from the list for the specified operation
            let was_present = self.operations
                .get_mut(&operation)
                .expect("Operation not found")
                .swap_remove(&cosigner);
            assert!(
                was_present,
                "Non existing signature"
            );
        }

        /* A member can invoke this method to create/sign an operation to dismiss this component
         * and send the account badge to a new one.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - component: address of the component to send the account badge
         */
        pub fn transfer_account_badge(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            component: Global<AnyComponent>,
        ) {

            // Verify that the operation is possible
            assert!(
                !self.account_badge.is_empty(),
                "The account badge is already gone"
            );

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::TransferAccountBadge,
                None,
                None,
                None,
                None,
                Some(component),
                None,
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, put the account badge in a bucket
                let account_badge_bucket = self.account_badge.take_all();

                // Deposit the bucket in the specified component
                component.call_ignore_rtn::<(NonFungibleBucket, )>(
                    "deposit_account_badge",
                    &(account_badge_bucket, )
                );
            }
        }

        /* A member can invoke this method to create/sign an operation to stake some XRD to a
         * validator.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - amount: the XRD amount to stake
         * - validator: address of the validator to stake to
         */
        pub fn stake(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            amount: Decimal,
            mut validator: Global<Validator>,
        ) {

            // Verify that the operation is possible
            self.check_availability(
                XRD,
                Some(amount),
                None,
            );
            assert!(
                validator.accepts_delegated_stake(),
                "This validator doesn't accept user stake"
            );

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::Stake,
                None,
                Some(amount),
                None,
                None,
                None,
                Some(validator),
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, use the account badge to withdraw
                // the XRD from the DAO treasury, stake them and put the LSU back into the account
                self.account_badge.authorize_with_non_fungibles(
                    &self.account_badge.non_fungible_local_ids(1),
                    || {
                        let xrd_bucket = self.account.withdraw(
                            XRD,
                            amount,
                        );

                        let lsu_bucket = validator.stake(FungibleBucket(xrd_bucket));

                        self.account.deposit(lsu_bucket.into());
                    }
                );
            }
        }

        /* A member can invoke this method to create/sign an operation to start the unstake of
         * some XRD from a validator.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - amount: the LSU amount to unstake
         * - validator: address of the validator to unstake from
         */
        pub fn unstake(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            amount: Decimal,
            mut validator: Global<Validator>,
        ) {

            // Find the lsu resource address for the specified validator
            let lsu_global_address: GlobalAddress = validator.get_metadata("pool_unit").ok().unwrap().unwrap();
            let lsu_address = ResourceAddress::try_from(lsu_global_address).unwrap();

            // Verify that the operation is possible
            self.check_availability(
                lsu_address,
                Some(amount),
                None,
            );

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::Unstake,
                None,
                Some(amount),
                None,
                None,
                None,
                Some(validator),
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, use the account badge to withdraw
                // the LSU from the DAO treasury, unstake them and put the claim NFT back into
                // the account
                self.account_badge.authorize_with_non_fungibles(
                    &self.account_badge.non_fungible_local_ids(1),
                    || {
                        let lsu_bucket = self.account.withdraw(
                            lsu_address,
                            amount,
                        );

                        let claim_nft_bucket = validator.unstake(FungibleBucket(lsu_bucket));

                        self.account.deposit(claim_nft_bucket.into());
                    }
                );
            }
        }

        /* A member can invoke this method to create/sign an operation to complete the unstake of
         * some XRD from a validator.
         * Input parameters:
         * - member_badge_proof: proof of ownership of a member badge
         * - description: human readable description of the operation
         * - non_fungible_ids: ids of the NFT to claim
         * - validator: address of the validator to unstake from
         */
        pub fn claim_unstaked_xrd(
            &mut self,
            member_badge_proof: Proof,
            description: String,
            non_fungible_ids: Vec<NonFungibleLocalId>,
            mut validator: Global<Validator>,
        ) {

            // Find the claim NFT resource address for the specified validator
            let claim_nft_global_address: GlobalAddress = validator.get_metadata("claim_nft").ok().unwrap().unwrap();
            let claim_nft_address = ResourceAddress::try_from(claim_nft_global_address).unwrap();

            // Verify that the operation is possible
            self.check_availability(
                claim_nft_address,
                None,
                Some(&non_fungible_ids),
            );
            //TODO: check claim epoch?

            // Get the id of the member badge
            let badge_id = self.get_badge_id(member_badge_proof);

            // Create the operation
            let operation = Operation::new(
                description.clone(),
                OperationType::ClaimUnstakedXrd,
                None,
                None,
                Some(non_fungible_ids.clone()),
                None,
                None,
                Some(validator),
            );

            // Register the operation in the operations KVS or add the cosigner if it's
            // already there
            if self.add_cosigner(operation, badge_id) {

                // If enough members signed the operation, use the account badge to withdraw
                // the Claim NFT from the DAO treasury, claim the XRD and put them back into
                // the account
                self.account_badge.authorize_with_non_fungibles(
                    &self.account_badge.non_fungible_local_ids(1),
                    || {
                        let claim_nft_bucket = self.account.withdraw_non_fungibles(
                            claim_nft_address,
                            non_fungible_ids,
                        );

                        let xrd_bucket = validator.claim_xrd(claim_nft_bucket);

                        self.account.deposit(xrd_bucket.into());
                    }
                );
            }
        }
    }
}
