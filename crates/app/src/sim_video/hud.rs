//! Crisp 5x7 bitmap HUD overlays and depth-aware 3D toolpath ribbon rendering
//! for the built-in stock-simulation movie frames.
//!
//! Provenance: clean-room 5x7 ASCII bitmap glyphs and alpha-blended RGB24 compositing.

pub(super) struct OpBar {
    pub first_seg: usize,
    pub end_seg: usize,
    pub color: [u8; 3],
}

pub(super) struct HudInfo<'a> {
    pub setup_title: &'a str,
    pub op_title: &'a str,
    pub tool_badge: &'a str,
    pub rpm: f64,
    pub feed: f64,
    pub is_rapid: bool,
    pub show_tool: bool,
    pub wcs_pos: [f32; 3],
    pub remaining_cm3: f64,
    pub removed_pct: f64,
    pub progress_frac: f32,
    pub footer_left: &'a str,
    pub footer_right: &'a str,
    pub banner: &'a str,
    pub move_num: usize,
    pub bars: &'a [OpBar],
    pub total_segs: usize,
}

#[rustfmt::skip]
fn glyph_5x7(c: char) -> [u8; 7] {
    match c {
        '0' => [0x0E,0x11,0x13,0x15,0x19,0x11,0x0E], '1' => [0x04,0x0C,0x04,0x04,0x04,0x04,0x0E],
        '2' => [0x0E,0x11,0x01,0x02,0x04,0x08,0x1F], '3' => [0x1F,0x02,0x04,0x02,0x01,0x11,0x0E],
        '4' => [0x02,0x06,0x0A,0x12,0x1F,0x02,0x02], '5' => [0x1F,0x10,0x1E,0x01,0x01,0x11,0x0E],
        '6' => [0x06,0x08,0x10,0x1E,0x11,0x11,0x0E], '7' => [0x1F,0x01,0x02,0x04,0x08,0x08,0x08],
        '8' => [0x0E,0x11,0x11,0x0E,0x11,0x11,0x0E], '9' => [0x0E,0x11,0x11,0x0F,0x01,0x02,0x0C],
        'A' => [0x0E,0x11,0x11,0x1F,0x11,0x11,0x11], 'B' => [0x1E,0x11,0x11,0x1E,0x11,0x11,0x1E],
        'C' => [0x0E,0x11,0x10,0x10,0x10,0x11,0x0E], 'D' => [0x1C,0x12,0x11,0x11,0x11,0x12,0x1C],
        'E' => [0x1F,0x10,0x10,0x1E,0x10,0x10,0x1F], 'F' => [0x1F,0x10,0x10,0x1E,0x10,0x10,0x10],
        'G' => [0x0E,0x11,0x10,0x17,0x11,0x11,0x0F], 'H' => [0x11,0x11,0x11,0x1F,0x11,0x11,0x11],
        'I' => [0x0E,0x04,0x04,0x04,0x04,0x04,0x0E], 'J' => [0x07,0x02,0x02,0x02,0x02,0x12,0x0C],
        'K' => [0x11,0x12,0x14,0x18,0x14,0x12,0x11], 'L' => [0x10,0x10,0x10,0x10,0x10,0x10,0x1F],
        'M' => [0x11,0x1B,0x15,0x15,0x11,0x11,0x11], 'N' => [0x11,0x11,0x19,0x15,0x13,0x11,0x11],
        'O' => [0x0E,0x11,0x11,0x11,0x11,0x11,0x0E], 'P' => [0x1E,0x11,0x11,0x1E,0x10,0x10,0x10],
        'Q' => [0x0E,0x11,0x11,0x11,0x15,0x12,0x0D], 'R' => [0x1E,0x11,0x11,0x1E,0x14,0x12,0x11],
        'S' => [0x0F,0x10,0x10,0x0E,0x01,0x01,0x1E], 'T' => [0x1F,0x04,0x04,0x04,0x04,0x04,0x04],
        'U' => [0x11,0x11,0x11,0x11,0x11,0x11,0x0E], 'V' => [0x11,0x11,0x11,0x11,0x11,0x0A,0x04],
        'W' => [0x11,0x11,0x11,0x15,0x15,0x1B,0x11], 'X' => [0x11,0x11,0x0A,0x04,0x0A,0x11,0x11],
        'Y' => [0x11,0x11,0x0A,0x04,0x04,0x04,0x04], 'Z' => [0x1F,0x01,0x02,0x04,0x08,0x10,0x1F],
        'a' => [0x00,0x00,0x0E,0x01,0x0F,0x11,0x0F], 'b' => [0x10,0x10,0x1E,0x11,0x11,0x11,0x1E],
        'c' => [0x00,0x00,0x0E,0x11,0x10,0x11,0x0E], 'd' => [0x01,0x01,0x0F,0x11,0x11,0x11,0x0F],
        'e' => [0x00,0x00,0x0E,0x11,0x1F,0x10,0x0E], 'f' => [0x06,0x09,0x08,0x1C,0x08,0x08,0x08],
        'g' => [0x00,0x00,0x0F,0x11,0x0F,0x01,0x0E], 'h' => [0x10,0x10,0x1E,0x11,0x11,0x11,0x11],
        'i' => [0x04,0x00,0x0C,0x04,0x04,0x04,0x0E], 'j' => [0x02,0x00,0x06,0x02,0x02,0x12,0x0C],
        'k' => [0x10,0x10,0x12,0x14,0x18,0x14,0x12], 'l' => [0x0C,0x04,0x04,0x04,0x04,0x04,0x0E],
        'm' => [0x00,0x00,0x1A,0x15,0x15,0x11,0x11], 'n' => [0x00,0x00,0x1E,0x11,0x11,0x11,0x11],
        'o' => [0x00,0x00,0x0E,0x11,0x11,0x11,0x0E], 'p' => [0x00,0x00,0x1E,0x11,0x1E,0x10,0x10],
        'q' => [0x00,0x00,0x0F,0x11,0x0F,0x01,0x01], 'r' => [0x00,0x00,0x16,0x19,0x10,0x10,0x10],
        's' => [0x00,0x00,0x0F,0x10,0x0E,0x01,0x1E], 't' => [0x08,0x08,0x1C,0x08,0x08,0x09,0x06],
        'u' => [0x00,0x00,0x11,0x11,0x11,0x13,0x0D], 'v' => [0x00,0x00,0x11,0x11,0x11,0x0A,0x04],
        'w' => [0x00,0x00,0x11,0x11,0x15,0x15,0x0A], 'x' => [0x00,0x00,0x11,0x0A,0x04,0x0A,0x11],
        'y' => [0x00,0x00,0x11,0x11,0x0F,0x01,0x0E], 'z' => [0x00,0x00,0x1F,0x02,0x04,0x08,0x1F],
        '.' => [0x00,0x00,0x00,0x00,0x00,0x0C,0x0C], ',' => [0x00,0x00,0x00,0x00,0x00,0x04,0x08],
        ':' => [0x00,0x0C,0x0C,0x00,0x0C,0x0C,0x00], '-' => [0x00,0x00,0x00,0x1F,0x00,0x00,0x00],
        '+' => [0x00,0x04,0x04,0x1F,0x04,0x04,0x00], '/' => [0x01,0x02,0x02,0x04,0x08,0x08,0x10],
        '(' => [0x02,0x04,0x08,0x08,0x08,0x04,0x02], ')' => [0x08,0x04,0x02,0x02,0x02,0x04,0x08],
        '[' => [0x0E,0x08,0x08,0x08,0x08,0x08,0x0E], ']' => [0x0E,0x02,0x02,0x02,0x02,0x02,0x0E],
        '_' => [0x00,0x00,0x00,0x00,0x00,0x00,0x1F], '%' => [0x19,0x1A,0x02,0x04,0x08,0x0B,0x13],
        '|' => [0x04,0x04,0x04,0x04,0x04,0x04,0x04], '=' => [0x00,0x00,0x1F,0x00,0x1F,0x00,0x00],
        '*' => [0x00,0x04,0x15,0x0E,0x15,0x04,0x00], '>' => [0x08,0x04,0x02,0x04,0x08,0x00,0x00],
        '#' => [0x0A,0x0A,0x1F,0x0A,0x1F,0x0A,0x0A], '\'' => [0x04,0x04,0x08,0x00,0x00,0x00,0x00],
        '"' => [0x0A,0x0A,0x00,0x00,0x00,0x00,0x00], '~' => [0x00,0x00,0x08,0x15,0x02,0x00,0x00],
        '°' => [0x06,0x09,0x06,0x00,0x00,0x00,0x00], 'Ø' => [0x0F,0x13,0x15,0x15,0x19,0x11,0x1E],
        _ => [0; 7],
    }
}

#[inline]
pub(super) fn blend_px(rgb: &mut [u8], w: i32, h: i32, x: i32, y: i32, c: [u8; 3], a: f32) {
    if x < 0 || y < 0 || x >= w || y >= h || a <= 0.0 {
        return;
    }
    let idx = ((y as usize) * (w as usize) + (x as usize)) * 3;
    let ia = 1.0 - a;
    rgb[idx] = (f32::from(rgb[idx]) * ia + f32::from(c[0]) * a).clamp(0.0, 255.0) as u8;
    rgb[idx + 1] = (f32::from(rgb[idx + 1]) * ia + f32::from(c[1]) * a).clamp(0.0, 255.0) as u8;
    rgb[idx + 2] = (f32::from(rgb[idx + 2]) * ia + f32::from(c[2]) * a).clamp(0.0, 255.0) as u8;
}

#[allow(clippy::too_many_arguments)]
pub(super) fn fill_rect(rgb: &mut [u8], w: u32, h: u32, x0: i32, y0: i32, rw: i32, rh: i32, c: [u8; 3], a: f32) {
    let (wi, hi) = (w as i32, h as i32);
    let xs = x0.max(0);
    let ys = y0.max(0);
    let xe = (x0 + rw).min(wi);
    let ye = (y0 + rh).min(hi);
    for y in ys..ye {
        for x in xs..xe {
            blend_px(rgb, wi, hi, x, y, c, a);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_text(
    rgb: &mut [u8],
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    text: &str,
    scale: i32,
    c: [u8; 3],
    a: f32,
) {
    let (wi, hi) = (w as i32, h as i32);
    let mut cx = x;
    for ch in text.chars() {
        let bits = glyph_5x7(ch);
        for (row, &mask) in bits.iter().enumerate() {
            for col in 0..5 {
                if (mask & (1 << (4 - col))) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            blend_px(rgb, wi, hi, cx + col * scale + sx, y + row as i32 * scale + sy, c, a);
                        }
                    }
                }
            }
        }
        cx += 6 * scale;
    }
}

/// Depth-tested line segment for the live 3D toolpath ribbon trail.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_line_2d(
    rgb: &mut [u8],
    zbuf: &[f32],
    w: u32,
    h: u32,
    x0: f32,
    y0: f32,
    d0: f32,
    x1: f32,
    y1: f32,
    d1: f32,
    c: [u8; 3],
    alpha: f32,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = dx.abs().max(dy.abs());
    if len < 0.01 {
        return;
    }
    let steps = (len.ceil() as i32).min(1200);
    let (wi, hi) = (w as i32, h as i32);
    for k in 0..=steps {
        let t = k as f32 / steps.max(1) as f32;
        let px = (x0 + dx * t).round() as i32;
        let py = (y0 + dy * t).round() as i32;
        if px < 0 || py < 0 || px >= wi || py >= hi {
            continue;
        }
        let d = d0 + (d1 - d0) * t;
        let zb = zbuf[py as usize * w as usize + px as usize];
        let vis = if d <= zb + 1.5 { alpha } else { alpha * 0.14 };
        blend_px(rgb, wi, hi, px, py, c, vis);
    }
}

/// Draw the compact dark top bar, tool pill, DRO card, transition banner, and segmented progress bar.
pub(super) fn draw_hud(rgb: &mut [u8], w: u32, h: u32, info: &HudInfo<'_>) {
    if w < 320 || h < 180 {
        return;
    }
    let wi = w as i32;
    let hi = h as i32;
    let top_h = 56;
    fill_rect(rgb, w, h, 0, 0, wi, top_h, [12, 16, 22], 0.94);
    fill_rect(rgb, w, h, 0, top_h, wi, 1, [70, 120, 180], 0.72);

    let card_w = 248.min((wi - 32).max(160));
    let card_x = wi - card_w - 14;
    let title_chars = info.setup_title.chars().count() as i32;
    let title_scale = if w >= 640 && 16 + title_chars * 12 + 12 < card_x { 2 } else { 1 };
    let title_y = if title_scale == 2 { 11 } else { 14 };
    draw_text(rgb, w, h, 16, title_y, info.setup_title, title_scale, [235, 242, 255], 1.0);
    draw_text(rgb, w, h, 16, 36, info.op_title, 1, [135, 200, 255], 1.0);

    // Top-right DRO / status readout card
    fill_rect(rgb, w, h, card_x, 8, card_w, 40, [18, 24, 34], 0.92);
    fill_rect(rgb, w, h, card_x, 8, card_w, 1, [60, 160, 110], 0.60);
    if info.show_tool {
        let dro = format!(
            "X:{:+6.1} Y:{:+6.1} Z:{:+6.1}",
            info.wcs_pos[0], info.wcs_pos[1], info.wcs_pos[2]
        );
        draw_text(rgb, w, h, card_x + 10, 14, &dro, 1, [120, 245, 175], 1.0);
    } else {
        let st = if info.progress_frac >= 0.99 {
            "STATUS: SETUP COMPLETE"
        } else {
            "STATUS: ORIENTING PART"
        };
        draw_text(rgb, w, h, card_x + 10, 14, st, 1, [120, 225, 255], 1.0);
    }
    let vol = format!(
        "Rem:{:5.1}cm3  Cut:{:4.1}%  M{}",
        info.remaining_cm3, info.removed_pct, info.move_num
    );
    draw_text(rgb, w, h, card_x + 10, 31, &vol, 1, [190, 205, 225], 1.0);

    // Top-center tool & RPM pill when window is wide enough
    if !info.tool_badge.is_empty() && wi >= 780 {
        let pill_w = 240;
        let pill_x = (card_x - pill_w - 14).max(380);
        if pill_x + pill_w <= card_x - 8 {
            fill_rect(rgb, w, h, pill_x, 8, pill_w, 40, [28, 38, 54], 0.90);
            fill_rect(rgb, w, h, pill_x, 8, pill_w, 1, [80, 130, 190], 0.55);
            draw_text(rgb, w, h, pill_x + 10, 14, info.tool_badge, 1, [245, 215, 115], 1.0);
            let feed_str = if !info.is_rapid && info.feed > 0.5 {
                format!("S{:.0} RPM   F{:.0} mm/min", info.rpm, info.feed)
            } else {
                format!("S{:.0} RPM   RAPID (G00)", info.rpm)
            };
            let fc = if !info.is_rapid && info.feed > 0.5 {
                [185, 205, 225]
            } else {
                [255, 140, 130]
            };
            draw_text(rgb, w, h, pill_x + 10, 31, &feed_str, 1, fc, 1.0);
        }
    }

    // Center bottom banner during transitions / showcase spin
    if !info.banner.is_empty() {
        let scale = if w >= 640 && info.banner.chars().count() * 12 + 36 < w as usize - 24 { 2 } else { 1 };
        let bw = (info.banner.chars().count() as i32) * (6 * scale) + 32;
        let bh = if scale == 2 { 34 } else { 22 };
        let bx = (wi - bw) / 2;
        let by = hi - 42 - bh - 8;
        fill_rect(rgb, w, h, bx, by, bw, bh, [14, 22, 34], 0.90);
        fill_rect(rgb, w, h, bx, by, bw, 2, [90, 190, 255], 0.92);
        draw_text(rgb, w, h, bx + 16, by + (bh - 7 * scale) / 2, info.banner, scale, [230, 244, 255], 1.0);
    }

    // Footer + segmented operation progress bar
    let foot_h = 42;
    fill_rect(rgb, w, h, 0, hi - foot_h, wi, foot_h, [12, 16, 22], 0.92);
    fill_rect(rgb, w, h, 0, hi - foot_h, wi, 1, [60, 80, 110], 0.65);
    let lw = info.footer_left.chars().count() as i32 * 6;
    let rw = info.footer_right.chars().count() as i32 * 6;
    let left_label = if lw + rw + 54 <= wi {
        info.footer_left
    } else if 12 * 6 + rw + 54 <= wi {
        "CamCraft Sim"
    } else {
        ""
    };
    if !left_label.is_empty() {
        draw_text(rgb, w, h, 18, hi - 35, left_label, 1, [160, 175, 195], 1.0);
    }
    if rw + 36 <= wi {
        draw_text(rgb, w, h, wi - rw - 18, hi - 35, info.footer_right, 1, [150, 165, 185], 1.0);
    }

    let bar_x = 18;
    let bar_y = hi - 18;
    let bar_w = wi - 36;
    let bar_h = 10;
    fill_rect(rgb, w, h, bar_x, bar_y, bar_w, bar_h, [35, 42, 54], 1.0);
    if info.total_segs > 0 {
        let frac = info.progress_frac.clamp(0.0, 1.0);
        let done_x = bar_x + (frac * bar_w as f32).round() as i32;
        for bar in info.bars {
            let ox0 = bar_x + ((bar.first_seg as f64 / info.total_segs as f64) * f64::from(bar_w)) as i32;
            let ox1 = bar_x + ((bar.end_seg as f64 / info.total_segs as f64) * f64::from(bar_w)) as i32;
            let cw = ox1.min(done_x) - ox0;
            if cw > 0 {
                fill_rect(rgb, w, h, ox0, bar_y, cw, bar_h, bar.color, 0.95);
            }
            fill_rect(rgb, w, h, ox0, bar_y, 1, bar_h, [10, 14, 20], 1.0);
        }
        fill_rect(rgb, w, h, done_x - 1, bar_y - 2, 3, bar_h + 4, [255, 255, 255], 0.95);
    }
}
