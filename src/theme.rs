//! Tema: koyu / açık paletler + kullanıcı renk override'ları.

use egui::Color32;
use std::collections::HashMap;

/// Kullanıcının özelleştirebileceği roller (çift dilli: key, TR, EN).
pub const ROLES: [(&str, &str, &str); 11] = [
    ("accent", "Vurgu rengi", "Accent color"),
    ("bg", "Arka plan", "Background"),
    ("panel", "Kenar panel", "Side panel"),
    ("card", "Kartlar", "Cards"),
    ("text", "Metin", "Text"),
    ("muted", "Soluk metin", "Muted text"),
    ("ok", "Başarı / yeşil", "Success / green"),
    ("err", "Hata / kırmızı", "Error / red"),
    ("warn", "Uyarı / sarı", "Warning / yellow"),
    ("outbg", "Konsol arka planı", "Console background"),
    ("outfg", "Konsol metni", "Console text"),
];

#[derive(Clone)]
pub struct Theme {
    pub bg: Color32,
    pub panel: Color32,
    pub card: Color32,
    pub card_h: Color32,
    pub accent: Color32,
    pub accent_d: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub ok: Color32,
    pub err: Color32,
    pub warn: Color32,
    pub outbg: Color32,
    pub outfg: Color32,
    pub accent_text: Color32,
}

fn hx(v: u32) -> Color32 {
    Color32::from_rgb(((v >> 16) & 0xff) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
}

pub fn dark() -> Theme {
    Theme {
        bg: hx(0x0b1119),
        panel: hx(0x131b29),
        card: hx(0x182234),
        card_h: hx(0x22314a),
        accent: hx(0x28d5e0),
        accent_d: hx(0x1aa8b4),
        text: hx(0xd7e3f0),
        muted: hx(0x7d8da3),
        ok: hx(0x3fd68f),
        err: hx(0xff5f6d),
        warn: hx(0xffc857),
        outbg: hx(0x0a0f17),
        outfg: hx(0xc9d6e5),
        accent_text: hx(0x06222a),
    }
}

pub fn light() -> Theme {
    Theme {
        bg: hx(0xeef2f7),
        panel: hx(0xdde5ef),
        card: hx(0xffffff),
        card_h: hx(0xe6edf6),
        accent: hx(0x0e9ab0),
        accent_d: hx(0x0b8296),
        text: hx(0x1c2b3a),
        muted: hx(0x5b6b7d),
        ok: hx(0x12805c),
        err: hx(0xcf2f43),
        warn: hx(0xb97912),
        outbg: hx(0xffffff),
        outfg: hx(0x22303f),
        accent_text: hx(0xffffff),
    }
}

pub fn parse_hex(s: &str) -> Option<Color32> {
    let s = s.trim().trim_start_matches('#');
    match s.len() {
        6 => {
            let v = u32::from_str_radix(s, 16).ok()?;
            Some(hx(v))
        }
        8 => {
            let v = u32::from_str_radix(s, 16).ok()?;
            Some(Color32::from_rgba_unmultiplied(
                ((v >> 24) & 0xff) as u8,
                ((v >> 16) & 0xff) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
            ))
        }
        _ => None,
    }
}

impl Theme {
    pub fn role(&self, key: &str) -> Color32 {
        match key {
            "accent" => self.accent,
            "bg" => self.bg,
            "panel" => self.panel,
            "card" => self.card,
            "text" => self.text,
            "muted" => self.muted,
            "ok" => self.ok,
            "err" => self.err,
            "warn" => self.warn,
            "outbg" => self.outbg,
            "outfg" => self.outfg,
            _ => self.accent,
        }
    }

    pub fn set_role(&mut self, key: &str, c: Color32) {
        match key {
            "accent" => self.accent = c,
            "bg" => self.bg = c,
            "panel" => self.panel = c,
            "card" => self.card = c,
            "text" => self.text = c,
            "muted" => self.muted = c,
            "ok" => self.ok = c,
            "err" => self.err = c,
            "warn" => self.warn = c,
            "outbg" => self.outbg = c,
            "outfg" => self.outfg = c,
            _ => {}
        }
    }

    pub fn to_hex(c: Color32) -> String {
        format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
    }

    pub fn build(mode: &str, overrides: &HashMap<String, String>) -> Theme {
        let mut t = if mode == "light" { light() } else { dark() };
        for (k, v) in overrides {
            if let Some(c) = parse_hex(v) {
                t.set_role(k, c);
            }
        }
        t
    }

    pub fn is_dark(&self) -> bool {
        let r = self.bg.r() as f32 / 255.0;
        let g = self.bg.g() as f32 / 255.0;
        let b = self.bg.b() as f32 / 255.0;
        0.2126 * r + 0.7152 * g + 0.0722 * b < 0.5
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut v = if self.is_dark() {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        v.panel_fill = self.card;
        v.window_fill = self.card;
        v.extreme_bg_color = self.bg;
        v.faint_bg_color = self.card_h;
        v.override_text_color = Some(self.text);
        v.selection.bg_fill = self.accent;
        v.selection.stroke.color = self.accent;
        v.hyperlink_color = self.accent;
        v.widgets.inactive.fg_stroke.color = self.text;
        v.widgets.hovered.fg_stroke.color = self.text;
        v.widgets.active.fg_stroke.color = self.text;
        ctx.set_visuals(v);

        let mut st = (*ctx.style()).clone();
        st.spacing.item_spacing = egui::vec2(8.0, 6.0);
        ctx.set_style(st);
    }
}
