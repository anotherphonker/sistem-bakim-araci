//! Gömülü renkli emoji (Twemoji PNG, 72x72).
//!
//! Emoji dosyaları `include_bytes!` ile .exe'nin İÇİNE gömülür; görünüm
//! Windows sürümünden ve sistem fontlarından tamamen bağımsızdır —
//! Windows 10 / 11 / LTSC fark etmeksizin birebir aynı renkli emoji çizilir.
//! Twemoji grafikleri CC-BY 4.0 lisanslıdır (© Twitter/Mozilla).

use std::collections::HashMap;

use egui::{ColorImage, Context, TextureHandle, TextureOptions};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Emoji {
    Home,
    Broom,
    Sponge,
    Shield,
    Disk,
    Optical,
    Info,
    Trash,
    Recycle,
    Refresh,
    Chart,
    Inbox,
    Moon,
    Palette,
    Clipboard,
    Pill,
    Stethoscope,
    Wrench,
    Sun,
    Magnifier,
    Bolt,
    Computer,
    Rocket,
    FlagTr,
    FlagUs,
    Laptop,
    Controls,
    Battery,
    Scale,
}

pub const ALL: [Emoji; 29] = [
    Emoji::Home,
    Emoji::Broom,
    Emoji::Sponge,
    Emoji::Shield,
    Emoji::Disk,
    Emoji::Optical,
    Emoji::Info,
    Emoji::Trash,
    Emoji::Recycle,
    Emoji::Refresh,
    Emoji::Chart,
    Emoji::Inbox,
    Emoji::Moon,
    Emoji::Palette,
    Emoji::Clipboard,
    Emoji::Pill,
    Emoji::Stethoscope,
    Emoji::Wrench,
    Emoji::Sun,
    Emoji::Magnifier,
    Emoji::Bolt,
    Emoji::Computer,
    Emoji::Rocket,
    Emoji::FlagTr,
    Emoji::FlagUs,
    Emoji::Laptop,
    Emoji::Controls,
    Emoji::Battery,
    Emoji::Scale,
];

impl Emoji {
    pub const fn label(self) -> &'static str {
        match self {
            Emoji::Home => "emoji_home",
            Emoji::Broom => "emoji_broom",
            Emoji::Sponge => "emoji_sponge",
            Emoji::Shield => "emoji_shield",
            Emoji::Disk => "emoji_disk",
            Emoji::Optical => "emoji_optical",
            Emoji::Info => "emoji_info",
            Emoji::Trash => "emoji_trash",
            Emoji::Recycle => "emoji_recycle",
            Emoji::Refresh => "emoji_refresh",
            Emoji::Chart => "emoji_chart",
            Emoji::Inbox => "emoji_inbox",
            Emoji::Moon => "emoji_moon",
            Emoji::Palette => "emoji_palette",
            Emoji::Clipboard => "emoji_clipboard",
            Emoji::Pill => "emoji_pill",
            Emoji::Stethoscope => "emoji_stethoscope",
            Emoji::Wrench => "emoji_wrench",
            Emoji::Sun => "emoji_sun",
            Emoji::Magnifier => "emoji_magnifier",
            Emoji::Bolt => "emoji_bolt",
            Emoji::Computer => "emoji_computer",
            Emoji::Rocket => "emoji_rocket",
            Emoji::FlagTr => "emoji_flagtr",
            Emoji::FlagUs => "emoji_flagus",
            Emoji::Laptop => "emoji_laptop",
            Emoji::Controls => "emoji_controls",
            Emoji::Battery => "emoji_battery",
            Emoji::Scale => "emoji_scale",
        }
    }

    pub const fn bytes(self) -> &'static [u8] {
        match self {
            Emoji::Home => include_bytes!("../assets/emoji/1f3e0.png"),
            Emoji::Broom => include_bytes!("../assets/emoji/1f9f9.png"),
            Emoji::Sponge => include_bytes!("../assets/emoji/1f9fd.png"),
            Emoji::Shield => include_bytes!("../assets/emoji/1f6e1.png"),
            Emoji::Disk => include_bytes!("../assets/emoji/1f4be.png"),
            Emoji::Optical => include_bytes!("../assets/emoji/1f4bf.png"),
            Emoji::Info => include_bytes!("../assets/emoji/2139.png"),
            Emoji::Trash => include_bytes!("../assets/emoji/1f5d1.png"),
            Emoji::Recycle => include_bytes!("../assets/emoji/267b.png"),
            Emoji::Refresh => include_bytes!("../assets/emoji/1f504.png"),
            Emoji::Chart => include_bytes!("../assets/emoji/1f4ca.png"),
            Emoji::Inbox => include_bytes!("../assets/emoji/1f4e5.png"),
            Emoji::Moon => include_bytes!("../assets/emoji/1f319.png"),
            Emoji::Palette => include_bytes!("../assets/emoji/1f3a8.png"),
            Emoji::Clipboard => include_bytes!("../assets/emoji/1f4cb.png"),
            Emoji::Pill => include_bytes!("../assets/emoji/1f48a.png"),
            Emoji::Stethoscope => include_bytes!("../assets/emoji/1fa7a.png"),
            Emoji::Wrench => include_bytes!("../assets/emoji/1f6e0.png"),
            Emoji::Sun => include_bytes!("../assets/emoji/2600.png"),
            Emoji::Magnifier => include_bytes!("../assets/emoji/1f50d.png"),
            Emoji::Bolt => include_bytes!("../assets/emoji/26a1.png"),
            Emoji::Computer => include_bytes!("../assets/emoji/1f5a5.png"),
            Emoji::Rocket => include_bytes!("../assets/emoji/1f680.png"),
            Emoji::FlagTr => include_bytes!("../assets/emoji/1f1f9-1f1f7.png"),
            Emoji::FlagUs => include_bytes!("../assets/emoji/1f1fa-1f1f8.png"),
            Emoji::Laptop => include_bytes!("../assets/emoji/1f4bb.png"),
            Emoji::Controls => include_bytes!("../assets/emoji/1f39b.png"),
            Emoji::Battery => include_bytes!("../assets/emoji/1f50b.png"),
            Emoji::Scale => include_bytes!("../assets/emoji/2696.png"),
        }
    }
}

/// Emoji dokuları: ilk çalıştırmada PNG baytlarından GPU dokusuna dönüştürülür.
pub struct Emojis {
    map: HashMap<Emoji, TextureHandle>,
}

impl Emojis {
    pub fn new() -> Self {
        Emojis {
            map: HashMap::new(),
        }
    }

    pub fn init(&mut self, ctx: &Context) {
        for e in ALL {
            if !self.map.contains_key(&e) {
                self.load(ctx, e);
            }
        }
    }

    fn load(&mut self, ctx: &Context, e: Emoji) {
        if let Some(img) = decode_png(e.bytes()) {
            let handle = ctx.load_texture(e.label(), img, TextureOptions::LINEAR);
            self.map.insert(e, handle);
        }
    }

    pub fn handle(&self, e: Emoji) -> Option<&TextureHandle> {
        self.map.get(&e)
    }
}

/// PNG (paletli / rgba dahil) → egui dokusu.
pub fn decode_png(bytes: &[u8]) -> Option<ColorImage> {
    let rgba = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some(ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        rgba.as_raw(),
    ))
}
