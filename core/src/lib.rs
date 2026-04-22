use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VerifyInput {
    pub board: [[char; 10]; 10],
    pub cboard: [[char; 10]; 10],
    pub salt: String,
    pub guess: [usize; 2],
    pub commitment: [u8; 32],
    pub round: usize,
    pub ships: [Ship; SHIP_COUNT],
    pub pre_round_commitment: [u8; 32],
}

pub const SHIP_COUNT: usize = 5;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Coord {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Ship {
    pub start: Coord,
    pub end: Coord,
}
