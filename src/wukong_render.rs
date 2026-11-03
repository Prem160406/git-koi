//! Pixel-art Wukong renderer. The character atlas is extracted from the approved
//! evolution board; it is one static pose per stage, not a finished animation sheet.

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
const CLOUD_CREAM: [u8; 3] = [131, 91, 89];

#[derive(Clone, Copy)]
struct SpriteRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

const SPRITES: [SpriteRect; 5] = [
    SpriteRect {
        x: 8,
        y: 126,
        width: 191,
        height: 234,
    },
    SpriteRect {
        x: 211,
        y: 155,
        width: 167,
        height: 205,
    },
    SpriteRect {
        x: 390,
        y: 111,
        width: 192,
        height: 249,
    },
    SpriteRect {
        x: 594,
        y: 54,
        width: 301,
        height: 306,
    },
    SpriteRect {
        x: 907,
        y: 8,
        width: 371,
        height: 352,
    },
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

    /// Return the number of frames in this behaviour's loop.
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

/// Render a transparent stage portrait with the production sprite and golden aura.
///
/// This keeps the renderer's native 256 × 256 canvas and shared character baseline,
/// while leaving the surrounding background transparent for composition and review.
pub fn render_wukong_sprite(stage: EvolutionStage) -> KoiFrame {
    let mut pixels = vec![[0, 0, 0, 0]; (FRAME_WIDTH_PIXELS * FRAME_HEIGHT_PIXELS) as usize];
    draw_aura(&mut pixels, stage, WukongBehavior::Meditating, 0.0);
    draw_character(&mut pixels, stage, 0, 0);
    KoiFrame::from_rgba_pixels(pixels)
}

/// Render only the transparent, animated aura and sparks for visual review.
pub fn render_wukong_aura_frame(
    stage: EvolutionStage,
    behavior: WukongBehavior,
    frame_index: u32,
) -> KoiFrame {
    let mut pixels = vec![[0, 0, 0, 0]; (FRAME_WIDTH_PIXELS * FRAME_HEIGHT_PIXELS) as usize];
    let phase = loop_phase(behavior, frame_index);
    draw_aura(&mut pixels, stage, behavior, phase);
    draw_sparks(&mut pixels, stage, phase);
    KoiFrame::from_rgba_pixels(pixels)
}

/// Render Wukong's environment layers without the character sprite.
///
/// The opaque background, clouds, leaves, aura, and behaviour-specific effects use
/// the same drawing routines as [`render_wukong_frame`].
pub fn render_wukong_environment_frame(
    stage: EvolutionStage,
    behavior: WukongBehavior,
    frame_index: u32,
) -> KoiFrame {
    let mut pixels = vec![
        [BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255];
        (FRAME_WIDTH_PIXELS * FRAME_HEIGHT_PIXELS) as usize
    ];
    let phase = loop_phase(behavior, frame_index);
    draw_background_clouds(&mut pixels, phase);
    draw_aura(&mut pixels, stage, behavior, phase);
    draw_leaves(&mut pixels, phase, behavior);
    if behavior == WukongBehavior::CloudSurfing {
        draw_cloud_surf_effect(&mut pixels, phase);
    }
    if behavior == WukongBehavior::Fighting {
        draw_energy_sweep(&mut pixels, phase);
    }
    KoiFrame::from_rgba_pixels(pixels)
}

/// Render one frame of Wukong's behaviour loop. The source character art is currently
/// one pose per stage; this renderer animates position, aura, clouds, leaves, and effects.
/// Dedicated frame-by-frame body animation will replace these procedural motions later.
pub fn render_wukong_frame(
    stage: EvolutionStage,
    behavior: WukongBehavior,
    frame_index: u32,
) -> KoiFrame {
    let mut pixels = vec![
        [BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255];
        (FRAME_WIDTH_PIXELS * FRAME_HEIGHT_PIXELS) as usize
    ];
    let phase = loop_phase(behavior, frame_index);
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
    draw_aura_front(&mut pixels, stage, phase);
    draw_sparks(&mut pixels, stage, phase);
    if behavior == WukongBehavior::Fighting {
        draw_energy_sweep(&mut pixels, phase);
    }
    KoiFrame::from_rgba_pixels(pixels)
}

fn loop_phase(behavior: WukongBehavior, frame_index: u32) -> f64 {
    let frame = frame_index % behavior.frame_count();
    frame as f64 / behavior.frame_count() as f64 * std::f64::consts::TAU
}

fn atlas() -> &'static Atlas {
    ATLAS.get_or_init(|| {
        let mut decoder = png::Decoder::new(Cursor::new(ATLAS_PNG));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().expect("embedded Wukong atlas is valid PNG");
        let buffer_size = reader
            .output_buffer_size()
            .expect("embedded Wukong atlas has a known output buffer size");
        let mut buffer = vec![0; buffer_size];
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
    // Deliberately increase the target height at every permanent evolution stage.
    // The final form is allowed to use nearly the full canvas width so it feels imposing.
    const TARGET_HEIGHTS: [f64; 5] = [120.0, 144.0, 164.0, 194.0, 224.0];
    let scale = TARGET_HEIGHTS[stage.index() as usize] / rect.height as f64;
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
    let drift = (phase.sin() * 5.0).round() as i32;
    draw_swirl_cloud(pixels, -7 + drift, 41, 1.15, CLOUD_DARK, CLOUD_MID, phase);
    draw_swirl_cloud(pixels, 189 - drift / 2, 33, 0.78, CLOUD_DARK, CLOUD_CREAM, -phase * 0.7);
    draw_swirl_cloud(pixels, 204 + drift / 3, 130, 0.75, CLOUD_DARK, CLOUD_MID, phase * 0.55);
    draw_swirl_cloud(pixels, -9 - drift / 3, 170, 0.9, CLOUD_DARK, CLOUD_CREAM, -phase * 0.45);
}

fn draw_swirl_cloud(pixels: &mut [[u8; 4]], x: i32, y: i32, scale: f64, base: [u8; 3], highlight: [u8; 3], phase: f64) {
    // Stepped cloud lobes and curled tails give the distant layers a hand-built pixel silhouette.
    for (ox, oy, rx, ry) in [(-22, 5, 13, 7), (-11, 0, 15, 11), (3, -7, 13, 12), (16, -1, 14, 9), (29, 5, 11, 6)] {
        let wobble = (phase.sin() * 1.2).round() as i32;
        fill_ellipse(pixels, x + ((ox as f64 * scale) as i32), y + ((oy as f64 * scale) as i32) + wobble,
            (rx as f64 * scale) as i32, (ry as f64 * scale) as i32, base);
    }
    fill_rect(pixels, x - (25.0 * scale) as i32, y + (5.0 * scale) as i32,
        (55.0 * scale) as i32, (5.0 * scale) as i32, base);
    fill_ellipse(pixels, x - (10.0 * scale) as i32, y - (7.0 * scale) as i32,
        (8.0 * scale) as i32, (5.0 * scale) as i32, CLOUD_MID);
    fill_ellipse(pixels, x + (15.0 * scale) as i32, y - (3.0 * scale) as i32,
        (5.0 * scale) as i32, (3.0 * scale) as i32, CLOUD_CREAM);
    for curl in 0..28 {
        let t = curl as f64 / 27.0;
        let angle = t * std::f64::consts::PI * 2.15 + phase * 0.12;
        let radius = (2.0 + 8.0 * t) * scale;
        let px = x + (-5.0 * scale) as i32 + (angle.cos() * radius).round() as i32;
        let py = y - (6.0 * scale) as i32 + (angle.sin() * radius * 0.72).round() as i32;
        put_pixel(pixels, px, py, highlight);
        if curl % 4 == 0 { put_pixel(pixels, px + 1, py, highlight); }
    }
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
        fill_ellipse(
            pixels,
            x + (px as f64 * s) as i32,
            y + (py as f64 * s) as i32,
            (rx as f64 * s) as i32,
            (ry as f64 * s) as i32,
            CLOUD_DARK,
        );
    }
    fill_rect(
        pixels,
        x - (17.0 * s) as i32,
        y + (5.0 * s) as i32,
        (39.0 * s) as i32,
        (4.0 * s) as i32,
        CLOUD_DARK,
    );
    fill_ellipse(
        pixels,
        x - (4.0 * s) as i32,
        y - (2.0 * s) as i32,
        (9.0 * s) as i32,
        (6.0 * s) as i32,
        CLOUD_MID,
    );
    fill_ellipse(
        pixels,
        x + (12.0 * s) as i32,
        y + (2.0 * s) as i32,
        (7.0 * s) as i32,
        (4.0 * s) as i32,
        CLOUD_LIGHT,
    );
    // A tiny curled highlight gives the plum clouds their traditional swirl motif.
    for step in 0..18 {
        let angle = step as f64 / 17.0 * std::f64::consts::PI * 1.6;
        let radius = (0.7 + step as f64 * 0.38) * s;
        let spiral_x = x - (4.0 * s) as i32 + (angle.cos() * radius).round() as i32;
        let spiral_y = y - (2.0 * s) as i32 + (angle.sin() * radius * 0.65).round() as i32;
        put_pixel(pixels, spiral_x, spiral_y, CLOUD_LIGHT);
    }
}

fn draw_aura(
    pixels: &mut [[u8; 4]],
    stage: EvolutionStage,
    behavior: WukongBehavior,
    phase: f64,
) {
    let level = stage.index() as i32;
    let strength = [1, 2, 3, 4, 5][level as usize];
    let motion = phase * match behavior { WukongBehavior::Meditating => 0.45, WukongBehavior::Training => 0.85, WukongBehavior::Fighting => 1.5, WukongBehavior::CloudSurfing => 0.7 };
    let paths = [
        (3.75, 0.48, 1.0, 32.0, 1), (4.25, 0.62, -2.2, 32.0, 1),
        (4.8, 0.72, 3.3, 44.0, 2), (5.35, 0.62, -1.6, 44.0, 2),
        (5.85, 0.55, 4.4, 36.0, 2), (0.05, 0.58, -4.5, 52.0, 2),
    ];
    for (index, (start, span, bend, length, thickness)) in paths.into_iter().enumerate().take(strength + 1) {
        let a0 = start + motion * (0.26 + index as f64 * 0.11);
        let extent = span + 0.06 * (motion * 2.0 + index as f64).sin();
        for step in 0..(length as usize) {
            let t = step as f64 / length;
            let angle = a0 + t * extent;
            let rx = 34.0 + level as f64 * 5.2 + bend * t;
            let ry = 42.0 + level as f64 * 5.0 + bend * t * 0.55;
            let flutter = (t * 7.0 + motion * 1.8 + index as f64).sin() * (3.0 + level as f64 * 0.8);
            let cx = 128.0 + bend * 2.2 + (index as f64 * 5.5).sin() * 6.0;
            let cy = 122.0 + (index as f64 * 2.3).cos() * 8.0;
            let x = (cx + angle.cos() * rx + flutter * angle.sin()).round() as i32;
            let y = (cy + angle.sin() * ry + flutter * angle.cos()).round() as i32;
            let color = if t < 0.2 || t > 0.78 { GOLD_LIGHT } else if index % 2 == 0 { GOLD } else { LEAF_AMBER };
            let actual_thickness = thickness + i32::from(level >= 3);
            for thick in 0..actual_thickness { put_pixel(pixels, x + thick, y - thick / 2, color); }
        }
        // Each arc grows a tapered, flame-like tip that flickers independently.
        let tip = a0 + extent;
        let tx = 128 + (tip.cos() * (34.0 + level as f64 * 5.2)).round() as i32;
        let ty = 132 + (tip.sin() * (42.0 + level as f64 * 5.0)).round() as i32;
        draw_wisp(pixels, tx, ty, tip + 0.7, 4 + level, index as f64 + motion);
    }
    if level >= 4 { // cloud-facing curls make the final stage's aura meet the platform
        draw_wisp(pixels, 66, 210, -0.25 + motion, 22, motion);
        draw_wisp(pixels, 192, 204, 2.85 + motion, 20, motion + 2.0);
    }
    if behavior == WukongBehavior::Fighting { draw_wisp(pixels, 179, 130, 0.2 + motion, 18, motion); }
}

fn draw_wisp(pixels: &mut [[u8; 4]], x: i32, y: i32, angle: f64, length: i32, phase: f64) {
    for step in 0..length {
        let t = step as f64 / length.max(1) as f64;
        let curl = (t * std::f64::consts::PI * 1.4 + phase).sin() * (2.0 + 3.0 * t);
        let px = x + (angle.cos() * step as f64 + curl * angle.sin()).round() as i32;
        let py = y + (angle.sin() * step as f64 - curl * angle.cos()).round() as i32;
        put_pixel(pixels, px, py, if step % 4 == 0 { GOLD_LIGHT } else { GOLD });
    }
}

fn draw_aura_front(pixels: &mut [[u8; 4]], stage: EvolutionStage, phase: f64) {
    if stage.index() >= 3 {
        draw_wisp(pixels, 88, 163, 1.2 + phase.sin() * 0.15, 11, phase);
        draw_wisp(pixels, 164, 174, 2.0 + phase.cos() * 0.18, 8, phase + 1.0);
    }
}

fn draw_sparks(pixels: &mut [[u8; 4]], stage: EvolutionStage, phase: f64) {
    let count = 3 + stage.index() as usize * 3;
    for i in 0..count {
        let seed = i as f64 * 2.399 + stage.index() as f64 * 0.73;
        let drift = (phase * (0.28 + (i % 3) as f64 * 0.09) + seed).sin();
        let radius = 43.0 + ((i * 17) % 29) as f64;
        let x = 128 + (seed.cos() * radius + drift * 5.0).round() as i32;
        let y = 133 + (seed.sin() * radius * 0.83 + phase.cos() * 2.0).round() as i32;
        if (phase * 2.0 + seed).sin() > -0.45 {
            let color = if i % 3 == 0 { GOLD_LIGHT } else if i % 3 == 1 { GOLD } else { LEAF_RUST };
            put_pixel(pixels, x, y, color);
            if i % 2 == 0 { put_pixel(pixels, x + 1, y, color); put_pixel(pixels, x, y - 1, color); }
        }
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
    if rx <= 0 || ry <= 0 {
        return;
    }
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
