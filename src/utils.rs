use std::any::type_name;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use num_traits::Num;
use regex::Regex;

pub fn parse_num<T: FromStr>(x: &str) -> Result<T>
{
    // https://github.com/dtolnay/anyhow/issues/318
    T::from_str(x).map_err(|_| anyhow!("Failed to parse {}: {:?}", type_name::<T>(), x))
}

/// Either nil-1 or 0-1 variants
pub fn parse_bool(x: &str) -> Result<bool> {
    match x {
        // https://warcraft.wiki.gg/wiki/COMBAT_LOG_EVENT#Death_Events
        "nil" | "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(anyhow!("Failed to parse bool: {:?}", x))
    }
}

pub fn parse_hex<T: FromStr + Num>(x: &str) -> Result<T> {
    T::from_str_radix(x.trim_start_matches("0x"), 16)
        .map_err(|_| anyhow!("Error parsing hex: {:?}", x))
}

/// Extracts and replaces the given regex, returning it
pub fn match_replace_all(re: &Regex, s: &str) -> (Vec<String>, String) {
    let matches = re.find_iter(s)
        .map(|m| m.as_str().to_string())
        .collect::<Vec<_>>();

    let s = re.replace_all(s, "").to_string();

    (matches, s)
}