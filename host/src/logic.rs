use crate::terminal;

use core::{Ship, VerifyInput, SHIP_COUNT};
use methods::{
    MISS_HIT_ELF, MISS_HIT_ID, NO_SHIP_SUNK_ELF, NO_SHIP_SUNK_ID, SHIP_SUNK_ELF, SHIP_SUNK_ID,
};

use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};



pub struct PlayerProofBundle {
    pub player: usize,
    pub round: usize,
    pub guess: [usize; 2],
    pub setup_commitment: [u8; 32],
    pub miss_hit_receipt: Receipt,
    pub sunk_receipt: Option<Receipt>,
    pub next_cboard: [[char; 10]; 10],
}


pub struct RoundResult {
    pub prover: usize,
    pub verifier: usize,
    pub guess: [usize; 2],
    pub verification: terminal::VerificationResult,
    pub win: bool,
    pub setup_commitment: [u8; 32],
}




const SHIP_NAMES: [&str; SHIP_COUNT] = [
    "Carrier",
    "Battleship",
    "Cruiser",
    "Submarine",
    "Destroyer",
];

fn to_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
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

fn find_hit_ship(ships: &[Ship; SHIP_COUNT], guess: [usize; 2]) -> Option<usize> {
    ships
        .iter()
        .enumerate()
        .find(|(_, s)| point_on_ship(s, guess[0], guess[1]))
        .map(|(i, _)| i)
}

fn ship_is_sunk(cboard: &[[char; 10]; 10], ship: &Ship) -> bool {
    let r1 = ship.start.row;
    let c1 = ship.start.col;
    let r2 = ship.end.row;
    let c2 = ship.end.col;

    if r1 >= 10 || c1 >= 10 || r2 >= 10 || c2 >= 10 {
        return false;
    }

    if r1 == r2 {
        let (a, b) = if c1 <= c2 { (c1, c2) } else { (c2, c1) };
        (a..=b).all(|c| cboard[r1][c] == 'X')
    } else if c1 == c2 {
        let (a, b) = if r1 <= r2 { (r1, r2) } else { (r2, r1) };
        (a..=b).all(|r| cboard[r][c1] == 'X')
    } else {
        false
    }
}

fn is_hit_from_ships(ships: &[Ship; SHIP_COUNT], guess: [usize; 2]) -> bool {
    find_hit_ship(ships, guess).is_some()
}

fn compute_setup_commitment(board: &[[char; 10]; 10], ships: &[Ship; SHIP_COUNT], salt: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    for row in board.iter() {
        for cell in row.iter() {
            hasher.update([*cell as u8]);
        }
    }

    for ship in ships.iter() {
        hasher.update((ship.start.row as u64).to_le_bytes());
        hasher.update((ship.start.col as u64).to_le_bytes());
        hasher.update((ship.end.row as u64).to_le_bytes());
        hasher.update((ship.end.col as u64).to_le_bytes());
    }

    hasher.update(salt.as_bytes());
    hasher.finalize().into()
}

fn compute_round_commitment(cboard: &[[char; 10]; 10], round: usize, commitment: [u8; 32]) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    for row in cboard.iter() {
        for cell in row.iter() {
            hasher.update([*cell as u8]);
        }
    }

    hasher.update((round as u64).to_le_bytes());
    hasher.update(commitment);
    hasher.finalize().into()
}








pub fn round(
    prover: usize,
    guess: [usize; 2],
    round: usize,
    boards: &[[[char; 10]; 10]; 2],
    cboards: &mut [[[char; 10]; 10]; 2],
    ships: &[[Ship; SHIP_COUNT]; 2],
    salts: &[String; 2],
) -> RoundResult {
    assert!(prover == 1 || prover == 2, "prover must be 1 or 2");
    assert!(guess[0] < 10 && guess[1] < 10, "invalid guess");

    let prover_idx = prover - 1;
    let verifier = if prover == 1 { 2 } else { 1 };
    let verifier_idx = verifier - 1;

    assert_eq!(
        cboards[verifier_idx][guess[0]][guess[1]],
        '.',
        "cell already played"
    );

    let cboard_before = cboards[verifier_idx];
    let proofs = player_prove(
        prover,
        &boards[prover_idx],
        &cboard_before,
        &ships[prover_idx],
        &salts[prover_idx],
        guess,
        round,
    );

    let (verification, win) = player_verify(verifier, &proofs);
    cboards[verifier_idx] = proofs.next_cboard;

    RoundResult {
        prover,
        verifier,
        guess,
        verification,
        win,
        setup_commitment: proofs.setup_commitment,
    }
}






pub fn player_prove(
    player: usize,
    board: &[[char; 10]; 10],
    cboard_before: &[[char; 10]; 10],
    ships: &[Ship; SHIP_COUNT],
    salt: &str,
    guess: [usize; 2],
    round: usize,
) -> PlayerProofBundle {
    use std::time::Instant;

    let t_total = Instant::now();
    assert!(guess[0] < 10 && guess[1] < 10, "invalid guess");

    let t_commit = Instant::now();
    let setup_commitment = compute_setup_commitment(board, ships, salt);
    let pre_round_hm = compute_round_commitment(cboard_before, round, setup_commitment);


    let t_prover = Instant::now();
    let prover = default_prover();


    let hm_input = VerifyInput {
        board: *board,
        cboard: *cboard_before,
        salt: salt.to_string(),
        guess,
        commitment: setup_commitment,
        round,
        ships: *ships,
        pre_round_commitment: pre_round_hm,
    };

    let t_env_hm = Instant::now();
    let env_hm = ExecutorEnv::builder().write(&hm_input).unwrap().build().unwrap();


    let t_hm = Instant::now();
    let miss_hit_receipt = prover.prove(env_hm, MISS_HIT_ELF).unwrap().receipt;


    let t_decode_hm = Instant::now();
    let (is_hit, commitment_from_hm, guess_from_hm, round_from_hm): (
        bool,
        [u8; 32],
        [usize; 2],
        usize,
    ) = miss_hit_receipt.journal.decode().unwrap();

    assert_eq!(commitment_from_hm, setup_commitment, "hm commitment mismatch");
    assert_eq!(guess_from_hm, guess, "hm guess mismatch");
    assert_eq!(round_from_hm, round, "hm round mismatch");

    let mut next_cboard = *cboard_before;
    next_cboard[guess[0]][guess[1]] = if is_hit { 'X' } else { 'O' };

    let sunk_receipt = if is_hit {
        let t_prep_sunk = Instant::now();
        let pre_round_sunk = compute_round_commitment(&next_cboard, round, setup_commitment);

        let sunk_input = VerifyInput {
            board: *board,
            cboard: next_cboard,
            salt: salt.to_string(),
            guess,
            commitment: setup_commitment,
            round,
            ships: *ships,
            pre_round_commitment: pre_round_sunk,
        };

        let ship_idx = find_hit_ship(ships, guess).expect("hit must belong to a ship");
        let sunk_now = ship_is_sunk(&next_cboard, &ships[ship_idx]);

        let (elf, _id) = if sunk_now {
            (SHIP_SUNK_ELF, SHIP_SUNK_ID)
        } else {
            (NO_SHIP_SUNK_ELF, NO_SHIP_SUNK_ID)
        };

        let t_env_sunk = Instant::now();
        let env_sunk = ExecutorEnv::builder().write(&sunk_input).unwrap().build().unwrap();

        let t_sunk = Instant::now();
        let receipt = prover.prove(env_sunk, elf).unwrap().receipt;

        Some(receipt)
    } else {
        None
    };


    PlayerProofBundle {
        player,
        round,
        guess,
        setup_commitment,
        miss_hit_receipt,
        sunk_receipt,
        next_cboard,
    }
}


pub fn player_verify(player: usize, proofs: &PlayerProofBundle) -> (terminal::VerificationResult, bool) {
    assert!(player == 1 || player == 2, "player must be 1 or 2");

    proofs.miss_hit_receipt.verify(MISS_HIT_ID).unwrap();

    let (is_hit, commitment_from_hm, guess_from_hm, round_from_hm): (
        bool,
        [u8; 32],
        [usize; 2],
        usize,
    ) = proofs.miss_hit_receipt.journal.decode().unwrap();

    assert_eq!(commitment_from_hm, proofs.setup_commitment, "hm commitment mismatch");
    assert_eq!(guess_from_hm, proofs.guess, "hm guess mismatch");
    assert_eq!(round_from_hm, proofs.round, "hm round mismatch");

    let mut ship_sunk_name: Option<String> = None;
    let mut win = false;
    let mut sunk_commitment: Option<String>=None;
    if is_hit {
        let sunk_receipt = proofs
            .sunk_receipt
            .as_ref()
            .expect("hit requires sunk/no_sunk receipt");

        let verified_as_sunk = sunk_receipt.verify(SHIP_SUNK_ID).is_ok();
        if !verified_as_sunk {
            sunk_receipt.verify(NO_SHIP_SUNK_ID).unwrap();
        }

        let (sunk, commitment2, ship_idx, win2, round2, guess2, cboard2, pre2): (
            bool,
            [u8; 32],
            usize,
            bool,
            usize,
            [usize; 2],
            [[char; 10]; 10],
            [u8; 32],
        ) = sunk_receipt.journal.decode().unwrap();
        sunk_commitment = Some(to_hex(&pre2));
        assert_eq!(commitment2, proofs.setup_commitment, "sunk commitment mismatch");
        assert_eq!(guess2, proofs.guess, "sunk guess mismatch");
        assert_eq!(round2, proofs.round, "sunk round mismatch");
        assert_eq!(cboard2, proofs.next_cboard, "sunk cboard mismatch");

        let recomputed_pre = compute_round_commitment(&cboard2, round2, commitment2);
        assert_eq!(pre2, recomputed_pre, "sunk pre-round commitment mismatch");

        if verified_as_sunk {
            assert!(sunk, "expected sunk=true");
        } else {
            assert!(!sunk, "expected sunk=false");
        }

        if sunk {
            ship_sunk_name = Some(SHIP_NAMES[ship_idx].to_string());
        }
        win = win2;
    } else {
        assert!(
            proofs.sunk_receipt.is_none(),
            "miss should not carry sunk/no_sunk receipt"
        );
    }
    

    let verification = terminal::VerificationResult {
        is_hit,
        proof_valid: true,
        commitment: to_hex(&proofs.setup_commitment),
        ship_sunk: ship_sunk_name,
        sunk_commitment,
        
    };

    (verification, win)
}
