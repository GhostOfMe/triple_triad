use ggez::graphics::{Canvas, Image, InstanceArray};
// use ggez::graphics::{DrawParam, Text};

use ggez::Context;
use std::rc::Rc;

use crate::core::DuelOutcome;
use crate::core::{CardAtlas, Suit};
use crate::graphics::sprite::Atlas;
use crate::utils::{Event, Sfx};

mod ai;
mod opponent;
pub mod state;

pub use state::Hand;

use state::Banner;
use state::Board;
use state::CardPicker;
use state::CoinFlip;
use state::Menu;
#[allow(dead_code)]
#[derive(Debug)]
enum State {
    Menu,
    CardPick,
    CoinFlip,
    Play,
    Fin,
}

// pub struct GameSummary {
//     outcome: DuelOutcome,
//     sudden_death: bool,
// }

pub struct PlayState {
    menu: Menu,
    card_pick: CardPicker,
    _coin_flip: CoinFlip,
    play: Board,
    fin: Banner,
    state_stack: Vec<State>,
}

impl PlayState {
    pub fn new(
        ctx: &mut Context,
        card_atlas: &Rc<CardAtlas>,
        sprite_sheet: &Rc<Atlas>,
        bg_image: &Rc<Image>,
    ) -> Self {
        let menu = Menu::new(ctx);
        let card_pick = CardPicker::new(ctx, card_atlas, sprite_sheet);
        let _coin_flip = CoinFlip {};
        let play = Board::empty(ctx, card_atlas, sprite_sheet, bg_image);
        let fin = Banner::new(sprite_sheet);
        let state_stack = vec![State::Play];
        Self {
            menu,
            card_pick,
            _coin_flip,
            play,
            fin,
            state_stack,
        }
    }
    pub fn init(&mut self) {
        self.play.init();
        self.menu.init(&self.play.rules);
        self.card_pick.init();
        self.state_stack.clear();
        self.fin.init();
        self.state_stack.push(State::Play);
        self.state_stack.push(State::CoinFlip);
        self.state_stack.push(State::Menu);
    }
    pub fn clear(&mut self) {
        self.state_stack.clear();
        self.play.clear();
        self.state_stack.push(State::Play);
        self.state_stack.push(State::CoinFlip);
    }
    pub fn update(&mut self, ctx: &mut Context) -> Option<Event> {
        if self.state_stack.is_empty() {
            return Some(Event::Quit);
        }

        if let Some(state) = self.state_stack.last() {
            match state {
                State::Menu => return self.update_menu(ctx),
                State::CardPick => {
                    if let Some((id, count)) = self.card_pick.update(ctx) {
                        self.play.blue_hand.add_card(id, count, true);
                        if count >= 4 {
                            self.state_stack.pop();
                            self.play.next_state();
                        }
                        return Some(Event::PlaySound(Sfx::Select));
                    }
                    self.play.update(ctx);
                }
                State::CoinFlip => {
                    self.play.first_turn(CoinFlip::first());
                    self.state_stack.pop();
                }
                State::Play => {
                    if let Some(e) = self.play.update(ctx) {
                        match e {
                            Event::GameSummary(DuelOutcome::Draw, true) => {
                                self.play.deal_sunnden_death();
                                self.state_stack.push(State::CoinFlip);
                                return None;
                            }
                            Event::GameSummary(outcome, _) => {
                                self.fin.outcome = outcome;
                                self.state_stack.push(State::Fin);
                            }
                            Event::PlaySound(s) => {
                                return Some(Event::PlaySound(s));
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                State::Fin => {
                    if let Some(e) = self.fin.update(ctx) {
                        match e {
                            Event::Finished => return Some(Event::Finished),
                            Event::PlaySound(s) => return Some(Event::PlaySound(s)),
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
        None
    }

    pub fn turn_marker_status(&self) -> [bool; 2] {
        self.play.turn_marker_status()
    }

    fn update_menu(&mut self, ctx: &mut Context) -> Option<Event> {
        if let Some(e) = self.menu.update(
            self.play.opponent.difficulty(),
            &self.play.rules,
            &self.play.opponent.cards(),
            ctx,
        ) {
            match e {
                Event::Play => {
                    self.play.clear();
                    self.play.red_hand = Hand::from_ids(
                        Suit::Red,
                        false,
                        &self.play.opponent.new_hand(),
                        &self.play.card_atlas,
                        &self.play.sprite_sheet,
                    );

                    self.state_stack.pop();

                    // if self.play.rules.open {
                    //     self.state_stack.push(State::OpenCards);
                    //     self.state_stack.pop();
                    // }
                    if self.play.rules.random {
                        self.play.blue_hand = Hand::from_ids(
                            Suit::Blue,
                            true,
                            &self.play.opponent.new_hand(),
                            // &(0..110).choose_multiple(&mut rand::thread_rng(), 5),
                            &self.play.card_atlas,
                            &self.play.sprite_sheet,
                        );
                        self.state_stack.pop();
                        return Some(Event::PlaySound(Sfx::Select));
                    }

                    self.play.wait_for_pick();
                    self.state_stack.push(State::CardPick);
                    return Some(Event::PlaySound(Sfx::Select));
                }
                Event::Quit => {
                    self.state_stack.clear();
                    return Some(Event::PlaySound(Sfx::Cancel));
                }
                Event::ChangeRule(r) => {
                    self.play.toggle_rule(r);
                    return Some(Event::PlaySound(Sfx::Select));
                }
                Event::ChangeDifficulty => {
                    let curr = self.play.opponent.difficulty();
                    let d = if curr >= 3 { 1 } else { curr + 1 };
                    self.play.opponent.set_difficulty(d);
                    return Some(Event::PlaySound(Sfx::Select));
                }
                Event::ToggleCards(n) => {
                    self.play.opponent.toogle_cards(n);

                    return Some(Event::PlaySound(Sfx::Select));
                }
                Event::None => {},
                _ => {
                    unreachable!();
                }
            };
        }

        None
    }

    pub fn draw(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        array: &mut InstanceArray,
        elem_array: &mut InstanceArray,
    ) {
        for state in &self.state_stack {
            match state {
                State::Menu => self.menu.draw(ctx, array, canvas),
                State::CardPick => self.card_pick.draw(ctx, canvas, array),
                State::CoinFlip => {}
                State::Play => self.play.draw(ctx, canvas, array, elem_array),
                State::Fin => self.fin.draw(canvas, array),
            }
        }

        //self.draw_state_stack(canvas);
    }

    // fn draw_state_stack(&self, canvas: &mut Canvas) {
    //     let text_raw = self
    //         .state_stack
    //         .iter()
    //         .rev()
    //         .map(|state| format!("{state:?}\n"))
    //         .collect::<String>();

    //     let text = Text::new(text_raw);
    //     let text_pos = [100., 520.];
    //     canvas.draw(&text, DrawParam::default().dest(text_pos));
    // }
}
