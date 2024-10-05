// who is jason and why cant he be parsed
use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use serde_json;

#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    pub username: String,
    pub password: String,
    pub lobotomize: bool,
}
pub fn read_config() -> ConfigFile {
    let file = File::open("./config.json").expect("no config.json idor");
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).expect("jason parse err")
}