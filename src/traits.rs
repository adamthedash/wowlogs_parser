use csv::StringRecord;

pub trait FromRecord {
    fn parse_record(line: &[&str]) -> Self;
}
