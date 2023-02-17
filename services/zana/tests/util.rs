use std::fs;

pub fn get_sample(sample: &str) -> String {
    fs::read_to_string(format!("tests/sample/{}", sample)).expect("could not read sample file")
}
