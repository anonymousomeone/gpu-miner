use std::env;
use std::sync::mpsc;
use std::time::Instant;

use modules::helpers;
use modules::jason;
use modules::mining::MinoeringResult;
use modules::network;
use modules::mining::miner;
use modules::mining::miner::DISPATCH_SIZE;

mod modules;

/** max dispatches for each thread
 */
const MAX_DISPATCHES: usize = 8;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    println!("Hello, world!");

    let (results_sender, results_reciever) = mpsc::channel::<MinoeringResult>();

    let mut miner = miner::Minoer::new(MAX_DISPATCHES, results_sender);
    let client = reqwest::blocking::Client::new();
    let config = jason::read_config();

    if !config.lobotomize {
        loop { println!("tampering detected!!!!!!") }
    }

    println!("minoering..");
    let mut hash = network::get_hash(&client, &config);
    // let hash = String::from("8deda67f452dc5de673a01fad1580ca4429bc166a4e3dc5d3911535616327e32");
    loop {
        // println!("Hash: {}", hash);
        let prehash = helpers::sha1_prehash(&hash);
        let mut data: [u32; 10] = [0; 10];
        let nonce: u64 = 10000000000000000000;

        data[0] = prehash[0];
        data[1] = prehash[1];
        data[2] = prehash[2];
        data[3] = prehash[3];
        data[4] = prehash[4];
    
        let instant: Instant = Instant::now();

        miner.mine(data, nonce);

        for result in results_reciever.recv() {
            let mut string = String::new();
            let real_nonce = result.nonce;
            let minoers_mined = result.minoers_mined;
    
            for x in 0..4 {
                let mut data = result.hashes[x].to_ne_bytes();
                data.reverse();
                string.push_str(&hex::encode(data));
            }

            let diff = Instant::now().duration_since(instant);
            let hashes = minoers_mined as u64 * DISPATCH_SIZE as u64 * 64;
            println!("Took {}s, looked through {} hashes, with ~{}h/s", diff.as_secs(), hashes, (hashes as f64 / (diff.as_millis() as f64 / 1000f64)) as u64);
            
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
            miner.stop_mining();
            println!();
            // break 'outer;
        }
    }
}