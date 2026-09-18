//! Dil desteği (Türkçe / English).
//!
//! Aktif dil global bir bayrakla tutulur; UI ipliği ve worker iplikleri
//! `t(tr, en)` çağrısıyla o anki dile göre metin seçer. Bayrağı değiştirmek
//! tüm metinlerin anında yenilenmesini sağlar.

use std::sync::atomic::{AtomicBool, Ordering};

/// true = English, false = Türkçe.
static LANG_EN: AtomicBool = AtomicBool::new(false);

pub fn is_en() -> bool {
    LANG_EN.load(Ordering::Relaxed)
}

pub fn set_en(v: bool) {
    LANG_EN.store(v, Ordering::Relaxed);
}

/// Aktif dile göre iki `'static` metinden birini seç.
pub fn tr(turkish: &'static str, english: &'static str) -> &'static str {
    if is_en() {
        english
    } else {
        turkish
    }
}

/// Aktif dile göre iki hesaplanmış (format!'lı) metinden birini seç.
pub fn pick(tr: String, en: String) -> String {
    if is_en() {
        en
    } else {
        tr
    }
}
