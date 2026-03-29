use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct VerifyInputHM {
    pub board: [[char; 10]; 10],
    pub salt: String,
    pub guess: [usize; 2],
    pub commitment: [u8; 32],
    pub round: usize,
}
