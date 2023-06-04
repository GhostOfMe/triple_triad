use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use serde::Deserialize;
use serde_with::serde_as;

#[derive(Clone, Debug, Default)]
pub struct Rules {
    pub open: bool,
    pub random: bool,
    pub plus: bool,
    pub same: bool,
    pub same_wall: bool,
    pub elemental: bool,
    pub sudden_death: bool,
}

#[derive(Clone, Debug, Copy)]
pub enum DuelOutcome {
    Win,
    Lose,
    Draw,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Suit {
    Red,
    Blue,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Card {
    id: usize,
    pub name: String,
    pub level: u8,
    #[serde(rename = "powLeft")]
    pub pow_left: u8,
    #[serde(rename = "powRight")]
    pub pow_right: u8,
    #[serde(rename = "powTop")]
    pub pow_top: u8,
    #[serde(rename = "powBottom")]
    pub pow_bottom: u8,
    pub element: Option<Element>,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Fire,
    Ice,
    Thunder,
    Earth,
    Poison,
    Wind,
    Water,
    Holy,
}

impl Distribution<Element> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Element {
        match rng.gen_range(0..=7) {
            0 => Element::Fire,
            1 => Element::Ice,
            2 => Element::Thunder,
            3 => Element::Earth,
            4 => Element::Poison,
            5 => Element::Wind,
            6 => Element::Water,
            7 => Element::Holy,
            _ => unreachable!(),
        }
    }
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct CardAtlas {
    //#[serde_as(as = "Vec<(_)>")]
    pub cards: Vec<Card>,
}

impl Card {
    pub const fn rank_as_slice(&self) -> [u8; 4] {
        [self.pow_top, self.pow_right, self.pow_bottom, self.pow_left]
    }
}
impl CardAtlas {
    pub fn parse_atlas_json(filename: &str) -> Self {
        use std::fs::File;
        use std::io::BufReader;
        let path = std::path::PathBuf::from(filename);
        println!("Loading: {path:?}");
        let file = File::open(path).expect("Couldn't find the card atlas file");
        let buf_reader = BufReader::new(file);
        serde_json::from_reader(buf_reader).expect("Couldn't create the card atlas")
    }
}
