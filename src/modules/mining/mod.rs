pub mod miner;
mod shader;
mod init;

pub struct MinoeringResult {
    pub nonce: u64,
    pub hashes: Vec<u32>
}

impl MinoeringResult {
    pub fn new(nonce: u64, hashes: Vec<u32>) -> MinoeringResult{
        MinoeringResult {
            nonce,
            hashes
        }
    }
}

pub enum MinoerControlType {
    Stop,
    Start([u32; 10], u64)
}