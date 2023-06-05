use std::sync::{Arc, Mutex};
use std::thread;

use crate::core::{Element, Rules, Suit};
use crate::graphics::{CardEntity, ElementEntity, ElementalEffect};
use crate::state::play_state::Hand;

use crate::utils;

use super::opponent::AiEvent;

const TIMEOUT: f32 = 1.0;

#[derive(Clone, Debug)]
struct Cell {
    card: Option<CardSimple>,
    element: Option<Element>,
}

#[derive(Debug, Clone, Copy)]
struct Rank {
    pub t: u8,
    pub b: u8,
    pub l: u8,
    pub r: u8,
}

#[derive(Debug, Clone, Copy)]
struct CardSimple {
    rank: Rank,
    rank_elem: Rank,
    pub suit: Suit,
    pub element: Option<Element>,
}

impl From<&CardEntity> for CardSimple {
    fn from(value: &CardEntity) -> Self {
        let t = value.rank_top();
        let b = value.rank_bottom();
        let l = value.rank_left();
        let r = value.rank_right();

        let rank = Rank { t, b, l, r };
        let suit = value.controller;
        let element = value.element();

        let mut res = Self {
            rank,
            rank_elem: rank,
            suit,
            element,
        };
        match value.elemental_effect {
            ElementalEffect::Malus => res.add_malus(),
            ElementalEffect::None => {}
            ElementalEffect::Bonus => res.add_bonus(),
        }

        res
    }
}

impl CardSimple {
    pub fn add_bonus(&mut self) {
        self.rank_elem.t += 1;
        self.rank_elem.b += 1;
        self.rank_elem.l += 1;
        self.rank_elem.r += 1;
    }

    pub fn add_malus(&mut self) {
        self.rank_elem.t = self.rank_elem.t.saturating_sub(1);
        self.rank_elem.b = self.rank_elem.b.saturating_sub(1);
        self.rank_elem.l = self.rank_elem.l.saturating_sub(1);
        self.rank_elem.r = self.rank_elem.r.saturating_sub(1);
    }

    pub fn flip_suit(&mut self) {
        match self.suit {
            Suit::Red => self.suit = Suit::Blue,
            Suit::Blue => self.suit = Suit::Red,
        }
    }
}

#[derive(Clone, Debug)]
struct BoardSimple {
    pub red_hand: [Option<CardSimple>; 5],
    pub blue_hand: [Option<CardSimple>; 5],
    pub board: [Cell; 9],
    pub rules: Rules,
}

impl BoardSimple {
    pub fn new(
        red_hand: &Hand,
        blue_hand: &Hand,
        cards: &[Option<CardEntity>],
        elem: &[Option<ElementEntity>; 9],
        rules: &Rules,
    ) -> Self {
        let red_hand_vec: Vec<Option<CardSimple>> = red_hand
            .cards
            .iter()
            .map(|maybe_card| maybe_card.as_ref().map(CardSimple::from))
            .collect();

        let mut red_hand: [Option<CardSimple>; 5] = [None; 5];
        red_hand[..5].copy_from_slice(&red_hand_vec[..5]);

        let blue_hand_vec: Vec<Option<CardSimple>> = blue_hand
            .cards
            .iter()
            .map(|maybe_card| maybe_card.as_ref().map(CardSimple::from))
            .collect();

        let mut blue_hand: [Option<CardSimple>; 5] = [None; 5];

        blue_hand[..5].copy_from_slice(&blue_hand_vec[..5]);

        let board: [Cell; 9] = std::array::from_fn(|i| {
            let card = cards[i].as_ref().map(CardSimple::from);
            let element = elem[i].as_ref().map(|e| e.element);
            Cell { card, element }
        });

        Self {
            red_hand,
            blue_hand,
            board,
            rules: rules.clone(),
        }
    }

    pub fn put_card(&mut self, mut card: CardSimple, n: usize) {
        match (card.element, self.board[n].element) {
            (Some(c), Some(b)) if c == b => card.add_bonus(),
            (_, None) => {}
            (_, _) => card.add_malus(),
        }
        self.board[n].card = Some(card);
    }

    pub fn check(&mut self, n: usize, combo: bool) {
        assert!(n < 9);
        assert!(self.board[n].card.is_some());

        let border_mask: [Option<usize>; 4] = [
            if n >= 3 { Some(n - 3) } else { None },
            if n % 3 <= 1 { Some(n + 1) } else { None },
            if n / 3 <= 1 { Some(n + 3) } else { None },
            if n % 3 >= 1 { Some(n - 1) } else { None },
        ];

        let card_tmp = self.board[n].card.as_ref().unwrap();

        let ranks = [
            card_tmp.rank.t,
            card_tmp.rank.r,
            card_tmp.rank.b,
            card_tmp.rank.l,
        ];

        let ranks_elem = [
            card_tmp.rank_elem.t,
            card_tmp.rank_elem.r,
            card_tmp.rank_elem.b,
            card_tmp.rank_elem.l,
        ];

        let ranks_other: [Option<u8>; 4] = core::array::from_fn(|i| {
            if let Some(idx) = border_mask[i].as_ref() {
                if let Some(card) = self.board[*idx].card.as_ref() {
                    if card.suit == self.board[n].card.as_ref().unwrap().suit {
                        return None;
                    }
                    return match i {
                        0 => Some(card.rank.b),
                        1 => Some(card.rank.l),
                        2 => Some(card.rank.t),
                        3 => Some(card.rank.r),
                        _ => unreachable!(),
                    };
                }
            }
            None
        });

        let ranks_other_wall: [Option<u8>; 4] = core::array::from_fn(|i| {
            if let Some(idx) = border_mask[i].as_ref() {
                if let Some(card) = self.board[*idx].card.as_ref() {
                    if card.suit == self.board[n].card.as_ref().unwrap().suit {
                        return None;
                    }
                    return match i {
                        0 => Some(card.rank.b),
                        1 => Some(card.rank.l),
                        2 => Some(card.rank.t),
                        3 => Some(card.rank.r),
                        _ => unreachable!(),
                    };
                }
            }
            Some(10)
        });

        let ranks_other_elem: [Option<u8>; 4] = core::array::from_fn(|i| {
            if let Some(idx) = border_mask[i].as_ref() {
                if let Some(card) = self.board[*idx].card.as_ref() {
                    if card.suit == self.board[n].card.as_ref().unwrap().suit {
                        return None;
                    }
                    return match i {
                        0 => Some(card.rank_elem.b),
                        1 => Some(card.rank_elem.l),
                        2 => Some(card.rank_elem.t),
                        3 => Some(card.rank_elem.r),
                        _ => unreachable!(),
                    };
                }
            }
            None
        });

        let cards_flipped_same = if self.rules.same && combo {
            if self.rules.same_wall {
                let wall_tmp = utils::check_same(ranks, ranks_other_wall);
                core::array::from_fn(|i| wall_tmp[i] && border_mask[i].is_some())
            } else {
                utils::check_same(ranks, ranks_other)
            }
        } else {
            [false; 4]
        };

        let cards_flipped_plus = if self.rules.plus && combo {
            utils::check_plus(ranks, ranks_other)
        } else {
            [false; 4]
        };

        let card_flipped = utils::check_normal(ranks_elem, ranks_other_elem);

        // Multi zip
        //
        // https://stackoverflow.com/questions/29669287/how-can-i-zip-more-than-two-iterators
        //

        macro_rules! zip {
            ($x: expr) => ($x);
            ($x: expr, $($y: expr), +) => (
                $x.iter().zip(
                    zip!($($y), +))
            )
        }

        for (i, (normal, (same, plus))) in
            zip!(card_flipped, cards_flipped_same, cards_flipped_plus).enumerate()
        {
            if *same {
                self.board[border_mask[i].unwrap()]
                    .card
                    .as_mut()
                    .unwrap()
                    .flip_suit();
                self.check(border_mask[i].unwrap(), false);
                continue;
            }
            if plus {
                self.board[border_mask[i].unwrap()]
                    .card
                    .as_mut()
                    .unwrap()
                    .flip_suit();
                self.check(border_mask[i].unwrap(), false);
                continue;
            }
            if *normal {
                self.board[border_mask[i].unwrap()]
                    .card
                    .as_mut()
                    .unwrap()
                    .flip_suit();
            }
        }
    }

    fn score(&self) -> (usize, usize) {
        let mut red = self.red_hand.iter().flatten().count();
        let mut blue = self.blue_hand.iter().flatten().count();

        for c in self.board.iter().filter_map(|cell| cell.card) {
            match c.suit {
                Suit::Red => red += 1,
                Suit::Blue => blue += 1,
            }
        }
        (red, blue)
    }
}

fn solve_recur(turn: Suit, board: &BoardSimple, depth: usize) -> (usize, usize) {
    if depth < 1 {
        return board.score();
    }

    if board.board.iter().filter(|c| c.card.is_none()).count() == 0 {
        return board.score();
    }

    // if board.red_hand.iter().flatten().count() == 0 {
    //     return board.score();
    // }

    // if board.blue_hand.iter().flatten().count() == 0 {
    //     return board.score();
    // }

    let hand = match turn {
        Suit::Blue => &board.blue_hand,
        Suit::Red => &board.red_hand,
    };

    if hand.iter().flatten().count() == 0 {
        return board.score();
    }

    let mut best_score = (0, 0);

    for (n, c) in hand.iter().enumerate() {
        for m in 0..9 {
            if c.is_none() {
                continue;
            }
            if board.board[m].card.is_some() {
                continue;
            }
            let mut new_board = board.clone();
            match turn {
                Suit::Red => {
                    let card = new_board.red_hand[n].take().unwrap();
                    new_board.put_card(card, m);
                    new_board.check(m, true);
                }
                Suit::Blue => {
                    let card = new_board.blue_hand[n].take().unwrap();
                    new_board.put_card(card, m);
                    new_board.check(m, true);
                }
            }
            let new_turn = match turn {
                Suit::Red => Suit::Blue,
                Suit::Blue => Suit::Red,
            };
            let score = solve_recur(new_turn, &new_board, depth - 1);
            match turn {
                Suit::Red => {
                    if score.0 > best_score.0 {
                        best_score = score;
                    }
                }
                Suit::Blue => {
                    if score.1 > best_score.1 {
                        best_score = score;
                    }
                }
            }
        }
    }

    best_score
}

#[derive(Debug, Clone)]
pub enum Action {
    Check(usize),
    Put(usize, usize),
    PutBest,
}
#[derive(Debug, Clone)]
enum ThreadStatus {
    Waiting,
    Active,
    Finished,
}

#[derive(Debug, Clone)]
struct Move {
    pub from: usize,
    pub to: usize,
    pub score: usize,
}

impl Move {
    pub const fn new() -> Self {
        Self {
            from: 0,
            to: 0,
            score: 0,
        }
    }
}

pub struct Opponent {
    pub diffuculty: usize,
    pub actions: Vec<Action>,
    maybe_move: Option<Move>,
    timer: f32,
    solve_result: Arc<Mutex<Move>>,
    thread_status: Arc<Mutex<ThreadStatus>>,
}

impl Opponent {
    pub fn new() -> Self {
        let solve_result = Arc::new(Mutex::new(Move::new()));
        let thread_status = Arc::new(Mutex::new(ThreadStatus::Waiting));

        Self {
            diffuculty: 1,
            actions: vec![],
            maybe_move: None,
            timer: 0.0,
            solve_result,
            thread_status,
        }
    }

    // pub fn clear(&mut self) {
    //     self.actions.clear();
    //     self.timer = 0.0;
    //     self.maybe_move = None;
    // }

    pub fn diffuculty(&mut self, value: usize) {
        self.diffuculty = value.min(5);
    }

    pub fn think(
        &mut self,
        dt: f32,
        red_hand: &mut Hand,
        blue_hand: &mut Hand,
        field: &[Option<CardEntity>; 9],
        elem: &[Option<ElementEntity>; 9],
        rules: &Rules,
    ) -> Option<AiEvent> {
        self.timer -= dt;

        if self.timer >= 0.0 {
            return None;
        }
        if matches!(*self.thread_status.lock().unwrap(), ThreadStatus::Active) {
            return None;
        }

        let board = BoardSimple::new(red_hand, blue_hand, field, elem, rules);
        if self.actions.is_empty() {
            self.maybe_move = None;
            self.actions.push(Action::PutBest);

            for (i, _) in red_hand
                .cards
                .iter()
                .enumerate()
                .filter(|(_, maybe_card)| maybe_card.is_some())
                .rev()
            {
                self.actions.push(Action::Check(i));
            }
        }

        let current_action = self.actions.last().cloned().expect("state stack is empty");
        let thread_status = &self.thread_status.lock().unwrap().clone();

        match current_action {
            Action::Check(i) => match thread_status {
                ThreadStatus::Waiting => {
                    *self.thread_status.lock().unwrap() = ThreadStatus::Active;
                    *self.solve_result.lock().unwrap() = Move::new();
                    red_hand.set_focus(i);
                    self.solve(i, &board, self.diffuculty);
                    self.timer = TIMEOUT;
                    return Some(AiEvent::Focus);
                }
                ThreadStatus::Active => return None,
                ThreadStatus::Finished => {
                    *self.thread_status.lock().unwrap() = ThreadStatus::Waiting;
                    match &self.maybe_move {
                        None => {
                            self.maybe_move = Some(self.solve_result.lock().unwrap().clone());
                        }
                        Some(m) => {
                            let new_move = self.solve_result.lock().unwrap().clone();
                            if new_move.score > m.score {
                                self.maybe_move = Some(new_move);
                            }
                        }
                    }
                    self.actions.pop();
                }
            },
            Action::Put(from, to) => {
                red_hand.set_focus(from);
                red_hand.select_focused();
                self.actions.clear();
                return Some(AiEvent::Put(to));
            }
            Action::PutBest => {
                let from = self.maybe_move.as_ref().unwrap().from;
                let to = self.maybe_move.as_ref().unwrap().to;
                red_hand.set_focus(from);
                self.timer = TIMEOUT;
                self.actions.pop();
                self.actions.push(Action::Put(from, to));
                return Some(AiEvent::Focus);
            }
        }
        None
    }

    fn solve(&mut self, n: usize, board: &BoardSimple, depth: usize) {
        //if only one space left

        if board
            .board
            .iter()
            .filter(|cell| cell.card.is_none())
            .count()
            == 1
        {
            let board_tmp = board.clone();
            let res_clone = Arc::clone(&self.solve_result);
            let sts_clone = Arc::clone(&self.thread_status);
            thread::spawn(move || {
                let from = board_tmp
                    .red_hand
                    .iter()
                    .position(std::option::Option::is_some)
                    .unwrap();
                let to = board_tmp
                    .board
                    .iter()
                    .position(|c| c.card.is_none())
                    .unwrap();
                let best_move = Move { from, to, score: 5 };
                *res_clone.lock().unwrap() = best_move;
                *sts_clone.lock().unwrap() = ThreadStatus::Finished;
            });
        }
        let board_tmp = board.clone();
        let res_clone = Arc::clone(&self.solve_result);
        let sts_clone = Arc::clone(&self.thread_status);
        thread::spawn(move || {
            let mut best_move = Move {
                from: 0,
                to: 0,
                score: 0,
            };
            for i in 0..9 {
                if board_tmp.board[i].card.is_some() {
                    continue;
                }
                let mut new_board = board_tmp.clone();
                let card = new_board.red_hand[n].take().unwrap();
                new_board.put_card(card, i);
                new_board.check(i, true);
                let score = solve_recur(Suit::Blue, &new_board, depth - 1);
                if score.0 > best_move.score {
                    best_move = Move {
                        from: n,
                        to: i,
                        score: score.0,
                    };
                }
            }

            *res_clone.lock().unwrap() = best_move;
            *sts_clone.lock().unwrap() = ThreadStatus::Finished;
        });
    }
}

// fn can_flip_any(
//     card: &CardEntity,
//     field: &[Option<CardEntity>; 9],
//     elem: &[Option<ElementEntity>; 9],
// ) -> Option<usize> {
//     let pow_slice = [
//         card.rank_top_with_elemental(),
//         card.rank_right_with_elemental(),
//         card.rank_bottom_with_elemental(),
//         card.rank_left_with_elemental(),
//     ];
//     for (i, _) in field
//         .iter()
//         .enumerate()
//         .filter(|(_, maybe_card)| maybe_card.is_none())
//     {
//         let mut can_flip = [false; 4];
//         let elem_bonus = match (card.element(), elem[i].as_ref()) {
//             (None, Some(_)) => -1,
//             (Some(this), Some(other)) if this != other.element => -1,
//             (Some(this), Some(other)) if this == other.element => 1,
//             (_, _) => 0,
//         };

//         if let Some(top) = rank_to_the_top(i, Suit::Red, field) {
//             can_flip[0] = (pow_slice[0] as i8 + elem_bonus) as u8 > top;
//         }

//         if let Some(right) = rank_to_the_right(i, Suit::Red, field) {
//             can_flip[1] = (pow_slice[1] as i8 + elem_bonus) as u8 > right;
//         }

//         if let Some(bottom) = rank_to_the_bottom(i, Suit::Red, field) {
//             can_flip[2] = (pow_slice[2] as i8 + elem_bonus) as u8 > bottom;
//         }

//         if let Some(left) = rank_to_the_left(i, Suit::Red, field) {
//             can_flip[3] = (pow_slice[3] as i8 + elem_bonus) as u8 > left;
//         }

//         if can_flip.iter().any(|item| *item) {
//             return Some(i);
//         }
//     }
//     None
// }

// fn find_first_empty(field: &[Option<CardEntity>; 9]) -> usize {
//     let (first_empty, _) = field
//         .iter()
//         .enumerate()
//         .find(|(_, maybe_card)| maybe_card.is_none())
//         .expect("Hand is empty");
//     first_empty
// }
