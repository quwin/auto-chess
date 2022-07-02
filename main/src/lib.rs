use scrypto::prelude::*;

blueprint! {
    pub struct AutoChess {
        // Vault for holding Xrd from sales
        collected_xrd: Vault,
        // NFT to store data between rounds
        account_nft: ResourceAddress,
        developer_vault: Vault,
        system_badge: ResourceAddress,

    }
    impl AutoChess {
        pub fn new() -> (ComponentAddress, Bucket) {
            // Creates developer badge for methods. Necessary to control system_badge
            let mut developer_badge = ResourceBuilder::new_fungible()
                .metadata("name", "developer")
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1000000);

            let developer_rule: AccessRule = rule!(require(developer_badge.resource_address()));

            // Creates system badge changing NFT Data. Necessary for game expansions.
            let system_badge = ResourceBuilder::new_fungible()
                .metadata("name", "system")
                .divisibility(DIVISIBILITY_NONE)
                .mintable(developer_rule.clone(), MUTABLE(developer_rule.clone()))
                .no_initial_supply();

            let system_rule: AccessRule = rule!(require(system_badge));

            // NFT for account data
            let account_nft = ResourceBuilder::new_non_fungible()
                .metadata("name", "structs::Combat Data")
                .mintable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .burnable(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .restrict_withdraw(AccessRule::DenyAll, MUTABLE(developer_rule.clone()))
                .updateable_non_fungible_data(system_rule.clone(), MUTABLE(developer_rule.clone()))
                .no_initial_supply(); 

            // Sets values for instantiation
            let instantiate = Self {
                developer_vault: Vault::with_bucket(developer_badge.take(9999)),
                collected_xrd: Vault::new(RADIX_TOKEN),
                account_nft,
                system_badge,
            }
            .instantiate();
            // Sets access for various methods
            let access_rules = AccessRules::new()
                .method("withdraw_xrd", rule!(require(developer_badge.resource_address())))
                .method("take_developer_badge", rule!(require(developer_badge.resource_address())))
                .method("mint_system_badge", rule!(require(developer_badge.resource_address())));
            (instantiate.add_access_check(access_rules).globalize(), developer_badge)
        }
        pub fn withdraw_xrd(&mut self) -> Bucket {
            self.collected_xrd.take_all()
        }
        pub fn take_developer_badge(&mut self, amount: Decimal) -> Bucket {
            self.developer_vault.take(amount)
        }
        pub fn mint_system_badge(&mut self, amount: Decimal) -> Bucket {
            borrow_resource_manager!(self.system_badge).mint(amount)
        }
    }
}