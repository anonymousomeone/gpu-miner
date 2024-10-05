use reqwest::blocking::Client;
use serde::Deserialize;
use crate::jason::ConfigFile;

#[derive(Deserialize, Debug)]
pub struct UserHashSetResponse {
    pub balance: u32,
    pub reward: u32,
    pub newhash: String,
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct UserHashGetResponse {
    hash: String,
    status: String,
}

pub fn send_to_server(client: &Client, config: &ConfigFile, nonce: u64) -> Option<UserHashSetResponse> {
    let res = client.post("https://gabserver.eu/v1/userhashset")
        .body("{\"username\":\"".to_string() + &config.username +"\",\"password\":\"" + &config.password + "\",\"threadid\":0,\"nonce\":" + &nonce.to_string() +"}")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .unwrap();

    // RAAH I LOVE RUSSY
    res.json::<UserHashSetResponse>().ok()
}

pub fn get_hash(client: &Client, config: &ConfigFile) -> String {
    let res = client.post("https://gabserver.eu/v1/userhashget")
        .body("{\"username\":\"".to_string() + &config.username +"\",\"password\":\"" + &config.password +"\",\"threadid\":0}")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .unwrap();

    let res = res.json::<UserHashGetResponse>().expect("hash get failed (unreal engine)");

    return res.hash;
}