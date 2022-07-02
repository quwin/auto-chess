use scrypto::prelude::*;

#[derive(TypeId, Decode, Encode, Describe, Copy, Clone, PartialEq, Debug)]
pub struct AbilityInfo {
    pub effect: AbilityTypes,
    pub activation: Activation,
    pub summon: Option<(u128, u8)>,
    pub gold: Option<u8>,
    pub stats: Option<(Decimal, Decimal)>,
    pub exp: Option<u8>,
    pub targeting: Targeting,
    pub times: u8,
    pub repetition: u8,
}

#[derive(TypeId, Decode, Encode, Describe, Copy, Clone, PartialEq, Debug)]
pub struct FighterInfo {
    pub id: u128,
    pub tier: u8,
    pub health: Decimal,
    pub attack: Decimal,
    pub ability: AbilityInfo,
    pub gear: u128,
    pub cost: u8,
}

#[derive(TypeId, Decode, Encode, Describe, Copy, Clone, PartialEq, Debug)]
pub struct GearInfo {
    pub id: u128,
    pub tier: u8,
    pub cost: u8,
}

#[derive(TypeId, Decode, Encode, Describe, Clone, PartialEq, Debug)]
pub struct Shop {
    pub fighters: Vec<Option<FighterInfo>>,
    pub gear: Vec<Option<GearInfo>>,
}

#[derive(TypeId, Decode, Encode, Describe, Clone, PartialEq, Debug)]
pub struct Combat {
    pub state: bool,
    pub round: u8,
    pub shop_tier: u8,
    pub fighters: Vec<Option<FighterInfo>>,
    pub bench: Vec<Option<FighterInfo>>,
    pub next_round_choice: Choices,
    pub gold: u8,
    pub player_health: Decimal,
    pub player_wins: u8,
    //This is an actual monstrosity, but theres a reason
    pub next_enemy: Option<Vec<Option<FighterInfo>>>,
}

#[derive(NonFungibleData, TypeId, Decode, Encode, Describe, PartialEq, Debug)]
pub struct Account {
    #[scrypto(mutable)]
    pub total_wins: u128,
    #[scrypto(mutable)]
    pub skins_equiped: u32,
    #[scrypto(mutable)]
    pub combat_info: Combat,
    pub shop_info: Shop,
}

#[derive(TypeId, Decode, Encode, Describe, Clone, PartialEq, Debug)]
pub enum Actions {
    Sell(FighterInfo),
    Buy(FighterInfo),
    Use((GearInfo,FighterInfo)),
    SetFighters((Vec<Option<FighterInfo>>,Vec<Option<FighterInfo>>)),
    Combine((FighterInfo,FighterInfo, bool)),
}

#[derive(TypeId, Decode, Encode, Describe, Clone, PartialEq, Debug)]
pub enum Choices {
    EnemyScouter,
    MoreFighters,
    MoreGear,
    MoreGold,
    None,
}

#[derive(TypeId, Decode, Encode, Describe, Copy, Clone, PartialEq, Debug)]
pub enum AbilityTypes {
    Gold,
    Gear,
    Summon,
    Stats,
    Exp,
}

#[derive(TypeId, Decode, Encode, Describe, Clone, PartialEq, Debug)]
pub enum BattleEnd {
    Victory,
    Defeat,
    Draw,
}

#[derive(TypeId, Decode, Encode, Describe, Copy, Clone, PartialEq, Debug)]
pub enum Position {
    Last,
    First,
    MostHP,
    LeastHP,
    MostATK,
    LeastATK,
    Specific(usize),
    Random,
}

#[derive(TypeId, Decode, Encode, Describe, Copy, Clone, PartialEq, Debug)]
pub enum Targeting {
    User,
    Ally(Position),
    Enemy(Position),
}

#[derive(TypeId, Decode, Encode, Describe, PartialEq, Debug)]
pub struct GameData {
    pub tier_odds: HashMap<u8, Vec<u8>>,
    pub fighter_amounts: LazyMap<u8, u8>,
    pub fighter_data: LazyMap<(u128, u8), FighterInfo>,
    pub gear_amounts: LazyMap<u8, u8>,
    pub gear_data: LazyMap<(u128, u8), GearInfo>,
    pub matchmaking_amounts: LazyMap<u8, u128>,
    pub matchmaking_data: LazyMap<(u128, u8), Vec<Option<FighterInfo>>>,
    pub starting_gold: u8,
    pub starting_health: Decimal,
    pub starting_shop_fighters: usize,
    pub starting_shop_gear: usize,
    pub health_on_loss: Decimal,
    pub wins_for_victory: u8,
    pub rounds_for_shop_increase: u8,
    pub rounds_for_bench_increase: u8,
    pub rounds_for_fighter_increase: u8,
}

#[derive(TypeId, Decode, Encode, Describe, Copy, Clone, PartialEq, Debug)]
pub enum Activation {
    Buy,
    Sell,
    TurnEnd,
    TurnStart,
    Hurt(u8),
    Death,
    Buff(u8),
    CombatStart,
    Kill,
    Hit,
    Passive,
    Levelup,
    ShopTierUp,
    AllySummon,
    EnemySummon,
}