use std::ops::Range;

pub mod miner;
mod init;
mod shader;
mod worker;

pub struct MinoeringResult {
    pub nonce: u64,
    pub hashes: Vec<u32>,
    pub minoers_mined: usize,
}

impl MinoeringResult {
    pub fn new(nonce: u64, hashes: Vec<u32>, minoers_mined: usize) -> MinoeringResult{
        MinoeringResult {
            nonce,
            hashes,
            minoers_mined,
        }
    }
}

#[derive(Clone)]
pub enum MinoerControlType {
    Stop,
    Start([u32; 10], u64, Range<u64>)
}