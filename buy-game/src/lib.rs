use scrypto::prelude::*;
pub mod rng;
pub mod structs;
pub mod actions;
pub mod combat;

blueprint! {
    pub struct AutoChess {
        // Vault for holding Xrd from sales
        collected_xrd: Vault,
        // NFT to store data between rounds
        account_nft: ResourceAddress,
        // Vault to hold with badge for system actions
        system_vault: Vault,
        game_price: Decimal,
    }
    impl AutoChess {
        pub fn new(game_price: Decimal, account_nft: ResourceAddress, system_badge: Bucket) -> ComponentAddress {
           Self {
                system_vault: Vault:with_bucket(system_badge),
                collected_xrd: Vault::new(RADIX_TOKEN),
                account_nft,
                game_price,
            }
            .instantiate()
            .globalize()
        }
        pub fn withdraw_xrd(&mut self) -> Bucket {
            let withdraw = self.collected_xrd.take_all();
            withdraw
        }
        pub fn buy_game(&mut self, mut payment: Bucket) -> (Bucket,Bucket) {
            let key_bucket: Bucket = self.system_vault.take(1);
            //Default account data between games
            let combat_data = structs::Combat {
                state: true,
                round: 1,
                shop_tier: 1,
                fighters: Vec::from([]),
                bench: Vec::from([]),
                next_round_choice: structs::Choices::None,
                gold: self.game_data.starting_gold,
                player_health: self.game_data.starting_health,
                player_wins: 0,
                next_enemy: None,
            };
            let mut new_shop_data = structs::Shop {
                fighters: Vec::from([]),
                gear: Vec::from([]),
            };
            for _x in 0..self.game_data.starting_shop_fighters {
                new_shop_data.fighters.push(Some(self.random_animal(1)));
            };
            for _y in 0..self.game_data.starting_shop_gear {
                new_shop_data.gear.push(Some(self.random_gear(1)));
            };
            let account_data = structs::Account {
                total_wins: 0,
                skins_equiped: 0,
                combat_info: combat_data,
                shop_info: new_shop_data,

            };
            let new_character = self.system_vault.authorize(|| borrow_resource_manager!(self.account_nft)
                .mint_non_fungible(&NonFungibleId::from_u64(self.game_data.account_number), account_data));
                    
            self.game_data.account_number += 1;
            self.collected_xrd.put(payment.take(self.game_data.game_price));
            self.system_vault.put(key_bucket);
            return (new_character, payment,)
        }
    }
}