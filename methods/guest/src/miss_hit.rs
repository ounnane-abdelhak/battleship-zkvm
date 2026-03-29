use risc0_zkvm::guest::{entry, env};
entry!(main);

use core::VerifyInputHM;
use sha2::{Sha256, Digest};



fn main() {
    let mut hasher = Sha256::new();
    let inputs: VerifyInputHM = env::read();

    assert!(inputs.guess[0] < 10 && inputs.guess[1] < 10, "invalid guess");

    let is_hit = inputs.board[inputs.guess[0]][inputs.guess[1]]=='S';
    for i in inputs.board.iter(){
        for j in i.iter(){
            hasher.update([*j as u8]);
        }
    }
    hasher.update(&inputs.salt.as_bytes());
    let result: [u8; 32] = hasher.finalize().into();
    assert_eq!(inputs.commitment, result, "commitment mismatch!");

    env::commit(&(is_hit, result, inputs.guess,inputs.round));



}
