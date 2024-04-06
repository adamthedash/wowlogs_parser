use std::str::FromStr;

use crate::traits::FromRecord;

#[derive(Debug)]
pub struct CombatLogVersion {
    version: u8,
    advanced_logging_enabled: bool,
    build_version: String,
    project_id: u8,
}

impl FromRecord for CombatLogVersion {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            version: u8::from_str(line[0]).unwrap(),
            advanced_logging_enabled: u8::from_str(line[2]).unwrap() == 1,
            build_version: line[4].to_string(),
            project_id: u8::from_str(line[6]).unwrap(),
        }
    }
}


#[derive(Debug)]
pub struct MapChange {
    ui_map_id: u64,
    ui_map_name: String,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl FromRecord for MapChange {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            ui_map_id: u64::from_str(line[0]).unwrap(),
            ui_map_name: line[1].to_string(),
            x0: f32::from_str(line[2]).unwrap(),
            x1: f32::from_str(line[3]).unwrap(),
            y0: f32::from_str(line[4]).unwrap(),
            y1: f32::from_str(line[5]).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct ZoneChange {
    instance_id: u64,
    zone_name: String,
    difficulty_id: u8,
}

impl FromRecord for ZoneChange {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            instance_id: u64::from_str(line[0]).unwrap(),
            zone_name: line[1].to_string(),
            difficulty_id: u8::from_str(line[2]).unwrap(),
        }
    }
}


#[derive(Debug)]
pub struct WorldMarkerPlaced {
    instance_id: u64,
    marker: u8,
    x: f32,
    y: f32,
}

impl FromRecord for WorldMarkerPlaced {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            instance_id: u64::from_str(line[0]).unwrap(),
            marker: u8::from_str(line[1]).unwrap(),
            x: f32::from_str(line[2]).unwrap(),
            y: f32::from_str(line[3]).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct EncounterStart {
    encounter_id: u64,
    encounter_name: String,
    difficulty_id: u8,
    group_size: u8,
    instance_id: u64,
}

impl FromRecord for EncounterStart {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            encounter_id: u64::from_str(line[0]).unwrap(),
            encounter_name: line[1].to_string(),
            difficulty_id: u8::from_str(line[2]).unwrap(),
            group_size: u8::from_str(line[3]).unwrap(),
            instance_id: u64::from_str(line[4]).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct EncounterEnd {
    encounter_id: u64,
    encounter_name: String,
    difficulty_id: u8,
    group_size: u8,
    success: bool,
    fight_time: u64,
}

impl FromRecord for EncounterEnd {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            encounter_id: u64::from_str(line[0]).unwrap(),
            encounter_name: line[1].to_string(),
            difficulty_id: u8::from_str(line[2]).unwrap(),
            group_size: u8::from_str(line[3]).unwrap(),
            success: line[4] == "1",
            fight_time: u64::from_str(line[5]).unwrap(),
        }
    }
}