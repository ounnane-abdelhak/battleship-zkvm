use risc0_zkvm::guest::env;
use core::VerifyInputHM;
use sha2::{Sha256, Digest};


fn main() {
    let mut hasher = Sha256::new();
    let inputs: VerifyInputHM = env::read();
    let is_hit = inputs.board[inputs.guess[0]][inputs.guess[1]];
    for i in inputs.board.iter(){
        for j in i.iter(){
            hasher.update(&[*j as u8]);
        }
    }
    hasher.update(&inputs.salt.as_bytes());
    assert_eq!(inputs.commitment, result, "commitment mismatch!");

env::commit(&(is_hit, result, inputs.guess,inputs.round));



}
