use crate::core::Rules;
use crate::utils::{Event, Rule};
use ggez::event::MouseButton;
use ggez::graphics::{Canvas, Color, InstanceArray, PxScale, Rect, Text, TextFragment};
use ggez::Context;

use crate::consts;
use crate::graphics::TextBox;

const MENU_BG_POS: [f32; 2] = [286.0, 116.0];
const MENU_BG_DIMENSIONS: [f32; 2] = [240.0, 312.0];
const PLAY_BUTTON_POS: [f32; 2] = [370.0, 336.0];
const QUIT_BUTTON_POS: [f32; 2] = [370.0, 377.0];

pub struct MenuItem {
    pub label: String,
    pub disabled: bool,
    pub rect: Rect,
    pub callback: Event,
}
impl MenuItem {
    pub fn draw(&self, canvas: &mut Canvas) {
        let color = if self.disabled {
            Color::from_rgb(127, 127, 127)
        } else {
            Color::from_rgb(255, 255, 255)
        };

        let label = Text::new(TextFragment {
            text: self.label.clone(),
            color: Some(color),
            font: Some("pixel font".into()),
            scale: Some(PxScale::from(consts::FONT_SIZE)),
        });
        
        let shadow = Text::new(TextFragment {
            text: self.label.clone(),
            color: Some(Color::from_rgb(50, 50, 50)),
            font: Some("pixel font".into()),
            scale: Some(PxScale::from(consts::FONT_SIZE)),
        });
        canvas.draw(&shadow, [self.rect.x + 2.0, self.rect.y + 2.0]);
        canvas.draw(&label, [self.rect.x, self.rect.y]);
    }
}

pub struct Menu {
    bg_rect: TextBox,
    items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(ctx: &mut Context) -> Self {
        let mut items: Vec<MenuItem> = Vec::with_capacity(10);

        let rules_label = MenuItem {
            label: "Rules:".into(),
            disabled: false,
            rect: Rect::new(MENU_BG_POS[0] + 10.0, 15.0, 0.0, 0.0),
            callback: Event::None,
        };
        let labels = [
            "Open",
            "Elemental",
            "Random",
            "Same",
            "Wall",
            "Plus",
            "Sudden Death",
        ];
        items.push(rules_label);

        for (i, (label, rule)) in labels.iter().zip(Rule::iterator()).enumerate() {
            let i_small = u8::try_from(i).expect("Value is too big!");

            let item = MenuItem {
                label: String::from(*label),
                disabled: true,
                rect: Rect {
                    x: MENU_BG_POS[0] + 25.0,
                    y: 20.0 + consts::FONT_SIZE + f32::from(i_small) * (consts::FONT_SIZE + 5.0),
                    w: label.len() as f32 * 10.0,
                    h: consts::FONT_SIZE,
                },
                callback: Event::ChangeRule(*rule),
            };
            items.push(item);
        }
        let mut items_size: f32 = f32::from(u8::try_from(items.len() - 1).expect("vec is too big"));

        let difficulty_label = MenuItem {
            label: "Difficulty:".into(),
            disabled: false,
            rect: Rect::new(
                MENU_BG_POS[0] + 10.0,
                20.0 + consts::FONT_SIZE + items_size * (consts::FONT_SIZE + 5.0),
                "Difficulty:x".len() as f32 * 10.0,
                consts::FONT_SIZE,
            ),
            callback: Event::ChangeDifficulty,
        };

        items.push(difficulty_label);

        items_size = f32::from(u8::try_from(items.len() - 1).expect("vec is too big"));

        let cards_label = MenuItem {
            label: "Cards aviable:".into(),
            disabled: false,
            rect: Rect::new(
                MENU_BG_POS[0] + 10.0,
                20.0 + consts::FONT_SIZE + items_size * (consts::FONT_SIZE + 5.0),
                60.0,
                consts::FONT_SIZE,
            ),
            callback: Event::None,
        };

        items.push(cards_label);

        items_size = f32::from(u8::try_from(items.len() - 1).expect("vec is too big"));

        for i in 1..11 {
            let i_small = u8::try_from(i - 1).expect("Value is too big!");
            let item = MenuItem {
                label: format!("{i}"),
                disabled: false,
                rect: Rect::new(
                    MENU_BG_POS[0] + 25.0 + 20.0 * f32::from(i_small),
                    20.0 + consts::FONT_SIZE + items_size * (consts::FONT_SIZE + 5.0),
                    20.0,
                    consts::FONT_SIZE,
                ),
                callback: Event::ToggleCards(i - 1),
            };

            items.push(item);
        }

        items_size += 1.0;

        let play_button = MenuItem {
            label: "Play".into(),
            disabled: false,
            rect: Rect::new(
                PLAY_BUTTON_POS[0],
                20.0 + consts::FONT_SIZE + items_size * (consts::FONT_SIZE + 5.0),
                60.0,
                consts::FONT_SIZE,
            ),
            callback: Event::Play,
        };

        let quit_button = MenuItem {
            label: "Quit".into(),
            disabled: false,
            rect: Rect::new(
                QUIT_BUTTON_POS[0],
                20.0 + consts::FONT_SIZE + (items_size + 1.0) * (consts::FONT_SIZE + 5.0),
                60.0,
                consts::FONT_SIZE,
            ),
            callback: Event::Quit,
        };

        items_size += 2.0;

        let box_height = items_size * (consts::FONT_SIZE + 5.0) + 50.0;

        let box_x_pos = (consts::WINDOW_DIMENSIONS[0] - MENU_BG_DIMENSIONS[0]) / 2.0;
        let box_y_pos = (consts::WINDOW_DIMENSIONS[1] - box_height) / 2.0;

        //let items = vec![play_button, quit_button];
        items.push(play_button);
        items.push(quit_button);

        for item in items.iter_mut() {
            item.rect.y += box_y_pos;
        }

        Self {
            bg_rect: TextBox::new(
                ctx,
                [box_x_pos, box_y_pos],
                [MENU_BG_DIMENSIONS[0], box_height],
                (64, 64, 64),
            ),
            items,
        }
    }
    pub fn init(&mut self, rules: &Rules) {
        self.update_rules(rules);
    }
    fn update_difficulty(&mut self, n: usize) {
        self.items[8].label = format!("Difficulty: {n}");
    }

    fn update_rules(&mut self, rules: &Rules) {
        self.items[1].disabled = !rules.open;
        self.items[2].disabled = !rules.elemental;
        self.items[3].disabled = !rules.random;
        self.items[4].disabled = !rules.same;
        self.items[5].disabled = !rules.same_wall;
        self.items[6].disabled = !rules.plus;
        self.items[7].disabled = !rules.sudden_death;
    }
    fn update_cards_aviable(&mut self, cards_aviable: &[bool; 10]){
        for (i, val) in cards_aviable.iter().enumerate(){
            self.items[i+ 10].disabled = !val;
        }

    }

    pub fn update(&mut self, difficulty: usize, rules: &Rules, cards_aviable: &[bool; 10], ctx: &mut Context) -> Option<Event> {
        //let _rect = Rect::new(consts::BOARD_OFFSET[0], consts::BOARD_OFFSET[1], 240., 300.);
        self.update_rules(rules);
        self.update_difficulty(difficulty);
        self.update_cards_aviable(cards_aviable);
        if ctx.mouse.button_just_pressed(MouseButton::Left) {
            for item in &self.items {
                if item.rect.contains(ctx.mouse.position()) {
                    return Some(item.callback);
                }
            }
        }

        None
    }

    pub fn draw(&self, _ctx: &mut Context, _array: &mut InstanceArray, canvas: &mut Canvas) {
        // canvas.draw(
        //     &self.bg_rect,
        //     DrawParam::default()
        //         .scale(MENU_BG_DIMENSIONS)
        //         .dest(MENU_BG_POS),
        // );
        self.bg_rect.draw(canvas);
        for item in &self.items {
            item.draw(canvas);
        }
    }
}
