use std::str::FromStr;
use std::u64;

use itertools::izip;

use crate::guid::GUID;
use crate::traits::FromRecord;

#[derive(Debug)]
pub struct PowerInfo {
    power_type: i32,
    current_power: u64,
    max_power: u64,
    power_cost: u64,

}


#[derive(Debug)]
pub struct AdvancedParams {
    info_guid: String,
    owner_guid: String,
    current_hp: u64,
    max_hp: u64,
    attack_power: u64,
    spell_power: u64,
    armor: u64,
    absorb: u64,
    power_info: Vec<PowerInfo>,
    position_x: f32,
    position_y: f32,
    ui_map_id: u64,
    facing: f32,
    level_or_ilvl: u64,
}

impl FromRecord for AdvancedParams {
    fn parse_record(line: &[&str]) -> Self {
        let power_info = izip!(
            line[8].split("|"), line[9].split("|"), line[10].split("|"), line[11].split("|")
        )
            .map(|(t, cur, max, cost)| PowerInfo {
                power_type: i32::from_str(t).unwrap(),
                current_power: u64::from_str(cur).unwrap(),
                max_power: u64::from_str(max).unwrap(),
                power_cost: u64::from_str(cost).unwrap(),
            })
            .collect();


        Self {
            info_guid: line[0].to_string(),
            owner_guid: line[1].to_string(),
            current_hp: u64::from_str(line[2]).unwrap(),
            max_hp: u64::from_str(line[3]).unwrap(),
            attack_power: u64::from_str(line[4]).unwrap(),
            spell_power: u64::from_str(line[5]).unwrap(),
            armor: u64::from_str(line[6]).unwrap(),
            absorb: u64::from_str(line[7]).unwrap(),
            power_info,
            position_x: f32::from_str(line[12]).unwrap(),
            position_y: f32::from_str(line[13]).unwrap(),
            ui_map_id: u64::from_str(line[14]).unwrap(),
            facing: f32::from_str(line[15]).unwrap(),
            level_or_ilvl: u64::from_str(line[16]).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct Actor {
    guid: GUID,
    pub name: String,
    flags: u64,
    raid_flags: u64,
}

impl Actor {
    pub fn parse_record(line: &[&str]) -> Option<Self> {
        if line[0] == "0000000000000000" {
            // Null actor
            return None;
        }

        Some(Self {
            guid: GUID::from_str(line[0]).unwrap(),
            name: line[1].to_string(),
            flags: u64::from_str_radix(line[2].trim_start_matches("0x"), 16)
                .expect("Error parsing target flags"),
            raid_flags: u64::from_str_radix(line[3].trim_start_matches("0x"), 16)
                .expect("Error parsing target raid flags"),
        })
    }
}
