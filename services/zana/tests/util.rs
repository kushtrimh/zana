use std::fs;

#[cfg(test)]
pub fn get_sample(sample: &str) -> String {
    fs::read_to_string(format!("tests/sample/{}", sample)).expect("could not read sample file")
}

#[cfg(test)]
pub fn get_json_value(sample: &str) -> serde_json::Value {
    let sample = get_sample(sample);
    serde_json::from_str(&sample).expect("could not parse json")
}

#[cfg(test)]
pub fn set_property_to_null(sample: &str, pointer: &str) -> serde_json::Value {
    let mut v: serde_json::Value =
        serde_json::from_str(&get_sample(sample)).expect("could not parse json");
    v.pointer_mut(pointer)
        .expect(&format!("{} not part of the sample", pointer))
        .take();
    v
}
