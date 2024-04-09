use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use itertools::Itertools;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use crate::events::Event;

mod enums;
mod traits;
mod prefixes;
mod common_components;
mod guid;
mod suffixes;
mod utils;
mod special;
mod advanced;
mod events;


fn parse_file<R: Read>(buf_reader: R) where
{
    // Used to parse lines
    let mut binding = csv::ReaderBuilder::new();
    let mut reader = binding
        .has_headers(false)
        .flexible(true)
        .from_reader(buf_reader);


    reader.records()
        .filter_map(Result::ok)
        .map(|line| Event::parse(&line.iter().collect_vec()))
        .for_each(|e| {
            match e {
                Ok(x) => println!("{:?}", x),
                Err(x) => eprintln!("{}", x)
            }
        });
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    // Get the number of bytes currently in the file - we only want to tail it

    let mut prev_size = File::open(path)?.metadata()?.len();
    for res in rx {
        match res {
            Ok(event) => {
                let mut file = File::open(&event.paths[0])?;
                let new_size = file.metadata()?.len();

                file.seek(SeekFrom::Current(prev_size as i64))?;

                parse_file(BufReader::new(file));

                prev_size = new_size;
            }

            Err(x) => {}
        }
    }

    Ok(())
}

fn main() {
    let wowlog_path = PathBuf::from_str(r"E:\Games\Blizzard\World of Warcraft\_retail_\Logs\WoWCombatLog-040924_182641.txt").unwrap();
    watch(wowlog_path).unwrap();
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::PathBuf;
    use std::str::FromStr;

    use crate::parse_file;

    #[test]
    fn test1() {
        let wowlog_path = PathBuf::from_str(r"E:\Games\Blizzard\World of Warcraft\_retail_\Logs\WoWCombatLog-040624_135724.txt").unwrap();

        let file = File::open(wowlog_path)
            .expect("Error loading wowlogs file.");

        parse_file(file);
    }

    #[test]
    fn test2() {
        let wowlog_path = PathBuf::from_str("/test_data/WoWCombatLog-021524_201412.txt").unwrap();

        let file = File::open(wowlog_path)
            .expect("Error loading wowlogs file.");

        parse_file(file);
    }

    #[test]
    fn test3() {
        let file = "2/15 20:14:12.865  COMBAT_LOG_VERSION,20,ADVANCED_LOG_ENABLED,1,BUILD_VERSION,10.2.5,PROJECT_ID,1\n".as_bytes();

        parse_file(file);
    }
}

