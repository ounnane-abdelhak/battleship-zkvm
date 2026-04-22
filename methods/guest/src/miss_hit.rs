#![no_main]
use risc0_zkvm::guest::{entry, env};
entry!(main);

use core::{VerifyInput,Ship};
use risc0_zkvm::sha::rust_crypto::{Sha256, Digest};




fn main() {
    let input: VerifyInput = env::read();

    let board: [[char; 10]; 10]=input.board;
    let cboard: [[char; 10]; 10]=input.cboard;
    let salt: String=input.salt;
    let guess: [usize; 2]=input.guess;
    let commitment: [u8; 32]=input.commitment;
    let round: usize=input.round;
    let ships: [Ship; 5] = input.ships;
    let pre_round_commitment:[u8; 32]=input.pre_round_commitment;



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

    


    let mut pre_round_commit_hasher=Sha256::new();

    for i in cboard.iter(){
        for j in i.iter(){
            pre_round_commit_hasher.update([*j as u8]);
        }
    }

    pre_round_commit_hasher.update((round as u64).to_le_bytes());

    pre_round_commit_hasher.update(&commitment);    

    let pre_round_commit: [u8; 32] = pre_round_commit_hasher.finalize().into();

    assert_eq!(pre_round_commitment,pre_round_commit,"not same pre-round commitment");



    env::commit(&(is_hit, result, guess,round));


}
