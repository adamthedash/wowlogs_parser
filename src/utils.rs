use std::any::type_name;
use std::fmt::Debug;
use std::str::FromStr;

pub fn parse_num<T: FromStr>(x: &str) -> T
    where <T as FromStr>::Err: Debug {
    T::from_str(x).expect(&format!("Failed to parse {}: {}", type_name::<T>(), x))
}

/// Either nil-1 or 0-1 variants
pub fn parse_bool(x: &str) -> bool {
    match x {
        // https://warcraft.wiki.gg/wiki/COMBAT_LOG_EVENT#Death_Events
        "nil" | "0" => false,
        "1" => true,
        _ => panic!("Failed to parse bool: {}", x)
    }
}