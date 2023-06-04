use ggez::{
    audio::{SoundSource, Source},
    graphics::{Canvas, Image, InstanceArray},
    Context, GameResult,
};

use std::rc::Rc;

use crate::{
    core::CardAtlas,
    graphics::sprite::Atlas,
    state::{Fade as FadeState, PlayState},
    utils::Sfx,
};
use crate::{graphics::Shape, utils::Event};

enum State {
    FadeIn,
    FadeOut,
    Play,
    InitPlay,
    ClearPlay,
}

pub struct WgpuShapes {
    left: Shape,
    right: Shape,
    enabled: [bool; 2],
}

impl WgpuShapes {
    pub fn new(ctx: &mut Context) -> Self {
        Self {
            left: Shape::new(ctx, 16f32),
            right: Shape::new(ctx, -16f32),
            enabled: [false; 2],
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.left.update(ctx);
        self.right.update(ctx);
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        if self.enabled[0] {
            self.left.draw(ctx);
        }
        if self.enabled[1] {
            self.right.draw(ctx);
        }
    }
}

pub struct SoundManager {
    pub bgm: Source,
    pub flip_card: Source,
    pub move_card: Source,
    pub select: Source,
    pub cancel: Source,
    pub victory: Source,
}

pub struct App {
    play_state: PlayState,
    fade_state: FadeState,
    state_stack: Vec<State>,
    array: InstanceArray,
    elem_array: InstanceArray,
    pub wgpu_shapes: WgpuShapes,
    sound_manager: SoundManager,
}

impl App {
    pub fn new(
        ctx: &mut Context,
        card_atlas: &Rc<CardAtlas>,
        sprite_sheet: &Rc<Atlas>,
        bg_image: &Rc<Image>,
        array: InstanceArray,
        elem_array: InstanceArray,
        mut sound_manager: SoundManager,
    ) -> Self {
        let play_state = PlayState::new(ctx, card_atlas, sprite_sheet, bg_image);
        let fade_state = FadeState::new(ctx);
        println!("{}", sound_manager.bgm.volume()); 
        sound_manager.bgm.set_volume(0.0); 
        sound_manager.victory.set_volume(0.0); 
        Self {
            play_state,
            fade_state,
            state_stack: vec![State::Play, State::InitPlay, State::FadeIn],
            array,
            elem_array,
            wgpu_shapes: WgpuShapes::new(ctx),
            sound_manager,
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        for state in &self.state_stack {
            match state {
                State::InitPlay | State::ClearPlay => {}
                State::Play => {
                    self.play_state
                        .draw(ctx, canvas, &mut self.array, &mut self.elem_array)
                }
                State::FadeIn | State::FadeOut => self.fade_state.draw(canvas),
            }
        }
    }
    pub fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ctx.time.delta().as_secs_f32();

        self.wgpu_shapes.update(ctx);
        self.wgpu_shapes.enabled = self.play_state.turn_marker_status();

        if let Some(state) = self.state_stack.last() {
            match state {
                State::InitPlay => {
                    self.play_state.init();
                    self.state_stack.pop();
                }
                State::ClearPlay => {
                    self.play_state.clear();
                    self.state_stack.pop();
                }
                State::Play => {
                    if let Some(e) = self.play_state.update(ctx) {
                        match e {
                            Event::PlaySound(Sfx::Fanfare) => {
                                self.sound_manager.bgm.stop(ctx)?;
                                self.sound_manager.victory.play(ctx)?
                            }
                            Event::PlaySound(Sfx::Flip) => {
                                self.sound_manager.flip_card.play_detached(ctx)?
                            }
                            Event::PlaySound(Sfx::Move) => {
                                self.sound_manager.move_card.play_detached(ctx)?
                            }
                            Event::PlaySound(Sfx::Select) => {
                                self.sound_manager.select.play_detached(ctx)?
                            }
                            Event::PlaySound(Sfx::Cancel) => {
                                self.sound_manager.cancel.play_detached(ctx)?
                            }

                            Event::Finished => {
                                self.state_stack.push(State::InitPlay);
                                self.state_stack.push(State::FadeIn);
                                self.state_stack.push(State::ClearPlay);
                                self.state_stack.push(State::FadeOut);
                                //self.state_stack.push(State::ClearPlay);
                            }
                            Event::Quit => ctx.request_quit(),
                            _ => {}
                        }
                    }
                }
                State::FadeOut => {
                    if !self.sound_manager.bgm.stopped() {
                        self.sound_manager.bgm.stop(ctx)?;
                    }

                    self.sound_manager.bgm.set_volume(self.sound_manager.bgm.volume() - dt / 2.0);
                    self.sound_manager.victory.set_volume(self.sound_manager.victory.volume() - dt / 2.0);
                    if let Some(e) = self.fade_state.fade_out_update(dt) {
                        match e {
                            Event::Finished => {
                                self.sound_manager.victory.stop(ctx)?;
                                self.state_stack.pop();
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                State::FadeIn => {
                    if self.sound_manager.bgm.stopped() {
                        self.sound_manager.bgm.play(ctx).expect("Audio error");
                    }
                    self.sound_manager.bgm.set_volume(self.sound_manager.bgm.volume() + dt / 2.0);
                    self.sound_manager.victory.set_volume(self.sound_manager.victory.volume() + dt / 2.0);
                    if let Some(e) = self.fade_state.fade_in_update(dt) {
                        match e {
                            Event::Finished => {
                                self.state_stack.pop();
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
