use std::u8;

use scrypto::prelude::*;
use super::structs;

pub fn sell_fighter(mut combat_data: structs::Combat, fighter: structs::FighterInfo) -> structs::Combat {
    if combat_data.fighters.contains(&(Some(fighter))) == true {
        let usize = combat_data.fighters.iter().position(|x| *x == Some(fighter)).expect("Not found!");
        combat_data.fighters[usize] = None;
    }
    else {
        let bench_usize = combat_data.bench.iter().position(|x| *x == Some(fighter)).expect("Not found!");
        combat_data.bench[bench_usize] = None;
    };
    let gold = match fighter.tier {
        1..=2 => 1,
        3..=8 => 2,
        9 => 3,
        _ => 0,
    };
    combat_data.gold += gold;
    combat_data
}
pub fn buy_fighter(mut combat_data: structs::Combat, fighter: structs::FighterInfo, mut shop_data: structs::Shop) -> (structs::Combat,structs::Shop) {
    let shop_usize = shop_data.fighters.iter().position(|x| *x == Some(fighter)).expect("Not found!");
    shop_data.fighters[shop_usize] = None;
    // Makes sure that there is room in bench or active slots
    if combat_data.fighters.contains(&None) == true {
        let usize = combat_data.fighters.iter().position(|x| *x == None).expect("Not found!");
        combat_data.fighters[usize] = Some(fighter);
    }
    else {
        let usize = combat_data.bench.iter().position(|x| *x == None).expect("Not found!");
        combat_data.bench[usize] = Some(fighter);
    };
    combat_data.gold -= fighter.cost;
    (combat_data,shop_data)
}
//TODO, make it so gear does specific stuff
pub fn use_gear(mut combat_data: structs::Combat, gear: structs::GearInfo, fighter: structs::FighterInfo, mut shop_data: structs::Shop) -> (structs::Combat,structs::Shop) {
    assert!(combat_data.fighters.contains(&Some(fighter)) == true || combat_data.bench.contains(&Some(fighter)) == true);
    let mut new_fighter = fighter.clone();
    new_fighter.health += dec!(1);
    new_fighter.attack += dec!(1);
    let usize = combat_data.fighters.iter().position(|x| *x == Some(fighter)).expect("Not found!");
    if combat_data.fighters.contains(&Some(fighter)) == true {
        combat_data.fighters[usize] = Some(new_fighter);
    }
    else {
        combat_data.bench[usize] = Some(new_fighter);
    };
    shop_data.gear.swap_remove(shop_data.gear.iter().position(|x| *x == Some(gear)).expect("Not found!"));
    shop_data.gear.push(None);
    combat_data.gold -= gear.cost;
    (combat_data, shop_data)
}
pub fn combine_fighters(mut combat_data: structs::Combat, big_fighter: structs::FighterInfo, smol_fighter: structs::FighterInfo, mut shop_data: structs::Shop, buy: bool) -> (structs::Combat, structs::Shop) {
    assert!(big_fighter.tier < 9);
    assert!(big_fighter.id == smol_fighter.id);
    let upgrades: u8 = match smol_fighter.tier {
        0..=2 => 1,
        3..=u8::MAX => 2,
    };
    let mut new_fighter = big_fighter.clone();
    new_fighter.tier += smol_fighter.tier;
    if new_fighter.tier > 9 { new_fighter.tier = 9};
    new_fighter.health += upgrades;
    new_fighter.attack += upgrades;
    // Updates big fighter
    if combat_data.fighters.contains(&Some(big_fighter)) == true {
        let usize = combat_data.fighters.iter().position(|x| *x == Some(big_fighter)).expect("Not found!");
        combat_data.fighters[usize] = Some(new_fighter);
    }
    else {
        let usize = combat_data.bench.iter().position(|x| *x == Some(big_fighter)).expect("Not found!");
        combat_data.bench[usize] = Some(new_fighter);
    };
    // Removes the fodder for combining
    if buy == false {
        if combat_data.fighters.contains(&Some(smol_fighter)) == true {
            let usize = combat_data.fighters.iter().position(|x| *x == Some(smol_fighter)).expect("Not found!");
            combat_data.fighters[usize] = None;
        }
        else {
            let usize = combat_data.bench.iter().position(|x| *x == Some(smol_fighter)).expect("Not found!");
            combat_data.bench[usize] = None;
        };
    }
    else {
        let usize = shop_data.fighters.iter().position(|x| *x == Some(smol_fighter)).expect("Not found!");
        shop_data.fighters[usize] = None;
        combat_data.gold -= smol_fighter.cost
    };
    (combat_data,shop_data)
}
pub fn set_positions(mut combat_data: structs::Combat, fighters: Vec<Option<structs::FighterInfo>>, bench: Vec<Option<structs::FighterInfo>>) -> structs::Combat {
    let mut new_fighter_vec: Vec<Option<structs::FighterInfo>> = Vec::new();
    let mut new_bench_vec: Vec<Option<structs::FighterInfo>> = Vec::new();
    // Necessary due to altering the length of the Vecs later
    let bench_len = combat_data.bench.len();
    let fighters_len = combat_data.fighters.len();
    // Checks if fighter from proposed Vec is from active fighters or bench, removes them from active fighters/bench
    // Design process to avoid duplicating fighters
    for y in 0..fighters_len {
        if combat_data.fighters.contains(&fighters[y]) == true {
            combat_data.fighters.swap_remove(combat_data.fighters.iter().position(|x| *x == fighters[y]).expect("Not found: fighter from fighter"));
            new_fighter_vec.push(fighters[y]);
        }
        else {
            combat_data.bench.swap_remove(combat_data.bench.iter().position(|x| *x == fighters[y]).expect("Not found: fighter from bench"));
            new_fighter_vec.push(fighters[y]);
        };
    };
    for z in 0..bench_len {
        if combat_data.fighters.contains(&bench[z]) == true {
            combat_data.fighters.swap_remove(combat_data.fighters.iter().position(|x| *x == bench[z]).expect("Not found: bench from fighter"));
            new_bench_vec.push(bench[z]);
        }
        else {
            combat_data.bench.swap_remove(combat_data.bench.iter().position(|x| *x == bench[z]).expect("Not found; bench from bench"));
            new_bench_vec.push(bench[z]);
        };     
    };
    combat_data.fighters = new_fighter_vec;
    combat_data.bench = new_bench_vec;
    assert!(combat_data.fighters.is_empty() == false);
    assert!(combat_data.bench.is_empty() == false);
    combat_data
}
// Bunch of tests, too bad I can't make test_fighter_x be a const so I don't have to set the variables 20 times
#[cfg(test)]
mod tests {
    use crate::structs;
    use crate::actions;
    use scrypto::prelude::*;

    #[test]
    fn test_sell() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 2,
            tier: 4,
            health: dec!(3),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 2,
        };
        let test_fighter_3 = structs::FighterInfo {
            id: 3,
            tier: 9,
            health: dec!(3),
            attack: dec!(3),
            ability: test_ability,
            gear: 0,
            cost: 4,
        };
        let mut test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([None,Some(test_fighter_1),Some(test_fighter_2),None,Some(test_fighter_3)]),
            bench: Vec::from([None,Some(test_fighter_3)]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        // Checks selling fighter 1
        let sale1 = actions::sell_fighter(test_combat_data.clone(), test_fighter_1);
        test_combat_data.fighters = Vec::from([None,None,Some(test_fighter_2),None,Some(test_fighter_3)]);
        assert_eq!(test_combat_data.fighters, sale1.fighters);
        test_combat_data.gold += 1;
        assert_eq!(test_combat_data, sale1);
        // Checks selling fighter 2
        let sale2 = actions::sell_fighter(sale1.clone(), test_fighter_2);
        test_combat_data.fighters = Vec::from([None,None,None,None,Some(test_fighter_3)]);
        assert_eq!(test_combat_data.fighters, sale2.fighters);
        test_combat_data.gold += 2;
        assert_eq!(test_combat_data, sale2);
        // Checks selling fighter 3
        let sale3 = actions::sell_fighter(sale2.clone(), test_fighter_3);
        test_combat_data.fighters = Vec::from([None,None,None,None,None]);
        assert_eq!(test_combat_data.fighters, sale3.fighters);
        test_combat_data.gold += 3;
        assert_eq!(test_combat_data, sale3);
        // Checks selling fighter 3 in bench
        let sale4 = actions::sell_fighter(sale3.clone(), test_fighter_3);
        test_combat_data.bench = Vec::from([None,None]);
        assert_eq!(test_combat_data.bench, sale4.bench);
        test_combat_data.gold += 3;
        assert_eq!(test_combat_data, sale4);
    }
    #[test]
    #[should_panic]
    fn panic_sell() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 2,
            tier: 4,
            health: dec!(3),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 2,
        };
        let test_fighter_3 = structs::FighterInfo {
            id: 3,
            tier: 3,
            health: dec!(3),
            attack: dec!(3),
            ability: test_ability,
            gear: 0,
            cost: 4,
        };
        let mut test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([None,Some(test_fighter_1),Some(test_fighter_2),None,Some(test_fighter_3)]),
            bench: Vec::from([None,Some(test_fighter_3)]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        // Checks you can't sell something you don't have
        let mut foo_fighter = test_fighter_1.clone();
        foo_fighter.tier = 9;
        assert!(foo_fighter != test_fighter_1);
        let sale1 = actions::sell_fighter(test_combat_data.clone(), foo_fighter);
        test_combat_data.fighters = Vec::from([None,None,Some(test_fighter_2),None,Some(test_fighter_3)]);
        assert_eq!(test_combat_data.fighters, sale1.fighters);
    }
    #[test]
    fn test_buy() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 2,
            tier: 4,
            health: dec!(3),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 2,
        };
        let test_fighter_3 = structs::FighterInfo {
            id: 3,
            tier: 3,
            health: dec!(3),
            attack: dec!(3),
            ability: test_ability,
            gear: 0,
            cost: 3,
        };
        let mut test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([None,None,None,Some(test_fighter_2),Some(test_fighter_1)]),
            bench: Vec::from([None,Some(test_fighter_3)]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        let mut test_shop_data = structs::Shop {
            fighters: Vec::from([Some(test_fighter_1),Some(test_fighter_2),None,Some(test_fighter_3),Some(test_fighter_1)]),
            gear: Vec::new(),
        };
        let buy1 = actions::buy_fighter(test_combat_data.clone(), test_fighter_1, test_shop_data.clone());
        test_combat_data.fighters = Vec::from([Some(test_fighter_1),None,None,Some(test_fighter_2),Some(test_fighter_1)]);
        test_shop_data.fighters = Vec::from([None,Some(test_fighter_2),None,Some(test_fighter_3),Some(test_fighter_1)]);
        assert_eq!(test_combat_data.fighters, buy1.0.fighters);
        assert_eq!(test_shop_data.fighters, buy1.1.fighters);
        test_combat_data.gold -= test_fighter_1.cost;
        assert_eq!(test_combat_data, buy1.0);

        let buy2 = actions::buy_fighter(buy1.0.clone(), test_fighter_2, buy1.1.clone());
        test_combat_data.fighters = Vec::from([Some(test_fighter_1),Some(test_fighter_2),None,Some(test_fighter_2),Some(test_fighter_1)]);
        test_shop_data.fighters = Vec::from([None,None,None,Some(test_fighter_3),Some(test_fighter_1)]);
        assert_eq!(test_combat_data.fighters, buy2.0.fighters);
        assert_eq!(test_shop_data.fighters, buy2.1.fighters);
        test_combat_data.gold -= test_fighter_2.cost;
        assert_eq!(test_combat_data, buy2.0);

        let buy3 = actions::buy_fighter(buy2.0.clone(), test_fighter_3, buy2.1.clone());
        test_combat_data.fighters = Vec::from([Some(test_fighter_1),Some(test_fighter_2),Some(test_fighter_3),Some(test_fighter_2),Some(test_fighter_1)]);
        test_shop_data.fighters = Vec::from([None,None,None,None,Some(test_fighter_1)]);
        assert_eq!(test_combat_data.fighters, buy3.0.fighters);
        assert_eq!(test_shop_data.fighters, buy3.1.fighters);
        test_combat_data.gold -= test_fighter_3.cost;
        assert_eq!(test_combat_data, buy3.0);

        let buy4 = actions::buy_fighter(buy3.0.clone(), test_fighter_1, buy3.1.clone());
        test_combat_data.bench = Vec::from([Some(test_fighter_1),Some(test_fighter_3)]);
        test_shop_data.fighters = Vec::from([None,None,None,None,None]);
        assert_eq!(test_combat_data.fighters, buy4.0.fighters);
        assert_eq!(test_combat_data.bench, buy4.0.bench);
        assert_eq!(test_shop_data.fighters, buy4.1.fighters);
        test_combat_data.gold -= test_fighter_1.cost;
        assert_eq!(test_combat_data, buy4.0);
    }
    #[test]
    #[should_panic]
    fn panic_buy() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 2,
            tier: 4,
            health: dec!(3),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 2,
        };
        let test_fighter_3 = structs::FighterInfo {
            id: 3,
            tier: 3,
            health: dec!(3),
            attack: dec!(3),
            ability: test_ability,
            gear: 0,
            cost: 3,
        };
        let mut test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([None,None,None,Some(test_fighter_2),Some(test_fighter_1)]),
            bench: Vec::from([None,Some(test_fighter_3)]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        let mut test_shop_data = structs::Shop {
            fighters: Vec::from([Some(test_fighter_1),Some(test_fighter_2),None,Some(test_fighter_3),Some(test_fighter_1)]),
            gear: Vec::new(),
        };
        let buy1 = actions::buy_fighter(test_combat_data.clone(), test_fighter_3, test_shop_data.clone());
        test_combat_data.fighters = Vec::from([Some(test_fighter_3),None,None,Some(test_fighter_2),Some(test_fighter_1)]);
        test_shop_data.fighters = Vec::from([Some(test_fighter_1),Some(test_fighter_2),None,None,Some(test_fighter_1)]);
        assert_eq!(test_combat_data.fighters, buy1.0.fighters);
        assert_eq!(test_shop_data.fighters, buy1.1.fighters); 
        test_combat_data.gold -= test_fighter_3.cost;
        assert_eq!(test_combat_data, buy1.0);

        actions::buy_fighter(buy1.0, test_fighter_3, buy1.1);
    }
    #[test]
    fn test_use() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 1,
            tier: 2,
            health: dec!(3),
            attack: dec!(3),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_gear_1 = structs::GearInfo {
            id: 1,
            cost: 1,
            tier: 1,
        };
        let test_gear_2 = structs::GearInfo {
            id: 22,
            cost: 4,
            tier: 2,
        };
        let mut test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([None,None,None,Some(test_fighter_2),Some(test_fighter_1)]),
            bench: Vec::from([None,None]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        let mut test_shop_data = structs::Shop {
            fighters: Vec::from([Some(test_fighter_1),Some(test_fighter_2),None,None,Some(test_fighter_1)]),
            gear: Vec::from([Some(test_gear_1),Some(test_gear_2),None]),
        };
        let use1 = actions::use_gear(test_combat_data.clone(), test_gear_1, test_fighter_1, test_shop_data.clone());
        test_shop_data.gear = Vec::from([None,Some(test_gear_2),None]);
        assert_eq!(test_shop_data.gear, use1.1.gear);
        let mut new_fighter = test_fighter_1.clone();
        // TODO
        new_fighter.health += 1;
        new_fighter.attack += 1;
        test_combat_data.fighters[4] = Some(new_fighter);
        test_combat_data.gold -= test_gear_1.cost;
        assert_eq!(test_combat_data, use1.0);
    }
    #[test]
    #[should_panic]
    fn panic_use() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 2,
            tier: 4,
            health: dec!(3),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 2,
        };
        let test_gear_1 = structs::GearInfo {
            id: 1,
            cost: 1,
            tier: 1,
        };
        let test_gear_2 = structs::GearInfo {
            id: 22,
            cost: 4,
            tier: 2,
        };
        let mut test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([None,None,None,Some(test_fighter_2),Some(test_fighter_1)]),
            bench: Vec::from([None,None]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        let mut test_shop_data = structs::Shop {
            fighters: Vec::from([Some(test_fighter_1),Some(test_fighter_2),None,None,Some(test_fighter_1)]),
            gear: Vec::from([Some(test_gear_1),None,None]),
        };
        let use1 = actions::use_gear(test_combat_data.clone(), test_gear_2, test_fighter_1, test_shop_data.clone());
        test_shop_data.gear = Vec::from([None,Some(test_gear_2),None]);
        assert_eq!(test_shop_data.gear, use1.1.gear);
        let mut new_fighter = test_fighter_1.clone();
        // TODO
        new_fighter.health += 1;
        new_fighter.attack += 1;
        test_combat_data.fighters[4] = Some(new_fighter);
        test_combat_data.gold -= test_gear_2.cost;
        assert_eq!(test_combat_data, use1.0);
    }
    #[test]
    fn test_combine() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 1,
            tier: 2,
            health: dec!(3),
            attack: dec!(3),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_3 = structs::FighterInfo {
            id: 1,
            tier: 3,
            health: dec!(4),
            attack: dec!(4),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_4 = structs::FighterInfo {
            id: 1,
            tier: 5,
            health: dec!(5),
            attack: dec!(5),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_5 = structs::FighterInfo {
            id: 1,
            tier: 8,
            health: dec!(7),
            attack: dec!(7),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_6 = structs::FighterInfo {
            id: 1,
            tier: 9,
            health: dec!(8),
            attack: dec!(8),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let mut test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([None,None,None,Some(test_fighter_2),Some(test_fighter_1)]),
            bench: Vec::from([None,Some(test_fighter_1)]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        let mut test_shop_data = structs::Shop {
            fighters: Vec::from([Some(test_fighter_3),Some(test_fighter_2),None,None,Some(test_fighter_1)]),
            gear: Vec::from([]),
        };
        let combine1: (structs::Combat, structs::Shop) = actions::combine_fighters(test_combat_data.clone(), test_fighter_1, test_fighter_1, test_shop_data.clone(), false);
        test_combat_data.fighters = Vec::from([None,None,None,Some(test_fighter_2),Some(test_fighter_2)]);
        test_combat_data.bench = Vec::from([None,None]);
        assert_eq!(test_shop_data, combine1.1);
        assert_eq!(test_combat_data, combine1.0);

        let combine2: (structs::Combat, structs::Shop) = actions::combine_fighters(combine1.0.clone(), test_fighter_2, test_fighter_1, combine1.1.clone(), true);
        test_combat_data.fighters = Vec::from([None,None,None,Some(test_fighter_3),Some(test_fighter_2)]);
        test_shop_data.fighters = Vec::from([Some(test_fighter_3),Some(test_fighter_2),None,None,None]);
        test_combat_data.gold -= test_fighter_1.cost;
        assert_eq!(test_shop_data, combine2.1);
        assert_eq!(test_combat_data, combine2.0);

        let combine3: (structs::Combat, structs::Shop) = actions::combine_fighters(combine2.0.clone(), test_fighter_3, test_fighter_2, combine2.1.clone(), true);
        test_combat_data.fighters = Vec::from([None,None,None,Some(test_fighter_4),Some(test_fighter_2)]);
        test_shop_data.fighters = Vec::from([Some(test_fighter_3),None,None,None,None]);
        test_combat_data.gold -= test_fighter_2.cost;
        assert_eq!(test_shop_data, combine3.1);
        assert_eq!(test_combat_data, combine3.0);

        let combine4: (structs::Combat, structs::Shop) = actions::combine_fighters(combine3.0.clone(), test_fighter_4, test_fighter_3, combine3.1.clone(), true);
        test_combat_data.fighters = Vec::from([None,None,None,Some(test_fighter_5),Some(test_fighter_2)]);
        test_shop_data.fighters = Vec::from([None,None,None,None,None]);
        test_combat_data.gold -= test_fighter_3.cost;
        assert_eq!(test_shop_data, combine4.1);
        assert_eq!(test_combat_data, combine4.0);

        let combine5: (structs::Combat, structs::Shop) = actions::combine_fighters(combine4.0.clone(), test_fighter_5, test_fighter_2, combine4.1.clone(), false);
        test_combat_data.fighters = Vec::from([None,None,None,Some(test_fighter_6),None]);
        assert_eq!(test_combat_data, combine5.0);
    }
    #[test]
    #[should_panic]
    fn panic_combine() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 1,
            tier: 2,
            health: dec!(3),
            attack: dec!(3),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_3 = structs::FighterInfo {
            id: 11,
            tier: 3,
            health: dec!(4),
            attack: dec!(4),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([None,None,None,Some(test_fighter_2),Some(test_fighter_1)]),
            bench: Vec::from([None,Some(test_fighter_1)]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        let test_shop_data = structs::Shop {
            fighters: Vec::from([Some(test_fighter_3),Some(test_fighter_2),None,None,Some(test_fighter_1)]),
            gear: Vec::from([]),
        };
        actions::combine_fighters(test_combat_data.clone(), test_fighter_1, test_fighter_3, test_shop_data.clone(), true);
    }
    #[test]
    fn test_set_position() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 1,
            tier: 2,
            health: dec!(3),
            attack: dec!(3),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_3 = structs::FighterInfo {
            id: 1,
            tier: 3,
            health: dec!(4),
            attack: dec!(4),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_4 = structs::FighterInfo {
            id: 1,
            tier: 5,
            health: dec!(5),
            attack: dec!(5),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_5 = structs::FighterInfo {
            id: 1,
            tier: 8,
            health: dec!(7),
            attack: dec!(7),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_6 = structs::FighterInfo {
            id: 1,
            tier: 9,
            health: dec!(8),
            attack: dec!(8),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let mut test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([Some(test_fighter_1),Some(test_fighter_2),Some(test_fighter_3),Some(test_fighter_4),Some(test_fighter_5)]),
            bench: Vec::from([None,Some(test_fighter_6)]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        let new_fighter_position = [Some(test_fighter_6),Some(test_fighter_3),Some(test_fighter_4),Some(test_fighter_5),None].to_vec();
        let new_bench_position = [Some(test_fighter_2),Some(test_fighter_1)].to_vec();
        let test_position: structs::Combat = actions::set_positions(test_combat_data.clone(),new_fighter_position.clone(), new_bench_position.clone());
        test_combat_data.fighters = new_fighter_position;
        test_combat_data.bench = new_bench_position;
        assert_eq!(test_combat_data, test_position);
    }
    #[test]
    #[should_panic]
    fn panic_position() {
        let test_ability = structs::AbilityInfo {
            effect: structs::AbilityTypes::Gold,
            activation: structs::Activation::Passive,
            summon: None,
            gold: None,
            stats: None,
            exp: None,
            targeting: structs::Targeting::User,
            times: 1,
            repetition: 0,
        };
        let test_fighter_1 = structs::FighterInfo {
            id: 1,
            tier: 1,
            health: dec!(2),
            attack: dec!(2),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_2 = structs::FighterInfo {
            id: 1,
            tier: 2,
            health: dec!(3),
            attack: dec!(3),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_fighter_3 = structs::FighterInfo {
            id: 11,
            tier: 3,
            health: dec!(4),
            attack: dec!(4),
            ability: test_ability,
            gear: 0,
            cost: 1,
        };
        let test_combat_data = structs::Combat {
            state: true,
            round: 1,
            shop_tier: 1,
            fighters: Vec::from([None,None,None,Some(test_fighter_2),Some(test_fighter_1)]),
            bench: Vec::from([None,Some(test_fighter_1)]),
            next_round_choice: structs::Choices::None,
            gold: 10,
            player_health: dec!(100),
            player_wins: 0,
            next_enemy: None,
        };
        let test_shop_data = structs::Shop {
            fighters: Vec::from([Some(test_fighter_3),Some(test_fighter_2),None,None,Some(test_fighter_1)]),
            gear: Vec::from([]),
        };
        actions::combine_fighters(test_combat_data.clone(), test_fighter_1, test_fighter_3, test_shop_data.clone(), true);
    }
}