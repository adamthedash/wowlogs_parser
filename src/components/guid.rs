use std::str::FromStr;

use anyhow::{bail, Context};
use anyhow::Result;
use strum::EnumString;

use crate::utils::parse_num;

#[derive(Debug)]
enum CastType {
    Local = 2,
    Active = 3,
    Passive = 4,
    TickA = 13,
    TickB = 16,
}

#[derive(Debug, EnumString)]
enum CreatureType {
    Creature,
    Pet,
    GameObject,
    Vehicle,
}


#[derive(Debug)]
pub enum GUID {
    BattlePet {
        id: u64
    },
    BNetAccount {
        account_id: u64
    },
    Cast {
        cast_type: CastType,
        server_id: u64,
        instance_id: u64,
        zone_uid: u64,
        spell_id: u64,
        cast_uid: u64,
    },
    ClientActor {
        x: u64,
        y: u64,
        z: u64,
    },
    Creature {
        unit_type: CreatureType,
        server_id: u64,
        instance_id: u64,
        zone_uid: u64,
        id: u64,
        spawn_uid: String,
    },
    // just a simple guid value
    Follower(u64),
    Item {
        server_id: u64,
        spawn_uid: u64,
    },
    Player {
        server_id: u64,
        player_uid: String,
    },
    Vignette {
        server_id: u64,
        instance_id: u64,
        zone_uid: u64,
        spawn_uid: u64,
    },
}

impl GUID {
    pub(crate) fn parse(s: &str) -> Result<Option<Self>> {
        if s == "0000000000000000" { return Ok(None); }

        let parts = s.split('-').collect::<Vec<_>>();

        let matched = match parts[0] {
            "Player" =>
                Self::Player {
                    server_id: parse_num(parts[1])?,
                    player_uid: parts[2].to_string(),
                },
            "Pet" | "Creature" | "GameObject" | "Vehicle" =>
                Self::Creature {
                    unit_type: CreatureType::from_str(parts[0])
                        .with_context(|| format!("Error parsing CreatureType: {}", parts[0]))?,
                    server_id: parse_num(parts[2])?,
                    instance_id: parse_num(parts[3])?,
                    zone_uid: parse_num(parts[4])?,
                    id: parse_num(parts[5])?,
                    spawn_uid: parts[6].to_string(),
                },
            _ => bail!("GUID type not found: {}", parts[0])
        };

        Ok(Some(matched))
    }
}


#[cfg(test)]
mod tests {
    use crate::components::guid::GUID;

    #[test]
    fn parse() {
        let parsed = GUID::parse("0000000000000000");
        assert!(parsed.is_ok_and(|x| x.is_none()));

        let parsed = GUID::parse("Player-1403-0A5506C6");
        assert!(parsed.is_ok_and(|x| x.is_some()));

        let parsed = GUID::parse("Creature-0-1469-2549-12530-209333-000011428A");
        assert!(parsed.is_ok_and(|x| x.is_some()));
    }
}