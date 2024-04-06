use std::str::FromStr;

use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

use crate::traits::FromRecord;

#[derive(Debug)]
pub enum Suffix {
    Damage(Damage),
    Missed(Missed),
    Heal(Heal),
    Aura(AuraSuffixes),
    Energize(Energize),
}

impl Suffix {
    pub fn parse_record(line: &[&str], suffix_type: EventSuffix) -> Option<Self> {
        match suffix_type {
            EventSuffix::DAMAGE => {
                Some(Self::Damage(Damage::parse_record(line)))
            }
            EventSuffix::MISSED => {
                Some(Self::Missed(Missed::parse_record(line)))
            }
            EventSuffix::HEAL => {
                Some(Self::Heal(Heal::parse_record(line)))
            }
            EventSuffix::AURA_APPLIED |
            EventSuffix::AURA_APPLIED_DOSE |
            EventSuffix::AURA_REFRESH |
            EventSuffix::AURA_REMOVED |
            EventSuffix::AURA_REMOVED_DOSE => { Some(Self::Aura(AuraSuffixes::parse_record(line))) }
            EventSuffix::CAST_SUCCESS |
            EventSuffix::CAST_START |
            EventSuffix::SUMMON |
            EventSuffix::CREATE |
            EventSuffix::ABSORBED => { None }
            EventSuffix::ENERGIZE => { Some(Self::Energize(Energize::parse_record(line))) }
            EventSuffix::SPLIT => { Some(Self::Damage(Damage::parse_record(line))) }

            x => {
                todo!("Suffix parsing not implemented: {:?}", x);
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct Damage {
    amount: u64,
    overkill: u64,
    school: i32,
    resisted: u64,
    blocked: u64,
    absorbed: u64,
    critical: bool,
    glancing: bool,
    crushing: bool,
    offhand: bool,
}

impl FromRecord for Damage {
    fn parse_record(line: &[&str]) -> Self {
        // println!("{:?}", line);
        Self {
            amount: u64::from_str(line[0]).unwrap(),
            overkill: u64::from_str(line[1]).unwrap(),
            school: i32::from_str(line[2]).unwrap(),
            resisted: u64::from_str(line[3]).unwrap(),
            blocked: u64::from_str(line[4]).unwrap(),
            absorbed: u64::from_str(line[5]).unwrap(),
            critical: line[6] != "0",
            glancing: line[7] != "nil",
            crushing: line[8] != "nil",
            offhand: line[9] != "nil",
        }
    }
}


#[derive(Debug, EnumString)]
pub enum MissType {
    ABSORB,
    BLOCK,
    DEFLECT,
    DODGE,
    EVADE,
    IMMUNE,
    MISS,
    PARRY,
    REFLECT,
    RESIST,
}

#[derive(Debug)]
pub struct Missed {
    miss_type: MissType,
    offhand: bool,
    amount_missed: u64,
    unknown_quantity: u64,
    critical: bool,
}

impl FromRecord for Missed {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            miss_type: MissType::from_str(line[0]).unwrap(),
            offhand: line[1] != "nil",
            amount_missed: u64::from_str(line[2]).unwrap(),
            unknown_quantity: u64::from_str(line[3]).unwrap(),
            critical: line[4] != "nil",
        }
    }
}

#[derive(Debug)]
pub struct Heal {
    amount: u64,
    over_healing: u64,
    absorbed: u64,
    critical: bool,
}

impl FromRecord for Heal {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            amount: u64::from_str(line[0]).unwrap(),
            over_healing: u64::from_str(line[1]).unwrap(),
            absorbed: u64::from_str(line[2]).unwrap(),
            critical: line[3] != "0",
        }
    }
}

#[derive(Debug, EnumString)]
enum AuraType {
    BUFF,
    DEBUFF,
}

#[derive(Debug)]
pub struct AuraSuffixes {
    aura_type: AuraType,
    amount: Option<u64>,
}


impl FromRecord for AuraSuffixes {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            aura_type: AuraType::from_str(line[0]).unwrap(),
            amount: if line.len() > 1 {
                Some(u64::from_str(line[1]).unwrap())
            } else { None },
        }
    }
}


#[derive(Debug)]
pub struct Energize {
    amount: f32,
    over_energize: f32,
    power_type: u64,
    max_power: u64,
}

impl FromRecord for Energize {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            amount: f32::from_str(line[0]).unwrap(),
            over_energize: f32::from_str(line[1]).unwrap(),
            power_type: u64::from_str(line[2]).unwrap(),
            max_power: u64::from_str(line[3]).unwrap(),
        }
    }
}

#[derive(Debug, AsRefStr, EnumIter, PartialEq)]
pub enum EventSuffix {
    SHIELD_MISSED,
    DAMAGE,
    MISSED,
    HEAL_ABSORBED,
    HEAL,
    ABSORBED,
    ENERGIZE,
    DRAIN,
    LEECH,
    INTERRUPT,
    DISPEL_FAILED,
    DISPEL,
    STOLEN,
    EXTRA_ATTACKS,
    AURA_APPLIED,
    AURA_APPLIED_DOSE,
    AURA_REMOVED,
    AURA_REMOVED_DOSE,
    AURA_REFRESH,
    AURA_BROKEN,
    AURA_BROKEN_SPELL,
    CAST_START,
    CAST_SUCCESS,
    CAST_FAILED,
    INSTAKILL,
    DURABILITY_DAMAGE_ALL,
    DURABILITY_DAMAGE,
    CREATE,
    SUMMON,
    DISSIPATES,
    SPLIT,
    SHIELD,
}

impl EventSuffix {
    pub fn parse(s: &str) -> Option<Self> {
        for e in Self::iter() {
            if s.ends_with(e.as_ref()) {
                return Some(e);
            }
        }

        return None;
    }
}

