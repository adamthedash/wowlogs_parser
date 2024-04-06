use std::str::FromStr;

use strum::EnumString;

use crate::guid::GUID::{Creature, Player};

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
        cast_type: u8,
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

impl FromStr for GUID {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split("-").collect::<Vec<_>>();

        match parts[0] {
            "Player" => {
                Ok(Player {
                    server_id: u64::from_str(parts[1]).unwrap(),
                    player_uid: parts[2].to_string(),
                })
            }
            "Pet" | "Creature" | "GameObject" | "Vehicle" => {
                Ok(
                    Creature {
                        unit_type: CreatureType::from_str(parts[0]).unwrap(),
                        server_id: u64::from_str(parts[2]).unwrap(),
                        instance_id: u64::from_str(parts[3]).unwrap(),
                        zone_uid: u64::from_str(parts[4]).unwrap(),
                        id: u64::from_str(parts[5]).unwrap(),
                        spawn_uid: parts[6].to_string(),
                    }
                )
            }
            _ => {
                panic!("GUID type not found: {}", parts[0]);
            }
        }
    }
}