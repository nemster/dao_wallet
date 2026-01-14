use scrypto::prelude::*;
use crate::cosigner::*;

/* List of possible operation types a member can sign.
 */
#[derive(ScryptoSbor, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum OperationType {

    // mint_member_badge method
    MintBadge = 0,

    // disable_member_badge method
    DisableBadge = 1,

    // enable_member_badge method
    EnableBadge = 2,

    // increase_min_cosigners method
    IncreaseMinCosigners = 3,

    // decrease_min_cosigners method
    DecreaseMinCosigners = 4,

    // send_fungibles method
    SendFungibles = 5,

    // send_non_fungibles method
    SendNonFungibles = 6,

    // transfer_account_badge method
    TransferAccountBadge = 7,

    // stake method
    Stake = 8,

    // unstake method
    Unstake = 9,

    // claim_unstaked_xrd method
    ClaimUnstakedXrd = 10,
}

/* An Operation is the internal representation of an operation a member can sign.
 */
#[derive(ScryptoSbor, PartialEq)]
pub struct Operation {

    // Human readable representation of the operation
    description: String,

    // Type of operation
    operation_type: OperationType,

    // Resource address (SendFungibles and SendNonFungibles types) or None
    resource: Option<ResourceAddress>,

    // Amount (SendFungibles, Stake, and Unstake types) or None
    amount: Option<Decimal>,

    // Non fungible ids (DisableBadge, EnableBadge, SendNonFungibles and ClaimUnstakedXrd type) or None
    non_fungible_ids: Option<Vec<NonFungibleLocalId>>,

    // Recipent account address (MintBadge, SendFungibles, SendNonFungibles types) or None
    recipient: Option<Global<Account>>,

    // Recipient component address (TransferAccountBadge type) or None
    component: Option<Global<AnyComponent>>,

    // Validator address (Stake, Unstake and ClaimUnstakedXrd type) or None
    validator: Option<Global<Validator>>,
}

/* This event is emitted when an operation is signed by the first cosigner.
 */
#[derive(ScryptoSbor, ScryptoEvent)]
pub struct NewOperationEvent {
    description: String,
    operation_type: OperationType,
    resource: Option<ResourceAddress>,
    amount: Option<Decimal>,
    non_fungible_ids: Option<Vec<NonFungibleLocalId>>,
    recipient: Option<Global<Account>>,
    component: Option<Global<AnyComponent>>,
    validator: Option<Global<Validator>>,

    // Member badge id of the first signer of the operation
    signer_badge_id: NonFungibleLocalId,
}

/* This event is emitted when an operation has enough cosigners and is actually executed.
 */
#[derive(ScryptoSbor, ScryptoEvent)]
pub struct OperationExecutedEvent {
    description: String,
    operation_type: OperationType,
    resource: Option<ResourceAddress>,
    amount: Option<Decimal>,
    non_fungible_ids: Option<Vec<NonFungibleLocalId>>,
    recipient: Option<Global<Account>>,
    component: Option<Global<AnyComponent>>,
    validator: Option<Global<Validator>>,

    // List of cosigners, including badge ids and transaction hashes
    cosigners: Cosigners,
}

/* u8 to OperationType conversion.
 * Input parameters:
 * - orig: numeric representation of the operation type
 * Output:
 * - an element of the OperationType enum
 */
impl From<u8> for OperationType {
    fn from(orig: u8) -> Self {
        match orig {
            0 => return OperationType::MintBadge,
            1 => return OperationType::DisableBadge,
            2 => return OperationType::EnableBadge,
            3 => return OperationType::IncreaseMinCosigners,
            4 => return OperationType::DecreaseMinCosigners,
            5 => return OperationType::SendFungibles,
            6 => return OperationType::SendNonFungibles,
            7 => return OperationType::TransferAccountBadge,
            8 => return OperationType::Stake,
            9 => return OperationType::Unstake,
            10 => return OperationType::ClaimUnstakedXrd,
            _  => Runtime::panic("Unknown operation type".to_string()),
        }
    }
}

impl Operation {

    /* Create a new operation.
     * Input parameters:
     * - description: human readable representation of the operation
     * - operation_type: type of operation
     * - resource: resource address to send (SendFungibles and SendNonFungibles types) or None
     * - amount: amount to send (SendFungibles type) or None
     * - non_fungible_ids: list of non fungibles to send (SendNonFungibles type) or None
     * - recipient: recipent account address (MintBadge, SendFungibles, SendNonFungibles types) or None
     * - component: recipient component address (TransferAccountBadge type) or None
     * Outputs:
     * - the Operation object
     */
    pub fn new(
        description: String,
        operation_type: OperationType,
        resource: Option<ResourceAddress>,
        amount: Option<Decimal>,
        non_fungible_ids: Option<Vec<NonFungibleLocalId>>,
        recipient: Option<Global<Account>>,
        component: Option<Global<AnyComponent>>,
        validator: Option<Global<Validator>>,
    ) -> Operation {
        Operation {
            description: description.trim().to_string(),
            operation_type: operation_type,
            resource: resource,
            amount: amount,
            non_fungible_ids: non_fungible_ids,
            recipient: recipient,
            component: component,
            validator: validator,
        }
    }

    /* Emit the NewOperationEvent.
     * Input parameters:
     * - signer_badge_id: id of the member badge that signed the operation
     */
    pub fn emit_new_event(
        &self,
        signer_badge_id: NonFungibleLocalId,
    ) {
        Runtime::emit_event(
            NewOperationEvent {
                description: self.description.clone(),
                operation_type: self.operation_type,
                resource: self.resource,
                amount: self.amount,
                non_fungible_ids: self.non_fungible_ids.clone(),
                recipient: self.recipient,
                component: self.component,
                validator: self.validator,
                signer_badge_id: signer_badge_id,
            }
        );
    }

    /* Emit the OperationExecutedEvent.
     * Input parameters:
     * - cosigners: list of signers of the executed operation
     */
    pub fn emit_executed_event(
        &self,
        cosigners: Cosigners,
    ) {
        Runtime::emit_event(
            OperationExecutedEvent {
                description: self.description.clone(),
                operation_type: self.operation_type,
                resource: self.resource,
                amount: self.amount,
                non_fungible_ids: self.non_fungible_ids.clone(),
                recipient: self.recipient,
                component: self.component,
                validator: self.validator,
                cosigners: cosigners,
            }
        );
    }
}
