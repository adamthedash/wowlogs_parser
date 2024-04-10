use anyhow::Result;

use crate::events::Event;

pub trait EventHandler {
    fn handle(&mut self, event: &Result<Event>);
}

pub struct StdLogger;

impl EventHandler for StdLogger {
    fn handle(&mut self, event: &Result<Event>) {
        match event {
            Ok(x) => println!("{:?}", x),
            Err(x) => eprintln!("{}", x)
        }
    }
}