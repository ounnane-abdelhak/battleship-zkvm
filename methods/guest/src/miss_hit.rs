#![no_main]
use risc0_zkvm::guest::{entry, env};
entry!(main);

use core::{VerifyInputHM,Ship};
use sha2::{Sha256, Digest};



fn main() {
    let input: VerifyInputHM = env::read();

    let board: [[char; 10]; 10]=input.board;
    let salt: String=input.salt;
    let guess: [usize; 2]=input.guess;
    let commitment: [u8; 32]=input.commitment;
    let round: usize=input.round;
    let ships: [Ship; 5] = input.ships;




    let mut hasher = Sha256::new();


    assert!(guess[0] < 10 && guess[1] < 10, "invalid guess");

    let is_hit = board[guess[0]][guess[1]]=='S';
    for i in board.iter(){
        for j in i.iter(){
            hasher.update([*j as u8]);
        }
    }

    for ship in ships.iter() {
    hasher.update((ship.start.row as u64).to_le_bytes());
    hasher.update((ship.start.col as u64).to_le_bytes());
    hasher.update((ship.end.row as u64).to_le_bytes());
    hasher.update((ship.end.col as u64).to_le_bytes());
    }


    hasher.update(&salt.as_bytes());

    
    let result: [u8; 32] = hasher.finalize().into();
    assert_eq!(commitment, result, "commitment mismatch");
    assert_eq!(is_hit, true, "no a hit");
    

    env::commit(&(is_hit, result, guess,round));


}
