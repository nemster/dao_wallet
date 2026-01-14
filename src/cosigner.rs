use scrypto::prelude::*;
use scrypto::prelude::rust::hash::Hasher;
use scrypto::prelude::rust::hash::Hash;
use crate::member::*;

/* A Cosigner is the internal representation of the signature of an operation by one of the
 * members.
 */
#[derive(ScryptoSbor, Clone)]
pub struct Cosigner {

    // Non fungible id of the member badge of the signer
    badge_id: NonFungibleLocalId,

    // The hash of the transaction used to sign the operation
    transaction_hash: scrypto::prelude::Hash,
    // TODO: how to obtain in the event a nice string representation such as
    // txid_tdx_2_1wms4dj33lcckwx9jsg8xrvc6wujygg9at94grhruw72zy082hylq58dugt instead of
    // 76e156ca31fe316718b2820e61b31a77244420bd596a81dc7c7794223ceab93e ?
    // Do we better leave the conversion task to the frontend reading the events?
}

/* Cosigners is the list of the Cosigner of an operation
 */
pub type Cosigners = IndexSet<Cosigner>;

impl Cosigner {

    /* Create a Cosigner object relative to the specified member badge
     * Input parameters:
     * - badge_id: non fungible id of the member badge used to sign an operation
     * Outputs:
     * - a Cosigner object
     */
    pub fn new(
        badge_id: NonFungibleLocalId,
    ) -> Cosigner {
        Cosigner {
            badge_id: badge_id,
            transaction_hash: Runtime::transaction_hash(),
        }
    }

    /* Check if this Cosigner corresponds to a currently enabled member badge or not.
     * Input parameters:
     * - resource_manager: the resource manager of the member badges
     * Outputs:
     * - true if the badge corresponding to this signature is currently enabled, false otherwise
     */
    pub fn is_enabled(
        &self,
        resource_manager: &NonFungibleResourceManager,
    ) -> bool {
        resource_manager.get_non_fungible_data::<Member>(
            &self.badge_id,
        )
            .is_enabled()
    }
}

impl PartialEq for Cosigner {

    /* Cosigners comparation.
     * Two Cosigners are equal if produced from the same member badge, we are ignoring the
     * transaction hash here.
     * This ensures that a member can't vote more than once for the same Operation.
     * Input parameters:
     * - other: a Cosigner to compare with this one
     * Outputs:
     * - true if the signatures were generated from the same member badge
     */
    fn eq(
        &self,
        other: &Self
    ) -> bool {
        self.badge_id == other.badge_id
    }
}

impl Eq for Cosigner {
}

impl Hash for Cosigner {

    /* Cosigner hash generation.
     * Two Cosigners are equal if produced from the same member badge, we are ignoring the
     * transaction hash here.
     * This ensures that a member can't vote more than once for the same Operation.
     * Input parameters:
     * - state: the state of the hasher
     */
    fn hash<H: Hasher>(
        &self,
        state: &mut H
    ) {
        self.badge_id.hash(state);
    }
}
