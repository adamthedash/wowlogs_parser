use std::str::FromStr;

use itertools::izip;

use crate::enums::PowerType;
use crate::guid::GUID;
use crate::utils::parse_num;

#[derive(Debug)]
struct PowerInfo {
    power_type: Option<PowerType>,
    current_power: u64,
    max_power: u64,
    power_cost: u64,
}

impl PowerInfo {
    fn parse(line: &[&str]) -> Vec<Self> {
        assert_eq!(line.len(), 4);

        izip!(
            line[0].split("|"),
            line[1].split("|"),
            line[2].split("|"),
            line[3].split("|")
        )
            .map(|(t, cur, max, cost)| PowerInfo {
                power_type: PowerType::parse(t),
                current_power: parse_num(cur),
                max_power: parse_num(max),
                power_cost: parse_num(cost),
            })
            .collect()
    }
}

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
    facing: f32,
}

impl Position {
    fn parse(line_xy: &[&str], line_facing: &str) -> Self {
        assert_eq!(line_xy.len(), 2);

        Self {
            x: parse_num(line_xy[0]),
            y: parse_num(line_xy[1]),
            facing: parse_num(line_facing),
        }
    }
}

#[derive(Debug)]
pub struct AdvancedParams {
    info_guid: Option<GUID>,
    owner_guid: Option<GUID>,
    current_hp: u64,
    max_hp: u64,
    attack_power: u64,
    spell_power: u64,
    armor: u64,
    absorb: u64,
    power_info: Vec<PowerInfo>,
    position: Position,
    ui_map_id: u64,
    level_or_ilvl: u64,
}

impl AdvancedParams {
    pub(crate) fn parse(line: &[&str]) -> Self {
        assert_eq!(line.len(), 17);

        Self {
            info_guid: GUID::parse(line[0]),
            owner_guid: GUID::parse(line[1]),
            current_hp: parse_num(line[2]),
            max_hp: parse_num(line[3]),
            attack_power: parse_num(line[4]),
            spell_power: parse_num(line[5]),
            armor: parse_num(line[6]),
            absorb: parse_num(line[7]),
            power_info: PowerInfo::parse(&line[8..12]),
            position: Position::parse(&line[12..14], line[15]),
            ui_map_id: parse_num(line[14]),
            level_or_ilvl: parse_num(line[16]),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::advanced::{AdvancedParams, Position, PowerInfo};

    #[test]
    fn parse_power_info() {
        let line = vec!["1", "0", "0", "0"];
        let parsed = PowerInfo::parse(&line);
        println!("{:?}", parsed);

        let line = vec!["3", "160", "160", "0"];
        let parsed = PowerInfo::parse(&line);
        println!("{:?}", parsed);

        let line = vec!["3|4", "43|6", "300|6", "25|6"];
        let parsed = PowerInfo::parse(&line);
        println!("{:?}", parsed);
    }

    #[test]
    fn parse_position() {
        let parsed = Position::parse(&vec!["3295.44", "13209.11"], "3.4506");
        println!("{:?}", parsed);
    }

    #[test]
    fn parse() {
        let line = vec!["Creature-0-1469-2549-12530-210177-000011428F", "0000000000000000", "5927873", "7468728", "0", "0", "5043", "0", "1", "0", "0", "0", "3295.44", "13209.11", "2232", "3.4506", "72"];
        let parsed = AdvancedParams::parse(&line);
        println!("{:?}", parsed);

        let line = vec!["Player-1393-077C088C", "0000000000000000", "696560", "696560", "14262", "2190", "4869", "0", "3", "160", "160", "0", "3316.10", "13199.07", "2232", "5.3044", "470"];
        let parsed = AdvancedParams::parse(&line);
        println!("{:?}", parsed);

        let line = vec!["Player-1335-0A264B4C", "0000000000000000", "621960", "621960", "12071", "1488", "4067", "0", "3|4", "43|6", "300|6", "25|6", "3471.75", "13115.98", "2232", "0.4119", "455"];
        let parsed = AdvancedParams::parse(&line);
        println!("{:?}", parsed);
    }
}