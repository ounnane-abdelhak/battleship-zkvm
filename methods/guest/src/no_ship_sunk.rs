use risc0_zkvm::guest::env;
use core::{VerifyInputSS,Ship};
use sha2::{Sha256, Digest};


fn main() {
    let input:VerifyInputSS = env::read();

    let cboard: [[char; 10]; 10]=input.cboard;
    let salt: String=input.salt;
    let guess: [usize; 2]=input.guess;
    let commitment: [u8; 32]=input.commitment;
    let round: usize=input.round;
    let ships: [Ship;5]=input.ships;

    

}