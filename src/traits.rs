pub trait FromRecord {
    fn parse_record(line: &[&str]) -> Self;
}


pub trait ToCamel {
    fn to_camel_case(self) -> String;
}

impl ToCamel for &str {
    fn to_camel_case(self) -> String {
        self.chars()
            .enumerate()
            .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c.to_ascii_lowercase() })
            .collect::<String>()
    }
}