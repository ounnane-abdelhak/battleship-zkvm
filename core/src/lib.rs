use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct VerifyInputHM {
    pub board: [[char; 10]; 10],
    pub salt: String,
    pub guess: [usize; 2],
    pub commitment: [u8; 32],
    pub round: usize,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifyInputSS {
    pub cboard: [[char; 10]; 10],
    pub salt: String,
    pub guess: [usize; 2],
    pub commitment: [u8; 32],
    pub round: usize,
    pub ships: [Ship;5],
}
