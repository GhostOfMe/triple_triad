use crate::core::DuelOutcome;

#[derive(Debug, Copy, Clone)]
pub enum Rule {
    Open,
    Elemental,
    Random,
    Same,
    Wall,
    Plus,
    SuddenDeath,
}

impl Rule {
    pub fn iterator() -> impl Iterator<Item = &'static Self> {
        use Rule::{Elemental, Open, Plus, Random, Same, SuddenDeath, Wall};
        [Open, Elemental, Random, Same, Wall, Plus, SuddenDeath].iter()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Event {
    Play,
    Quit,
    Finished,
    GameSummary(DuelOutcome, bool),
    ChangeRule(Rule),
    ChangeDifficulty,
    ToggleCards(usize),
    PlaySound(Sfx),
    None,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Sfx {
    Move,
    Select,
    Cancel,
    Fanfare,
    Flip,
}

pub fn border_mask(n: usize) -> [Option<usize>; 4] {
    [
        if n >= 3 { Some(n - 3) } else { None },
        if n % 3 <= 1 { Some(n + 1) } else { None },
        if n / 3 <= 1 { Some(n + 3) } else { None },
        if n % 3 >= 1 { Some(n - 1) } else { None },
    ]
}

pub fn check_plus(this: [u8; 4], other: [Option<u8>; 4]) -> [bool; 4] {
    let mut res = [false; 4];
    let mut plus: [Option<u8>; 4] = core::array::from_fn(|i| {
        if let Some(r2) = other[i] {
            return Some(r2 + this[i]);
        }
        None
    });

    //
    // https://stackoverflow.com/questions/46766560/how-to-check-if-there-are-duplicates-in-a-slice
    //
    // let plus_applies = (1..plus.len()).any(|i| plus[i..].contains(&plus[i - 1]));
    //

    plus.sort_unstable();

    let mut dup_sums: [u8; 4] = [0; 4];

    for i in 0..3 {
        match (plus[i], plus[i + 1]) {
            (Some(p1), Some(p2)) if p1 == p2 => dup_sums[i] = p1,
            (_, _) => {}
        }
    }

    let plus_count = dup_sums.iter().filter(|n| **n > 0).count();

    if plus_count < 1 {
        return res;
    }

    for ((this, maybe_other), flip) in this.iter().zip(other.iter()).zip(res.iter_mut()) {
        if let Some(other) = maybe_other {
            if dup_sums.contains(&(this + other)) {
                *flip = true;
            }
        }
    }

    res
}

pub fn check_same(this: [u8; 4], other: [Option<u8>; 4]) -> [bool; 4] {
    let mut res = [false; 4];

    let same_count = this
        .iter()
        .zip(other.iter())
        .filter(|(r1, maybe_r2)| maybe_r2.map_or(false, |r2| *r1 == &r2))
        .count();

    if same_count < 2 {
        return res;
    }

    for ((this, maybe_other), flip) in this.iter().zip(other.iter()).zip(res.iter_mut()) {
        if let Some(other) = maybe_other {
            if this == other {
                *flip = true;
            }
        }
    }

    res
}

pub fn check_normal(this: [u8; 4], other: [Option<u8>; 4]) -> [bool; 4] {
    let mut res = [false; 4];

    for ((this, maybe_other), f) in this.iter().zip(other.iter()).zip(res.iter_mut()) {
        if let Some(other) = maybe_other {
            if this > other {
                *f = true;
            }
        }
    }

    res
}
