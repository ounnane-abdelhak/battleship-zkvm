use std::cmp::max;

mod terminal;
mod logic;

fn to_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let (board1, salt1, ships1) = terminal::setup_player_terminal(1);
    let (board2, salt2, ships2) = terminal::setup_player_terminal(2);

    let boards = [board1, board2];
    let ships = [ships1, ships2];
    let salts = [salt1, salt2];

    let empty = [['.'; 10]; 10];
    let mut cboards = [empty, empty];

    let mut setup_hashes = ["pending".to_string(), "pending".to_string()];
    let mut round_idx: usize = 1;

    loop {
        let prover = if round_idx % 2 == 1 { 1 } else { 2 };
        let verifier = if prover == 1 { 2 } else { 1 };

        let hash_refs = [&setup_hashes[0], &setup_hashes[1]];
        let guess = terminal::rounds_terminal(
            verifier,
            &cboards,
            &hash_refs,
            0,
            None,
            round_idx as u32,
        );

        let result = logic::round(
            prover,
            guess,
            round_idx,
            &boards,
            &mut cboards,
            &ships,
            &salts,
        );
        


        setup_hashes[prover - 1] = to_hex(&result.setup_commitment);

        let hash_refs = [&setup_hashes[0], &setup_hashes[1]];
        terminal::rounds_terminal(
            verifier,
            &cboards,
            &hash_refs,
            1,
            Some(&result.verification),
            round_idx as u32,
        );

        if result.win {
            terminal::game_over_terminal(verifier, &salts, &setup_hashes);
            break;
        }

        round_idx += 1;
    }
}



