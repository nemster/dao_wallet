# DAO Wallet

This blueprint is a wrapper implementing M-of-N multisignature around an Account.  

During component creation, it creates an Account and a badge to manage it, it also creates member badges and sends them directly to the member's accounts (member badges are not transferable).  
It is also decided the number of cosigners that must sign an operation for it to be executed by the component (it's possible to update this number later).  

A member can create an operation of one of these types:  
0. `MintBadge`: create a new member badge and send it directly to the new member account.  
1. `DisableBadge`: disable an existing member badge so that it no longer has voting power and his vote on pending operations is ignored.  
2. `EnableBadge`: enable a existing member badge that has been previously disabled.  
3. `IncreaseMinCosigners`: increase by one the required number of cosigners for an operation.  
4. `DecreaseMinCosigners`: decrease by one the required number of cosigners for an operation.  
5. `SendFungibles`: send an amount of fungibles from the DAO fund to an account.  
6. `SendNonFungibles`: send non fungibles from the DAO fund to an account.  
7. `TransferAccountBadge`: transfer the account owner badge to a new component; this can be used to dismiss this component in favour of a new one.  
8. `Stake`: stake part of the XRD tresury to a validator.  
9. `Unstake`: start unstake process from a validator.  
10. `ClaimUnstakedXrd`: complete the unstake process.  
He also has to specify all of the details of the operation and add a human readable description of the operation.  

When a new operation is created a `NewOperationEvent` event is emitted, it can be used as a call to action for other members.  
Other members can cosign the operation by sumbitting a new transaction with exaclty the same parameters (human readable description included) of the ones used to create the operation.  

When enough members signed an operation, it is executed by the component and a `OperationExecutedEvent` is emitted.  
The `OperationExecutedEvent` contains all of the details of the operation (human readable description included) and the list of the transaction hashes from each cosigner.  
The `OperationExecutedEvent` event can be used as a public log of everything that happened to the funds and the member badges.  

Everyone can deposit any coin in the DAO fund just by depositing in the Account managed by the component.  

## `new`
Create a DaoWallet component and the managed account, distribute the member badges, set the initial cosigners number.  
```
CALL_FUNCTION
    Address("<PACKAGE_ADDRESS>")
    "DaoWallet"
    "new"
    Array<Address>(
        Address("<MEMBER_ACCOUNT>"),
        ...
    )
    <MIN_COSIGNERS>u64
;
```
`<PACKAGE_ADDRESS>`: the address of the package containing the `DaoWallet` blueprint.  
`<MEMBER_ACCOUNT>`: one of the accounts to send the member badges to. Any number of accounts can be specified (up to some transaction limit in minting and sending badges). Recipient accounts must have third party deposits enabled.  
`<MIN_COSIGNERS>`: the minimum number of cosigners (including the first one) required for an operation to be executed.  

## `mint_member_badge`
A member can invoke this method to create/sign an operation to mint a new member badge and send it to the specified account.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "mint_member_badge"
    Proof("member_proof")
    "<DESCRIPTION>"
    Address("<NEW_MEMBER_ACCOUNT>")
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation.  
`<NEW_MEMBER_ACCOUNT>`: account address of the new member to sent the badge to.  

## `disable_member_badge`
A member can invoke this method to create/sign an operation to disable a member badge.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "disable_member_badge"
    Proof("member_proof")
    "<DESCRIPTION>"
    NonFungibleLocalId("#<TARGET_MEMBER_BADGE_ID>#")
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation.  
`<TARGET_MEMBER_BADGE_ID>`: the numeric id of the member badge to disable.  

## `enable_member_badge`
A member can invoke this method to create/sign an operation to enable a member badge.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "enable_member_badge"
    Proof("member_proof")
    "<DESCRIPTION>"
    NonFungibleLocalId("#<TARGET_MEMBER_BADGE_ID>#")
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation.  
`<TARGET_MEMBER_BADGE_ID>`: the numeric id of the member badge to enable.  

## `increase_min_cosigners`
A member can invoke this method to create/sign an operation to increase the minimum number of cosigners for future operations.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "increase_min_cosigners"
    Proof("member_proof")
    "<DESCRIPTION>"
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation.  

## `decrease_min_cosigners`
A member can invoke this method to create/sign an operation to decrease the minimum number of cosigners for future operations.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "decrease_min_cosigners"
    Proof("member_proof")
    "<DESCRIPTION>"
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation.  

## `send_fungibles`
A member can invoke this method to create/sign an operation to send a bucket of fungibles to an account.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "send_fungibles"
    Proof("member_proof")
    "<DESCRIPTION>"
    Address("<FUNGIBLES_ADDRESS>")
    Decimal("<FUNGIBLES_AMOUNT>")
    Address("<RECIPIENT_ACCOUNT>")
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation.  
`<FUNGIBLES_ADDRESS>`: resource address of the fungibles to send.  
`<FUNGIBLES_AMOUNT>`: amount of fungibles to send.  
`<RECIPIENT_ACCOUNT>`: account address to send the fungibles to.  

## `send_non_fungibles`
A member can invoke this method to create/sign an operation to send a bucket of non fungibles to an account.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "send_non_fungibles"
    Proof("member_proof")
    "<DESCRIPTION>"
    Address("<NON_FUNGIBLES_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("<NON_FUNGIBLE_ID>"),
        ...
    )
    Address("<RECIPIENT_ACCOUNT>")
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation.  
`<NON_FUNGIBLES_ADDRESS>`: resource address of the non fungibles to send.  
`<NON_FUNGIBLE_ID>`: id of one of the non fungibles to send (including `#` or `{` or `"` depending on the non fungible id type). Any number of non fungibles can be sent in a single operation (until some transaction limit is hit).    
`<RECIPIENT_ACCOUNT>`: account address to send the fungibles to.  

## `transfer_account_badge`
A member can invoke this method to create/sign an operation to dismiss this component and send the account badge to a new one.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "send_non_fungibles"
    Proof("member_proof")
    "<DESCRIPTION>"
    Address("<RECIPIENT_COMPONENT>")
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.    
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation.  
`<RECIPIENT_COMPONENT>`: address of the component to transfer the account badge to.  
The receiving component must have a method like this:  
```
pub fn deposit_account_badge(
    &mut self,
    account_badge_bucket: NonFungibleBucket
)
```

## `stake`
A member can invoke this method to create/sign an operation to stake some XRD to a validator.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "stake"
    Proof("member_proof")
    "<DESCRIPTION>"
    Decimal("<XRD_AMOUNT>")
    Address("<VALIDATOR>")
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation to unsign.  
`<XRD_AMOUNT>`: the XRD amount to stake.  
`<VALIDATOR>`: the address of the validator to stake to.  

## `unstake`
A member can invoke this method to create/sign an operation to unstake some XRD from a validator.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "unstake"
    Proof("member_proof")
    "<DESCRIPTION>"
    Decimal("<LSU_AMOUNT>")
    Address("<VALIDATOR>")
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation to unsign.  
`<LSU_AMOUNT>`: the amount of LSU to unstake.  
`<VALIDATOR>`: the address of the validator to unstake from.  

## `claim_unstaked_xrd`
A member can invoke this method to create/sign an operation to complete the unstake of some XRD from a validator.  
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "claim_unstaked_xrd"
    Proof("member_proof")
    "<DESCRIPTION>"
    Array<NonFungibleLocalId>(NonFungibleLocalId("{<CLAIM_NFT_ID>}"), ...)
    Address("<VALIDATOR>")
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation to unsign.  
`<CLAIM_NFT_ID>`: the id of one of the claim NFT to use; any number of claim NFT can be claimed in a single operation up to hitting some transaction limit.  
`<VALIDATOR>`: the address of the validator to claim XRD from.  

## `remove_signature`
A member can invoke this method to remove his signature from one of the pending operations.
```
CALL_METHOD
    Address("<MEMBER_ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<MEMBER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<MEMBER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("member_proof")
;
CALL_METHOD
    Address("<DAO_WALLET_COMPONENT>")
    "remove_signature"
    Proof("member_proof")
    "<DESCRIPTION>"
    <OPERATION_TYPE>u8
    Some(Address("<RESOURCE_ADDRESS>"))
    Some(Decimal("<FUNGIBLES_AMOUNT>"))
    Some(Array<NonFungibleLocalId>(NonFungibleLocalId("<NON_FUNGIBLE_ID>"), ...))
    Some(Address("<RECIPIENT_ACCOUNT>"))
    Some(Address("<RECIPIENT_COMPONENT>"))
    Some(Address("<VALIDATOR>"))
;
```
`<MEMBER_ACCOUNT>`: the account address of the member signing the operation.  
`<MEMBER_BADGE_ADDRESS>`: resource address of the member badges.  
`<MEMBER_BADGE_ID>`: the numeric id of the badge of the member signing the operation.  
`<DAO_WALLET_COMPONENT>`: the component created through the `new` function.  
`<DESCRIPTION>`: human readable description of the operation to unsign.  
`<OPERATION_TYPE>`: a number from 0 to 10 representing the type of the operation to unsign.  
`<RESOURCE_ADDRESS>`: resource address for the `SendFungibles` and `SendNonFungibles` operations, replace the whole line with `None` otherwise.  
`<FUNGIBLES_AMOUNT>`: amount of fungibles for the `SendFungibles` operation, replace the whole line with `None` otherwise.  
`<NON_FUNGIBLE_ID>`: id of one of the non fungibles for the `SendNonFungibles`, `DisableBadge` and `EnableBadge` operations, replace the whole line with `None` otherwise.  
`<RECIPIENT_ACCOUNT>`: account address of the recipient for `SendFungibles`, `SendNonFungibles` and `MintBadge` operations, replace the whole line with `None` otherwise.  
`<RECIPIENT_COMPONENT>`: component address for the `TransferAccountBadge` operations, replace the whole line with `None` otherwise.  
`<VALIDATOR>`: address of the validator for the `Stake`, `Unstake` and `ClaimUnstakedXrd` operations, replace the whole line with `None` otherwise.  
 

