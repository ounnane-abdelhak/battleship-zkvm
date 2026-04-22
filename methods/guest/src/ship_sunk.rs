#![no_main]
use risc0_zkvm::guest::{entry, env};
entry!(main);
use core::{VerifyInput,Ship};
use risc0_zkvm::sha::rust_crypto::{Sha256, Digest};



fn ship_is_sunk(board: &[[char; 10]; 10], ship: &Ship) -> bool {
    let r1 = ship.start.row;
    let c1 = ship.start.col;
    let r2 = ship.end.row;
    let c2 = ship.end.col;


    if r1 >= 10 || c1 >= 10 || r2 >= 10 || c2 >= 10 {
        return false;
    }

    if r1 == r2 {
        let (a, b) = if c1 <= c2 { (c1, c2) } else { (c2, c1) };
        (a..=b).all(|c| board[r1][c] == 'X')
    } else if c1 == c2 {
        let (a, b) = if r1 <= r2 { (r1, r2) } else { (r2, r1) };
        (a..=b).all(|r| board[r][c1] == 'X')
    } else {

        false
    }
}

fn point_on_ship(ship: &Ship, row: usize, col: usize) -> bool {
    let r1 = ship.start.row;
    let c1 = ship.start.col;
    let r2 = ship.end.row;
    let c2 = ship.end.col;

    if r1 == r2 {
        let (a, b) = if c1 <= c2 { (c1, c2) } else { (c2, c1) };
        row == r1 && (a..=b).contains(&col)
    } else if c1 == c2 {
        let (a, b) = if r1 <= r2 { (r1, r2) } else { (r2, r1) };
        col == c1 && (a..=b).contains(&row)
    } else {
        false 
    }
}

fn find_hit_ship(ships: &[Ship; 5], guess: [usize; 2]) -> Option<usize>{
    let row = guess[0];
    let col = guess[1];

    for (idx, ship) in ships.iter().enumerate() {
        if point_on_ship(ship, row, col) {
            return Some(idx);
        }
    }
    None
    
}



fn main() {
    let input:VerifyInput = env::read();

    let board:[[char; 10]; 10]=input.board;
    let cboard: [[char; 10]; 10]=input.cboard;
    let salt: String=input.salt;
    let guess: [usize; 2]=input.guess;
    let commitment: [u8; 32]=input.commitment;
    let round: usize=input.round;
    let ships: [Ship;5]=input.ships;
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

    assert_eq!(is_hit, true, "no a hit");

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

    let s = find_hit_ship(&ships, guess).expect("hit must belong to a ship");


    let sunk =ship_is_sunk(&cboard, &ships[s]);
    assert_eq!(sunk,true,"no ship sunk");

    let mut win :bool=true ;
    for ship in ships.iter(){
        win= win && ship_is_sunk(&cboard, &ship);
    }

    env::commit(&(sunk,result,s,win,round,guess,cboard,pre_round_commit));
    
}
