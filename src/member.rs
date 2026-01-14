use scrypto::prelude::*;

/* A Member struct is the non fungible data of the member badges
 */
#[derive(ScryptoSbor, NonFungibleData)]
pub struct Member {

    // member badges can be enabled and disabled
    #[mutable]
    enabled: bool,

    // the date the member badge was minted
    creation_date: Instant,
}

impl Member {

    /* Create a new Member struct.
     * Outputs:
     * - a Member object
     */
    pub fn new() -> Member {
        Member {
            enabled: true,
            creation_date: Clock::current_time_rounded_to_seconds(),
        }
    }

    /* Check whether this member badge is enabled or not.
     * Outputs:
     * - true if the member badge is currently enabled
     */
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /* Enable or disable a member badge.
     * Input parameters:
     * - non_fungible_id: non fungible id of the member badge to enable/disable
     * - enabled: whether to enable it (true) or disable it (false)
     * - resource_manager: the resource manager of the member badges
     */
    pub fn enable(
        non_fungible_id: &NonFungibleLocalId,
        enabled: bool,
        resource_manager: &NonFungibleResourceManager,
    ) {
        resource_manager.update_non_fungible_data(
            &non_fungible_id,
            "enabled",
            enabled
        );
    }

    /* Count how many member badges are currently enabled.
     * Input parameters:
     * - resource_manager: the resource manager of the member badges
     * Outputs:
     * - the number of member badges that are currently enabled
     */
    pub fn count_enabled_members(
        resource_manager: &NonFungibleResourceManager,
    ) -> usize {
        let mut id: u64 = 1;
        let mut active_members: usize = 0;
        loop {
            let non_fungible_id = NonFungibleLocalId::Integer(id.into());
            if !resource_manager.non_fungible_exists(&non_fungible_id) {
                break;
            }
            if resource_manager.get_non_fungible_data::<Member>(&non_fungible_id).enabled {
                active_members += 1;
            }
            id += 1;
        }

        return active_members;
    }

    /* Check if the specified member badge is currently enabled.
     * Input parameters:
     * - non_fungible_id: the id of the member badge to check
     * - resource_manager: the resource manager of the member badges
     * Outputs:
     * - true if the member badge is enabled, false otherwise
     */
    pub fn is_badge_enabled(
        non_fungible_id: &NonFungibleLocalId,
        resource_manager: &NonFungibleResourceManager,
    ) -> bool {
        resource_manager.get_non_fungible_data::<Member>(&non_fungible_id).enabled
    }
}
