use scrypto::prelude::*;
use super::structs;

pub fn check_combat_end(team_1: Vec<Option<structs::FighterInfo>>, team_2: Vec<Option<structs::FighterInfo>>) -> (bool,bool,usize) {
    for x in 0..team_1.len() {
        // If not all team_1s are dead
        if team_1[x] != None {
            return (false,false,x)
        };
        // If all team_1s are dead
        if x == (team_1.len() - 1) && team_1[x] == None { 
            for y in 0..team_2.len() {
                // If there still is another team_2
                if team_2[y] != None {
                    return (true,true,0)
                };
                // If that was last team_2
                if y == (team_2.len() - 1) { 
                    return (true,false,0)
                };
            }
        };
    };
    panic!("How did we end up here?")
}
// TODO: Abilities
pub fn combat(mut attacker: Vec<Option<structs::FighterInfo>>, mut defender: Vec<Option<structs::FighterInfo>>) -> structs::BattleEnd {
    loop {
        let attacker_tuple = check_combat_end(attacker.clone(), defender.clone());
        if attacker_tuple.0 {
            match attacker_tuple.1 {
                true => return structs::BattleEnd::Defeat,
                false => return structs::BattleEnd::Draw,
            };
        };
        let defender_tuple = check_combat_end(defender.clone(), attacker.clone());
        if defender_tuple.0 {
            match defender_tuple.1 {
                true => return structs::BattleEnd::Victory,
                false => return structs::BattleEnd::Draw,
            };
        };
        let mut attacker_fighter = attacker[attacker_tuple.2].unwrap();
        let mut defender_fighter = defender[defender_tuple.2].unwrap();
        attacker_fighter.health -= defender_fighter.attack;
        defender_fighter.health -= attacker_fighter.attack;
        attacker[attacker_tuple.2] = Some(attacker_fighter);
        defender[defender_tuple.2] = Some(defender_fighter);
        if attacker_fighter.health <= dec!(0) {
            attacker[attacker_tuple.2] = None;
        };
        if defender_fighter.health <= dec!(0) {
            attacker[defender_tuple.2] = None;
        };
    };           
}