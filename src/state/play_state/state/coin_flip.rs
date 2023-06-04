use rand::{thread_rng, Rng};

use crate::core::Suit;

pub struct CoinFlip{
    
}

impl CoinFlip{
    pub fn first() -> Suit{
        if thread_rng().gen::<f32>() > 0.5{
            Suit::Red
        }else{
            Suit::Blue
        }
    }
}
