use scrypto::prelude::*;
pub mod rng;
pub mod structs;
pub mod actions;
pub mod combat;

blueprint! {
    pub struct AutoChessCombat {
        account_nft: ResourceAddress,
        system_vault: Vault,
        pub game_data: structs::GameData,

    }
    impl AutoChessCombat {
        pub fn new(account_nft: ResourceAddress, system_badge: Bucket) -> ComponentAddress {
            Self {
                system_vault: Vault::with_bucket(system_badge),
                account_nft,
                game_data: structs::GameData { 
                    tier_odds: HashMap::new(),
                    fighter_amounts:  LazyMap::new(),
                    fighter_data: LazyMap::new(),
                    gear_amounts: LazyMap::new(),
                    gear_data: LazyMap::new(),
                    matchmaking_data: LazyMap::new(),
                    matchmaking_amounts: LazyMap::new(),
                    starting_gold: 4,
                    starting_health: dec!("100"),
                    starting_shop_fighters: 5,
                    starting_shop_gear: 3,
                    health_on_loss: dec!("10"),
                    wins_for_victory: 10,
                    rounds_for_shop_increase: 2,
                    rounds_for_bench_increase: 3,
                    rounds_for_fighter_increase: 4,
                },
            }
            .instantiate()
            .globalize()
        }
        pub fn start_gameplay(&mut self, account_proof: Proof) {
            assert!(account_proof.resource_address() == self.account_nft);
            let account_data: structs::Account = account_proof.non_fungible().data();
            let combat_data: structs::Combat = account_data.combat_info;
            assert!(combat_data.state == false);
            let mut new_shop_data = structs::Shop {
                fighters: Vec::new(),
                gear: Vec::new(),
            };
            for _ in 0..self.game_data.starting_shop_fighters {
                new_shop_data.fighters.push(Some(self.random_animal(1)));
            };
            for _ in 0..self.game_data.starting_shop_gear {
                new_shop_data.gear.push(Some(self.random_gear(1)));
            };
            let new_combat_data = structs::Combat {
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
            let new_account_data = structs::Account {
                total_wins: account_data.total_wins,
                skins_equiped: account_data.skins_equiped,
                combat_info: new_combat_data,
                shop_info: new_shop_data,
            };
            self.system_vault.authorize(|| account_proof.non_fungible().update_data(new_account_data))
        }
        pub fn gameplay(&mut self, account_proof: Proof, mut actions: Vec<structs::Actions>, choice: structs::Choices) {
            assert!(account_proof.resource_address() == self.account_nft);
            let key_bucket: Bucket = self.system_vault.take(1);
            let mut account_data: structs::Account = account_proof.non_fungible().data();
            let mut combat_data: structs::Combat = account_data.combat_info;
            let mut shop_data: structs::Shop = account_data.shop_info;
            assert!(combat_data.state == true);
            // Validate actions user placed
            for x in 0..actions.len() {
                match &mut actions[x] {
                    structs::Actions::Sell(sold) => { 
                        combat_data = actions::sell_fighter(combat_data.clone(), *sold);
                        if sold.ability.activation == structs::Activation::Sell {
                            combat_data = self.non_combat_abilities(combat_data, x)
                        };
                    },
                    structs::Actions::Buy(bought) => { 
                        let data = actions::buy_fighter(combat_data.clone(), *bought, shop_data.clone());
                        combat_data = data.0;
                        shop_data = data.1;
                        if bought.ability.activation == structs::Activation::Buy {
                            combat_data = self.non_combat_abilities(combat_data, x)
                        };
                    },
                    structs::Actions::Use(tuple) => {
                        let data = actions::use_gear(combat_data.clone(), tuple.0, tuple.1, shop_data.clone());
                        combat_data = data.0;
                        shop_data = data.1;
                    },
                    structs::Actions::Combine(tuple) => { 
                        let data = actions::combine_fighters(combat_data.clone(), tuple.0.clone(), tuple.1.clone(), shop_data.clone(), tuple.2);
                        combat_data = data.0;
                        shop_data = data.1;
                    },
                    structs::Actions::SetFighters(vector) => { combat_data = actions::set_positions(combat_data.clone(), vector.0.clone(), vector.1.clone()) },   
                };
            };
            // End of Round Abilities
            for y in 0..combat_data.fighters.len() {
                if combat_data.fighters[y].unwrap().ability.activation == structs::Activation::TurnEnd {
                    combat_data = self.non_combat_abilities(combat_data, y)
                };
            }
            for y in 0..combat_data.bench.len() {
                if combat_data.bench[y].unwrap().ability.activation == structs::Activation::TurnEnd {
                    combat_data = self.non_combat_abilities(combat_data, y)
                };
            }
            // Enemy for combat
            let pairs = self.game_data.matchmaking_amounts.get(&combat_data.shop_tier).unwrap();
            let random_id = rng::seed(1, pairs);
            let enemy =  if combat_data.next_enemy == None { 
                self.game_data.matchmaking_data.get(&(random_id,combat_data.shop_tier)).unwrap()
            }
            else {
                combat_data.next_enemy.clone().unwrap()
            };
            let battle = combat::combat(combat_data.fighters.clone(), enemy);
            // Publish own team for into possible enemies. If you fight a specific enemy, you then replace their spot.
            self.game_data.matchmaking_data.insert((random_id,combat_data.shop_tier),combat_data.fighters.clone());
            if battle == structs::BattleEnd::Defeat {
                combat_data.player_health -= self.game_data.health_on_loss;
            };
            if battle == structs::BattleEnd::Victory {
                combat_data.player_wins += 1;
            };
            if combat_data.player_wins == self.game_data.wins_for_victory {
                combat_data.state = false;
                account_data.total_wins += 1;
                //TODO
                return
            };
            if combat_data.player_health <= dec!(0) {
                combat_data.state = false;
                self.start_gameplay(account_proof);
                self.system_vault.put(key_bucket);
                //TODO
                return
            };
            // Post-battle data setup for next transaction
            combat_data.shop_tier = match combat_data.round.clone() {
                1..=2 => 1,
                3..=4 => 2,
                5..=6 => 3,
                7..=8 => 4,
                9..=10 => 5,
                _ => 6
                
            };
            let mut new_shop_data = structs::Shop {
                fighters: Vec::from([Some(self.random_animal(combat_data.shop_tier.clone())),
                    Some(self.random_animal(combat_data.shop_tier.clone())),
                    Some(self.random_animal(combat_data.shop_tier.clone())),
                    Some(self.random_animal(combat_data.shop_tier.clone())),
                    Some(self.random_animal(combat_data.shop_tier.clone())),
                    ]),
                gear: Vec::from([
                    Some(self.random_gear(combat_data.shop_tier.clone())),
                    Some(self.random_gear(combat_data.shop_tier.clone())),
                ]),
            };
            // New vec data
            for round in 0..combat_data.round {
                if round % self.game_data.rounds_for_shop_increase == 0 { 
                    new_shop_data.fighters.push(Some(self.random_animal(combat_data.shop_tier.clone())));
                    new_shop_data.gear.push(Some(self.random_gear(combat_data.shop_tier.clone())));
                     
                };
                if round % self.game_data.rounds_for_bench_increase == 0 {
                    combat_data.bench.push(None);
                };
                if round % self.game_data.rounds_for_fighter_increase == 0 {
                    combat_data.fighters.push(None);
                };
            };
            // TurnStart Abilities
            for y in 0..combat_data.fighters.len() {
                if combat_data.fighters[y].unwrap().ability.activation == structs::Activation::TurnStart {
                    combat_data = self.non_combat_abilities(combat_data.clone(), y)
                };
            }
            for y in 0..combat_data.bench.len() {
                if combat_data.bench[y].unwrap().ability.activation == structs::Activation::TurnStart {
                    combat_data = self.non_combat_abilities(combat_data, y)
                };
            }
            // Next Round Choice
            combat_data.next_enemy = None;
            match combat_data.next_round_choice {
                structs::Choices::EnemyScouter => {
                    let pairs = self.game_data.matchmaking_amounts.get(&combat_data.shop_tier).unwrap();
                    let random_enemy_id = rng::seed(1, pairs);
                    combat_data.next_enemy = self.game_data.matchmaking_data.get(&(random_enemy_id,combat_data.shop_tier));
                },
                structs::Choices::MoreGold => combat_data.gold += combat_data.round,
                structs::Choices::MoreGear =>  { 
                    for _round in 0..combat_data.round {
                        new_shop_data.fighters.push(Some(self.random_animal(combat_data.shop_tier.clone())));
                    };
                },
                structs::Choices::MoreFighters => { 
                    for _round in 0..combat_data.round {
                        new_shop_data.gear.push(Some(self.random_gear(combat_data.shop_tier.clone())));
                    };
                },
                structs::Choices::None => {info!("Why would you do this?")},
            };
            let new_combat_data = structs::Combat {
                state: true,
                round: combat_data.round + 1,
                shop_tier: combat_data.shop_tier,
                fighters: combat_data.fighters,
                bench: combat_data.bench,
                next_round_choice: choice,
                gold: combat_data.gold,
                player_health: combat_data.player_health,
                player_wins: combat_data.player_wins,
                next_enemy: combat_data.next_enemy,
            };

            let new_account_data = structs::Account {
                total_wins: account_data.total_wins,
                skins_equiped: account_data.skins_equiped,
                combat_info: new_combat_data,
                shop_info: new_shop_data,
            };
            self.system_vault.authorize(|| account_proof.non_fungible().update_data(new_account_data));
            self.system_vault.put(key_bucket);
        }

        // Returns data of a random animal based on given shop tier
        pub fn random_animal(&self, tier: u8) -> structs::FighterInfo {
            let odds = rng::seed(1,100);
            let mut x = 0;
            loop {
                if odds >= self.game_data.tier_odds.get(&tier).unwrap()[x].into() && odds < self.game_data.tier_odds.get(&tier).unwrap()[x + 1].into() {
                    let tier_calc = self.game_data.fighter_amounts.get(&((x + 1) as u8));
                    let rng_animal = rng::seed(1, tier_calc.unwrap().into());
                    return self.game_data.fighter_data.get(&(rng_animal,tier_calc.unwrap())).unwrap()
                }
                else {
                    x += 1;
                    continue
                };
            }
        }
        pub fn random_gear(&self, tier: u8) -> structs::GearInfo {
            let odds = rng::seed(1,100);
            let mut x = 0;
            loop {
                if odds > self.game_data.tier_odds.get(&tier).unwrap()[x].into() && odds <= self.game_data.tier_odds.get(&tier).unwrap()[x + 1].into() {
                    let tier_calc = self.game_data.gear_amounts.get(&((x + 1) as u8));
                    let rng_gear = rng::seed(1, tier_calc.unwrap().into());
                    return self.game_data.gear_data.get(&(rng_gear,tier_calc.unwrap())).unwrap()
                }
                else {
                    x += 1;
                    continue
                };
            }
        }
        // Abilities
        pub fn gold_grant(&self, mut combat_data: structs::Combat, gold: u8) -> structs::Combat {
            combat_data.gold += gold;
            combat_data
        }
        
        pub fn stat_change(&self, mut fighter: structs::FighterInfo, health: Decimal, attack: Decimal) -> structs::FighterInfo {
            fighter.health = health;
            fighter.attack = attack;
            fighter
        }
        
        pub fn summon_fighter(&self, mut combat_data: structs::Combat, fighter: (u128, u8)) -> structs::Combat {
            let new_fighter = self.game_data.fighter_data.get(&fighter).unwrap();
            if let Some(unit) = combat_data.fighters.iter().position(|x| *x == None) {
                combat_data.fighters[unit] = Some(new_fighter);
                return combat_data
            }
            else {
                let usize = combat_data.bench.iter().position(|x| *x == None);
                if usize == None {
                    return combat_data
                };
                combat_data.bench[usize.unwrap()] = Some(new_fighter);
                return combat_data
            };
        }

        pub fn combat_summon_fighter(&self, team: Vec<Option<structs::FighterInfo>>, enemy_team: Vec<Option<structs::FighterInfo>>, fighter: (u128, u8), which: bool) -> Vec<Option<structs::FighterInfo>> {
            let new_fighter = self.game_data.fighter_data.get(&fighter).unwrap();
            let mut summon_team = if which == true {
                team
            }
            else {
                enemy_team
            };
            if summon_team.contains(&None) == true {
                let usize = summon_team.iter().position(|x| *x == None).unwrap();
                summon_team[usize] = Some(new_fighter);
                return summon_team
            }
            else {
                return summon_team
            };
        }
        
        pub fn exp_grant(&self, mut fighter: structs::FighterInfo, exp: u8) -> structs::FighterInfo {
            fighter.tier = exp;
            fighter
        }
        
        pub fn non_combat_abilities(&self, mut combat_data: structs::Combat, usize: usize) -> structs::Combat {
            let fighter = combat_data.fighters[usize].unwrap();
            match fighter.ability.effect {
                structs::AbilityTypes::Exp => { 
                    combat_data.fighters[usize] = Some(self.exp_grant(fighter, fighter.ability.exp.unwrap()));
                    return combat_data 
                },
                structs::AbilityTypes::Gold => { 
                    combat_data = self.gold_grant(combat_data, fighter.ability.gold.unwrap());
                    combat_data
                },
                //TODO
                structs::AbilityTypes::Gear => {
                    combat_data
                },
                structs::AbilityTypes::Summon => { 
                    combat_data = self.summon_fighter(combat_data, fighter.ability.summon.unwrap()); 
                    combat_data
                },
                structs::AbilityTypes::Stats => { 
                    combat_data.fighters[usize] = Some(self.stat_change(fighter, fighter.ability.stats.unwrap().0, fighter.ability.stats.unwrap().1)); 
                    combat_data
                },
            }
        }
        pub fn find_most_least_stat(&self, team: Vec<Option<structs::FighterInfo>>, health: bool, most: bool) -> usize {
            let mut usize = 0;
            let mut iter = 0;
            let mut find_most = dec!(0);
            let mut find_least = Decimal::MAX;
            loop {
                if team[iter] == None {
                    iter += 1;
                    continue;
                };
                let check = if health == true {
                    team[iter].unwrap().health
                }
                else {
                    team[iter].unwrap().attack
                };
                if most == true {
                    if check > find_most {
                        find_most = check;
                        usize = iter;
                    };
                }
                else {
                    if check < find_least {
                        find_least = check;
                        usize = iter;
                    };
                }
                iter += 1;
                if iter == team.len() {
                    return usize
                };
            }
        }

        pub fn combat_abilities(&self, team: Vec<Option<structs::FighterInfo>>, usize: usize, enemy_team: Vec<Option<structs::FighterInfo>>) -> (Vec<Option<structs::FighterInfo>>,Vec<Option<structs::FighterInfo>>) {
            let fighter = team[usize].unwrap();
            let mut targeted_team: Vec<Option<structs::FighterInfo>> = Vec::new();
            let mut target_usize = 0;
            let targeted = match fighter.ability.targeting {
                structs::Targeting::Enemy(x) => {
                    targeted_team = enemy_team.clone();
                    x
                },
                structs::Targeting::Ally(y) => { 
                    targeted_team = team.clone();
                    y
                },
                structs::Targeting::User => { 
                    targeted_team = team.clone();
                    structs::Position::Specific(usize)
                },
            };
            let mut target = match targeted {
                structs::Position::First => {
                    let u = targeted_team.iter().position(|x| *x != None).unwrap();
                    target_usize = u;
                    targeted_team[u].unwrap()
                },
                structs::Position::Last => {
                    let u = targeted_team.iter().rposition(|x| *x != None).unwrap();
                    target_usize = u;
                    targeted_team[u].unwrap()
                },
                structs::Position::MostHP => {
                    let u = self.find_most_least_stat(targeted_team.clone(), true, true);
                    target_usize = u;
                    targeted_team[u].unwrap()
                },
                structs::Position::LeastHP => {
                    let u = self.find_most_least_stat(targeted_team.clone(), true, false);
                    target_usize = u;
                    targeted_team[u].unwrap()
                },
                structs::Position::MostATK => {
                    let u = self.find_most_least_stat(targeted_team.clone(), false, true);
                    target_usize = u;
                    targeted_team[u].unwrap()
                },
                structs::Position::LeastATK => {
                    let u = self.find_most_least_stat(targeted_team.clone(), false, false);
                    target_usize = u;
                    targeted_team[u].unwrap()
                },
                structs::Position::Specific(z) => {
                    if targeted_team[z] == None {
                        let u = targeted_team.iter().position(|x| *x != None).unwrap();
                        target_usize = u;
                        targeted_team[u].unwrap()
                    }
                    else {
                        target_usize = z;
                        targeted_team[z].unwrap()
                    }
                },
                structs::Position::Random => {
                    let random = loop {
                        let u: usize = rng::seed(0, targeted_team.len() as u128 - 1) as usize;
                        if targeted_team[u] != None {
                            target_usize = u;
                            break targeted_team[u].unwrap()
                        };
                    };
                    random
                },
            };
            match fighter.ability.effect {
                structs::AbilityTypes::Summon => { 
                    match fighter.ability.targeting {
                        structs::Targeting::Enemy(_) => {
                            targeted_team = self.combat_summon_fighter(team.clone(), enemy_team.clone(), fighter.ability.summon.unwrap(), false);
                            return (team,targeted_team)
                        },
                        _ => { 
                            targeted_team = self.combat_summon_fighter(team.clone(), enemy_team.clone(), fighter.ability.summon.unwrap(), true);
                            return (targeted_team,enemy_team)
                        },
                    };
                },
                structs::AbilityTypes::Stats => { 
                    target = self.stat_change(target, fighter.health, fighter.attack);
                    targeted_team[target_usize] = Some(target);
                    match fighter.ability.targeting {
                        structs::Targeting::Enemy(_) => {
                            return (team,targeted_team)
                        },
                        _ => { 
                            return (targeted_team,enemy_team)
                        },
                    };
                },
                _ => { return (team,enemy_team) },
            }
        }
    }
}