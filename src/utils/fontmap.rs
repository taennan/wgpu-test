use crate::utils::paths;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use glam::U8Vec2;
use image::{Rgba, RgbaImage};
use std::{collections::HashMap, fs};

#[rustfmt::skip]
const CHARSET: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
    'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
    'Y', 'Z', '!', '№', ';', '%', ':', '?', '*', '(', ')', '_',
    '+', '-', '=', '.', ',', '/', '|', '"', '\'', '@', '#', '$',
    '^', '&', '{', '}', '[', ']', '>', '<', '\\', '`', '~', ' ',
];
const FONTMAP_COLS: u32 = 12;
const PX_SIZE: f32 = 64.0;

pub fn chars_to_fontmap_positions(string: &str) -> Vec<U8Vec2> {
    let char_positions_map: HashMap<char, U8Vec2> = CHARSET
        .iter()
        .enumerate()
        .map(|(i, c)| {
            (
                *c,
                U8Vec2::new(
                    (i as u32 % FONTMAP_COLS) as u8,
                    (i as u32 / FONTMAP_COLS) as u8,
                ),
            )
        })
        .collect();
    let positions = string
        .chars()
        .map(|c| {
            char_positions_map
                .get(&c)
                .unwrap_or(&(U8Vec2::ONE * 12))
                .clone()
        })
        .collect::<Vec<_>>();
    positions
}

pub fn save_fontmap(fontname: &str) {
    let font_bytes = fs::read(paths::font(fontname)).expect("Failed to open font file");

    let font = FontRef::try_from_slice(&font_bytes).expect("invalid font");
    let scale = PxScale::from(PX_SIZE);
    let scaled_font = font.as_scaled(scale);

    let cell_w = scaled_font.h_advance(font.glyph_id(' ')).ceil() as u32;
    let ascent = scaled_font.ascent();
    let cell_h = (ascent - scaled_font.descent() + scaled_font.line_gap()).ceil() as u32;

    let rows = (CHARSET.len() as u32 + FONTMAP_COLS - 1) / FONTMAP_COLS;

    let mut atlas = RgbaImage::new(FONTMAP_COLS * cell_w, rows * cell_h);

    for (i, &ch) in CHARSET.iter().enumerate() {
        let col = i as u32 % FONTMAP_COLS;
        let row = i as u32 / FONTMAP_COLS;
        let cell_x = col * cell_w;
        let cell_y = row * cell_h;

        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(scale, point(0.0, 0.0));

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();

            outlined.draw(|x, y, coverage| {
                let px = cell_x as i32 + bounds.min.x as i32 + x as i32;
                let py = cell_y as i32 + ascent as i32 + bounds.min.y as i32 + y as i32;

                if px >= 0 && py >= 0 && (px as u32) < atlas.width() && (py as u32) < atlas.height()
                {
                    let value = (coverage * 255.0).round() as u8;
                    // max-blend in case bounds ever overlap adjacent glyph ink
                    let existing = atlas.get_pixel(px as u32, py as u32).0[0];
                    atlas.put_pixel(
                        px as u32,
                        py as u32,
                        Rgba([u8::MAX, u8::MAX, u8::MAX, value.max(existing)]),
                    );
                }
            });
        }
        // characters with no outline (space, control chars) just leave the cell blank
    }

    let mut save_path = paths::texture(fontname);
    save_path.set_extension("png");
    atlas.save(save_path).expect("Failed to save font bitmap");
}
