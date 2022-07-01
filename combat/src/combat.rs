use scrypto::prelude::*;
use super::structs;

pub fn check_combat_end(attacker: Vec<Option<structs::FighterInfo>>, defender: Vec<Option<structs::FighterInfo>>) -> (bool,bool,usize) {
    for x in 0..attacker.len() {
        // If not all attackers are dead
        if attacker[x] != None {
            return (false,false,x)
        };
        // If all attackers are dead
        if x == (attacker.len() - 1) && attacker[x] == None { 
            for y in 0..defender.len() {
                // If there still is another defender
                if defender[y] != None {
                    return (true,true,0)
                };
                // If that was last defender
                if y == (defender.len() - 1) && defender[y] == None{ 
                    return (true,false,0)
                }
                // If that wasn't
                else { 
                    continue;
                };
            }
        }
        // Continues if attacker is dead but not all attackers are checked
        else { 
            continue;
        };
    };
    info!("This shouldn't be reachable");
    (true,true,0)
}
// TODO: Abilities
pub fn combat(mut attacker: Vec<Option<structs::FighterInfo>>, mut defender: Vec<Option<structs::FighterInfo>>) -> structs::BattleEnd {
    loop {
        let attacker_usize = check_combat_end(attacker.clone(), defender.clone());
        if attacker_usize.0 == true {
            match attacker_usize.1 {
                true => return structs::BattleEnd::Defeat,
                false => return structs::BattleEnd::Draw,
            };
        };
        let defender_usize = check_combat_end(defender.clone(), attacker.clone());
        if defender_usize.0 == true {
            match defender_usize.1 {
                true => return structs::BattleEnd::Victory,
                false => return structs::BattleEnd::Draw,
            };
        };
        let mut attacker_fighter = attacker[attacker_usize.2].unwrap();
        let mut defender_fighter = defender[defender_usize.2].unwrap();
        attacker_fighter.health -= defender_fighter.attack;
        defender_fighter.health -= attacker_fighter.attack;
        attacker[attacker_usize.2] = Some(attacker_fighter);
        defender[defender_usize.2] = Some(defender_fighter);
        if attacker_fighter.health <= dec!(0) {
            attacker[attacker_usize.2] = None;
        };
        if defender_fighter.health <= dec!(0) {
            attacker[defender_usize.2] = None;
        };
    };           
}