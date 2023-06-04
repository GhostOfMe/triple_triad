pub const WINDOW_DIMENSIONS: [f32; 2] = [800.0, 600.0];

pub const CARD_SIZE: [f32; 2] = [130.0, 165.0];
pub const SPRITE_SIZE: [f32; 2] = [64.0, 64.0];
pub const CARD_SCALE: [f32; 2] = [CARD_SIZE[0] / SPRITE_SIZE[1], CARD_SIZE[1] / SPRITE_SIZE[1]];
pub const SCALE_FACTOR: f32 = CARD_SIZE[1] / SPRITE_SIZE[1];

pub const TOP_MARGIN: f32 = 90.0;
pub const LEFT_MARGIN: f32 = 45.0;


pub const RIGHT_HAND_OFFSET: [f32; 2] = [
    WINDOW_DIMENSIONS[0] - CARD_SIZE[0] - LEFT_MARGIN,
    TOP_MARGIN,
];
pub const LEFT_HAND_OFFSET: [f32; 2] = [LEFT_MARGIN, TOP_MARGIN];
pub const HAND_STEP: f32 = 70.0;
pub const RIGHT_HAND_SELECTED_OFFSET: f32 = -25.0;

pub const COMBO_BANNER_DURATION: f32 = 1.0;
pub const ELEM_PROB: f32 = 0.3;

pub const RED_SCORE_POS: [f32; 2] = [86.0, 516.0];
pub const BLUE_SCORE_POS: [f32; 2] = [670.0, 516.0];

pub const FONT_SIZE: f32 = 25.0;

/*              Sprites               */

pub const CARD_BACK_SPRITE_ID: usize = 110;
pub const CARD_BLUE_SUIT_SPRITE_ID: usize = 112;
pub const CARD_RED_SUIT_SPRITE_ID: usize = 113;

pub const DIGIT_0_SPRITE_ID: usize = 114;
pub const DIGIT_1_SPRITE_ID: usize = 115;
pub const DIGIT_2_SPRITE_ID: usize = 116;
pub const DIGIT_3_SPRITE_ID: usize = 117;
pub const DIGIT_4_SPRITE_ID: usize = 118;
pub const DIGIT_5_SPRITE_ID: usize = 119;
pub const DIGIT_6_SPRITE_ID: usize = 120;
pub const DIGIT_7_SPRITE_ID: usize = 121;
pub const DIGIT_8_SPRITE_ID: usize = 122;
pub const DIGIT_9_SPRITE_ID: usize = 123;
pub const DIGIT_A_SPRITE_ID: usize = 124;

pub const ELEM_FIRE_SPRITE_ID: usize = 125;
pub const ELEM_ICE_SPRITE_ID: usize = 126;
pub const ELEM_POISON_SPRITE_ID: usize = 127;
pub const ELEM_WIND_SPRITE_ID: usize = 128;
pub const ELEM_TRHUNDER_SPRITE_ID: usize = 129;
pub const ELEM_EARTH_SPRITE_ID: usize = 130;
pub const ELEM_WATER_SPRITE_ID: usize = 131;
pub const ELEM_HOLY_SPRITE_ID: usize = 132;

pub const BIG_DIGIT_0_SPRITE_ID: usize = 133;
pub const BIG_DIGIT_1_SPRITE_ID: usize = 134;
pub const BIG_DIGIT_2_SPRITE_ID: usize = 135;
pub const BIG_DIGIT_3_SPRITE_ID: usize = 136;
pub const BIG_DIGIT_4_SPRITE_ID: usize = 137;
pub const BIG_DIGIT_5_SPRITE_ID: usize = 138;
pub const BIG_DIGIT_6_SPRITE_ID: usize = 139;
pub const BIG_DIGIT_7_SPRITE_ID: usize = 140;
pub const BIG_DIGIT_8_SPRITE_ID: usize = 141;
pub const BIG_DIGIT_9_SPRITE_ID: usize = 142;

pub const ELEM_MALUS_SPRITE_ID: usize = 143;
pub const ELEM_BONUS_SPRITE_ID: usize = 144;

pub const COMBO_PLUS_BANNER_SPRITE_ID: usize = 145;
pub const COMBO_SAME_BANNER_SPRITE_ID: usize = 146;
pub const COMBO_BANNER_SPRITE_ID: usize = 147;

pub const FINISH_MESSAGE_LOSE_SPRITE_ID: usize = 148;
pub const FINISH_MESSAGE_WON_SPRITE_ID: usize = 149;
pub const FINISH_MESSAGE_DRAW_SPRITE_ID: usize = 150;

//pub const CARD_ICON_SPRITE_ID: usize = 151;

pub const RIGHT_ARROW_SPRITE_ID: usize = 152;

pub const TOOLTIP_DELAY: f32 = 0.5;
