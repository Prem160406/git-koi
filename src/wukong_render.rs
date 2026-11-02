//! Pixel-art Wukong renderer. The character atlas is derived from the approved concept
//! sheet; it is a first static pose per evolution stage, not a finished animation sheet.

use std::io::Cursor;
use std::sync::OnceLock;

use crate::behavior::Behavior;
use crate::koi::{KoiDevelopment, RecentStreak};
use crate::render::{FRAME_HEIGHT_PIXELS, FRAME_WIDTH_PIXELS, KoiFrame};
use crate::wukong::EvolutionStage;

const ATLAS_PNG: &[u8] = include_bytes!("../assets/wukong/atlas.png");

const BACKGROUND: [u8; 3] = [22, 17, 29];
const CLOUD_DARK: [u8; 3] = [45, 31, 52];
const CLOUD_MID: [u8; 3] = [58, 39, 66];
const CLOUD_LIGHT: [u8; 3] = [72, 48, 78];
const GOLD_DARK: [u8; 3] = [150, 73, 28];
const GOLD: [u8; 3] = [232, 139, 48];
const GOLD_LIGHT: [u8; 3] = [255, 207, 112];
const LEAF_RUST: [u8; 3] = [194, 83, 39];
const LEAF_AMBER: [u8; 3] = [229, 151, 52];
const LEAF_OLIVE: [u8; 3] = [173, 151, 57];

#[derive(Clone, Copy)]
struct SpriteRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

const SPRITES: [SpriteRect; 5] = [
    SpriteRect { x: 0, y: 75, width: 70, height: 89 },
    SpriteRect { x: 70, y: 71, width: 74, height: 93 },
    SpriteRect { x: 144, y: 47, width: 91, height: 117 },
    SpriteRect { x: 235, y: 19, width: 150, height: 145 },
    SpriteRect { x: 385, y: 0, width: 184, height: 164 },
];

struct Atlas {
    width: u32,
    pixels: Vec<[u8; 4]>,
}

static ATLAS: OnceLock<Atlas> = OnceLock::new();

/// The four daily animation moods used by Wukong.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WukongBehavior {
    /// Calm breathing and slow floating.
    Meditating,
    /// A small bounce and staff-practice motion.
    Training,
    /// A short lunge with a golden energy sweep.
    Fighting,
    /// A buoyant cloud ride with drifting particles.
    CloudSurfing,
}

impl WukongBehavior {
    /// Translate the existing activity classifier into Wukong's behaviour vocabulary.
    pub fn from_development(development: &KoiDevelopment) -> Self {
        match crate::behavior::select_behavior(development) {
            Behavior::DeepRest | Behavior::Rest => Self::Meditating,
            Behavior::SurfaceFeeding => Self::Training,
            Behavior::Dart => Self::Fighting,
            Behavior::Cruise => {
                let current_streak_weeks = match &development.streaks.recent {
                    Some(RecentStreak::Current(weeks)) => weeks.len(),
                    Some(RecentStreak::Ended { .. }) | None => 0,
                };
                if current_streak_weeks >= 6 {
                    Self::CloudSurfing
                } else {
                    Self::Training
                }
            }
        }
    }

    pub fn frame_count(self) -> u32 {
        match self {
            Self::Meditating => 72,
            Self::Training => 48,
            Self::Fighting => 36,
            Self::CloudSurfing => 60,
        }
    }
}

/// Render a still Wukong portrait using the approved stage sprite.
pub fn render_wukong(stage: EvolutionStage) -> KoiFrame {
    render_wukong_frame(stage, WukongBehavior::Meditating, 0)
}

/// Render one frame of Wukong's behaviour loop. The source character art is currently
/// one pose per stage; this renderer animates position, aura, clouds, leaves, and effects.
/// Dedicated frame-by-frame body animation will replace these procedural motions later.
pub fn render_wukong_frame(
    stage: EvolutionStage,
    behavior: WukongBehavior,
    frame_index: u32,
) -> KoiFrame {
    let mut pixels = vec![[BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255]; (FRAME_WIDTH_PIXELS * FRAME_HEIGHT_PIXELS) as usize];
    let phase = frame_index as f64 / behavior.frame_count() as f64 * std::f64::consts::TAU;
    draw_background_clouds(&mut pixels, phase);
    draw_aura(&mut pixels, stage, behavior, phase);
    let bob = match behavior {
        WukongBehavior::Meditating => (phase.sin() * 1.5).round() as i32,
        WukongBehavior::Training => (phase.sin() * 3.0).round() as i32,
        WukongBehavior::Fighting => (phase.sin() * 1.0).round() as i32,
        WukongBehavior::CloudSurfing => (phase.sin() * 4.0).round() as i32,
    };
    let lunge = if behavior == WukongBehavior::Fighting {
        (phase.sin() * 3.0).round() as i32
    } else {
        0
    };

    draw_leaves(&mut pixels, phase, behavior);
    if behavior == WukongBehavior::CloudSurfing {
        draw_cloud_surf_effect(&mut pixels, phase);
    }
    draw_character(&mut pixels, stage, bob, lunge);

    if behavior == WukongBehavior::Fighting {
        draw_energy_sweep(&mut pixels, phase);
    }
    KoiFrame::from_rgba_pixels(pixels)
}

fn atlas() -> &'static Atlas {
    ATLAS.get_or_init(|| {
        let mut decoder = png::Decoder::new(Cursor::new(ATLAS_PNG));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().expect("embedded Wukong atlas is valid PNG");
        let mut buffer = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buffer).expect("embedded Wukong atlas decodes");
        let bytes = &buffer[..info.buffer_size()];
        let pixels = match info.color_type {
            png::ColorType::Rgba => bytes
                .chunks_exact(4)
                .map(|p| [p[0], p[1], p[2], p[3]])
                .collect(),
            png::ColorType::Rgb => bytes
                .chunks_exact(3)
                .map(|p| [p[0], p[1], p[2], 255])
                .collect(),
            png::ColorType::GrayscaleAlpha => bytes
                .chunks_exact(2)
                .map(|p| [p[0], p[0], p[0], p[1]])
                .collect(),
            png::ColorType::Grayscale => bytes
                .iter()
                .map(|&v| [v, v, v, 255])
                .collect(),
            png::ColorType::Indexed => unreachable!("EXPAND converts indexed PNGs"),
        };
        Atlas { width: info.width, pixels }
    })
}

fn draw_character(pixels: &mut [[u8; 4]], stage: EvolutionStage, bob: i32, lunge: i32) {
    let rect = SPRITES[stage.index() as usize];
    let atlas = atlas();
    let max_width = 226.0;
    let max_height = 224.0;
    let scale = (max_width / rect.width as f64)
        .min(max_height / rect.height as f64)
        .min(2.0);
    let draw_width = (rect.width as f64 * scale).round().max(1.0) as u32;
    let draw_height = (rect.height as f64 * scale).round().max(1.0) as u32;
    let left = (FRAME_WIDTH_PIXELS as i32 - draw_width as i32) / 2 + lunge;
    let top = FRAME_HEIGHT_PIXELS as i32 - 18 - draw_height as i32 + bob;
    for y in 0..draw_height {
        for x in 0..draw_width {
            let source_x = rect.x + (x as f64 / scale).floor() as u32;
            let source_y = rect.y + (y as f64 / scale).floor() as u32;
            if source_x >= atlas.width {
                continue;
            }
            let source = atlas.pixels[(source_y * atlas.width + source_x) as usize];
            if source[3] == 0 {
                continue;
            }
            let dest_x = left + x as i32;
            let dest_y = top + y as i32;
            if dest_x >= 0 && dest_y >= 0
                && dest_x < FRAME_WIDTH_PIXELS as i32
                && dest_y < FRAME_HEIGHT_PIXELS as i32
            {
                pixels[(dest_y as u32 * FRAME_WIDTH_PIXELS + dest_x as u32) as usize] = source;
            }
        }
    }
}

fn draw_background_clouds(pixels: &mut [[u8; 4]], phase: f64) {
    let drift = (phase.sin() * 8.0).round() as i32;
    draw_cloud(pixels, -16 + drift, 42, 1.0);
    draw_cloud(pixels, 158 - drift / 2, 25, 0.72);
    draw_cloud(pixels, 178 + drift / 3, 106, 0.55);
    draw_cloud(pixels, -32 - drift / 3, 155, 0.6);
}

fn draw_cloud(pixels: &mut [[u8; 4]], x: i32, y: i32, scale: f64) {
    let s = scale.max(0.5);
    let puffs = [
        (-13, 3, 11, 7),
        (-4, -3, 13, 10),
        (8, 0, 12, 8),
        (18, 4, 9, 5),
    ];
    for (px, py, rx, ry) in puffs {
        fill_ellipse(pixels, x + (px as f64 * s) as i32, y + (py as f64 * s) as i32,
            (rx as f64 * s) as i32, (ry as f64 * s) as i32, CLOUD_DARK);
    }
    fill_rect(pixels, x - (17.0 * s) as i32, y + (5.0 * s) as i32,
        (39.0 * s) as i32, (4.0 * s) as i32, CLOUD_DARK);
    fill_ellipse(pixels, x - (4.0 * s) as i32, y - (2.0 * s) as i32,
        (9.0 * s) as i32, (6.0 * s) as i32, CLOUD_MID);
    fill_ellipse(pixels, x + (12.0 * s) as i32, y + (2.0 * s) as i32,
        (7.0 * s) as i32, (4.0 * s) as i32, CLOUD_LIGHT);
}

fn draw_aura(pixels: &mut [[u8; 4]], stage: EvolutionStage, behavior: WukongBehavior, phase: f64) {
    let level = stage.index() as i32;
    let pulse = (phase.sin() * 2.0).round() as i32;
    let radius_x = 35 + level * 7 + pulse;
    let radius_y = 54 + level * 8 + pulse;
    let center_x = 128;
    let center_y = 122;
    draw_ring(pixels, center_x, center_y, radius_x, radius_y, GOLD_DARK);
    if level >= 2 || behavior == WukongBehavior::Fighting {
        draw_ring(pixels, center_x, center_y, radius_x - 3, radius_y - 5, GOLD);
    }
    if level >= 3 {
        draw_ring(pixels, center_x, center_y, radius_x - 7, radius_y - 9, GOLD_LIGHT);
    }
}

fn draw_ring(pixels: &mut [[u8; 4]], cx: i32, cy: i32, rx: i32, ry: i32, color: [u8; 3]) {
    for degree in 0..360 {
        let radians = degree as f64 * std::f64::consts::PI / 180.0;
        let x = cx + (radians.cos() * rx as f64).round() as i32;
        let y = cy + (radians.sin() * ry as f64).round() as i32;
        put_pixel(pixels, x, y, color);
    }
}

fn draw_leaves(pixels: &mut [[u8; 4]], phase: f64, behavior: WukongBehavior) {
    let speed = match behavior {
        WukongBehavior::Meditating => 1,
        WukongBehavior::Training => 2,
        WukongBehavior::Fighting => 4,
        WukongBehavior::CloudSurfing => 3,
    };
    for i in 0..7 {
        let angle = phase * speed as f64 + i as f64 * std::f64::consts::TAU / 7.0;
        let x = 16 + i * 34 + (angle.cos() * 9.0).round() as i32;
        let y = 28 + i * 27 + (angle.sin() * 7.0).round() as i32;
        let color = match i % 3 {
            0 => LEAF_RUST,
            1 => LEAF_AMBER,
            _ => LEAF_OLIVE,
        };
        put_pixel(pixels, x, y, color);
        put_pixel(pixels, x + 1, y - 1, color);
        put_pixel(pixels, x + 2, y, color);
        put_pixel(pixels, x + 1, y + 1, GOLD_DARK);
    }
}

fn draw_energy_sweep(pixels: &mut [[u8; 4]], phase: f64) {
    let radius = 32;
    let start = 205.0 + phase.sin() * 18.0;
    for offset in 0..125 {
        let degree = start + offset as f64;
        let radians = degree * std::f64::consts::PI / 180.0;
        let x = 128 + (radians.cos() * radius as f64).round() as i32;
        let y = 138 + (radians.sin() * radius as f64 * 0.75).round() as i32;
        put_pixel(pixels, x, y, if offset % 3 == 0 { GOLD_LIGHT } else { GOLD });
    }
}

fn draw_cloud_surf_effect(pixels: &mut [[u8; 4]], phase: f64) {
    let bob = (phase.sin() * 1.5).round() as i32;
    draw_cloud(pixels, 128, 228 + bob, 1.15);
    draw_cloud(pixels, 98, 234 + bob, 0.7);
    draw_cloud(pixels, 157, 233 + bob, 0.7);
}

fn fill_ellipse(pixels: &mut [[u8; 4]], cx: i32, cy: i32, rx: i32, ry: i32, color: [u8; 3]) {
    if rx <= 0 || ry <= 0 { return; }
    for y in -ry..=ry {
        for x in -rx..=rx {
            if x * x * ry * ry + y * y * rx * rx <= rx * rx * ry * ry {
                put_pixel(pixels, cx + x, cy + y, color);
            }
        }
    }
}

fn fill_rect(pixels: &mut [[u8; 4]], x: i32, y: i32, width: i32, height: i32, color: [u8; 3]) {
    for dy in 0..height.max(0) {
        for dx in 0..width.max(0) {
            put_pixel(pixels, x + dx, y + dy, color);
        }
    }
}

fn put_pixel(pixels: &mut [[u8; 4]], x: i32, y: i32, color: [u8; 3]) {
    if x >= 0 && y >= 0 && x < FRAME_WIDTH_PIXELS as i32 && y < FRAME_HEIGHT_PIXELS as i32 {
        pixels[(y as u32 * FRAME_WIDTH_PIXELS + x as u32) as usize] =
            [color[0], color[1], color[2], 255];
    }
}
