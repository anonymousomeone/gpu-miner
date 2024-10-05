use std::env;
use std::time::Instant;

use modules::helpers;
use modules::jason;
use modules::network;
use modules::mining::miner;
use modules::mining::miner::DISPATCH_SIZE;

mod modules;

const MAX_DISPATCHES: usize = 4;
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    println!("Hello, world!");

    let mut miner = miner::Minoer::new(MAX_DISPATCHES);
    let client = reqwest::blocking::Client::new();
    let config = jason::read_config();

    if !config.lobotomize {
        loop { println!("tampering detected!!!!!!") }
    }

    println!("minoering..");
    let mut hash = network::get_hash(&client, &config);
    let mut minoers_mined = 0;
    // let hash = String::from("8deda67f452dc5de673a01fad1580ca4429bc166a4e3dc5d3911535616327e32");
    loop {
        // println!("Hash: {}", hash);
        let prehash = helpers::sha1_prehash(&hash);
        let mut data: [u32; 10] = [0; 10];
        let mut output: Vec<u32> = Vec::with_capacity(512);
        let nonce: u64 = 10000000000000000000;

        data[0] = prehash[0];
        data[1] = prehash[1];
        data[2] = prehash[2];
        data[3] = prehash[3];
        data[4] = prehash[4];
    
        let instant: Instant = Instant::now();
        'outer: for i in 0..99999 {
            minoers_mined += 1;
            let nonce = nonce + i * 10_u64.pow(10);
            let nonce_arr = helpers::nonce_to_u32arr(nonce);
            data[5] = nonce_arr[0];
            data[6] = nonce_arr[1];
            data[7] = nonce_arr[2];
            data[8] = nonce_arr[3];
            data[9] = nonce_arr[4];
            miner.mine(data, nonce);

            if (i + 1) % MAX_DISPATCHES as u64 == 0 && (i != 0 || MAX_DISPATCHES == 1) {
                let results = miner.get_results();

                for result in results {
                    let mut string = String::new();
                    let real_nonce = result.nonce;
            
                    for x in 0..4 {
                        let mut data = result.hashes[x].to_ne_bytes();
                        data.reverse();
                        string.push_str(&hex::encode(data));
                    }
                    
                    let res = network::send_to_server(&client, &config, real_nonce);
    
                    match res {
                        Some(r) => {
                            hash = r.newhash;
                            println!("Nonce got: {}, Hash: {}, Reward: {}", real_nonce, string, r.reward);
                        },
                        None => {
                            println!("bad nonce: {}, hash: {}, source hash {}", real_nonce, string, hash);
                            hash = network::get_hash(&client, &config);
                        },
                    }
                    break 'outer;
                }
            }
    
            output.clear();
        }

        let diff = Instant::now().duration_since(instant);
        let hashes = minoers_mined as u64 * DISPATCH_SIZE as u64 * 64;
        println!("Took {}s, looked through {} hashes, with ~{}h/s", diff.as_secs(), hashes, (hashes as f64 / (diff.as_millis() as f64 / 1000f64)) as u64);
        minoers_mined = 0;
    }
}