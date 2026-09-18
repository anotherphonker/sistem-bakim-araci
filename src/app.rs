//! Uygulama ağacı: sol panel (2 kademe), emoji kartları (liste/tablo),
//! sağ/alt konsol, üst araç çubuğu, alt durum çubuğu, renk ayarı penceresi.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::{vec2, Align, Color32, Layout, RichText, Sense};

use crate::actions;
use crate::config::Config;
use crate::console::{Con, Console};
use crate::emoji::{Emoji, Emojis};
use crate::i18n::{self, tr};
use crate::sections::{self, Action, Item, SectionId};
use crate::theme::{self, Theme};
use crate::winutil;

/// Uygulama sürümü (Hakkında'da ve kenar çubuğunda gösterilir).
const VERSION: &str = "0.1.0";

/// Link açar; yalnızca Windows'ta gerçekten çalışır.
fn webbrowser_open(url: &str) -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", url]);
        actions::set_no_window_flags(&mut c);
        if let Ok(mut child) = c.spawn() {
            let _ = child.wait();
            return true;
        }
        false
    }
    #[cfg(not(windows))]
    {
        let _ = url;
        false
    }
}

// ---------------------------------------------------------------------------
// durum
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConsolePos {
    Bottom,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SidebarMode {
    Full,
    Icons,
}

struct Worker {
    stop: Arc<AtomicBool>,
    done: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

// ---------------------------------------------------------------------------
// uygulama
// ---------------------------------------------------------------------------

pub struct SistemBakimApp {
    config: Config,
    theme_mode: String,
    lang: String,
    theme: Theme,
    theme_dirty: bool,

    current: SectionId,
    console: Console,
    con: Con,
    console_pos: ConsolePos,
    sidebar_mode: SidebarMode,

    emojis: Emojis,
    emoji_size: f32,

    worker: Option<Worker>,
    color_open: bool,
    about_open: bool,

    os_cache: String,
    os_cache_at: Instant,
}

impl SistemBakimApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load();
        let theme_mode = config.mode.clone();
        let lang = config.lang.clone();
        i18n::set_en(lang == "en");
        let theme = Theme::build(&theme_mode, &overrides_of(&config, &theme_mode));
        let (console, con) = Console::start();
        let mut emojis = Emojis::new();
        emojis.init(&cc.egui_ctx);
        Self {
            config,
            theme_mode,
            lang,
            theme,
            theme_dirty: true,
            current: SectionId::Home,
            console,
            con,
            console_pos: ConsolePos::Bottom,
            sidebar_mode: SidebarMode::Full,
            emojis,
            emoji_size: 26.0,
            worker: None,
            color_open: false,
            about_open: false,
            os_cache: String::new(),
            os_cache_at: Instant::now() - Duration::from_secs(3600),
        }
        .init()
    }

    fn init(self) -> Self {
        if !cfg!(windows) {
            self.con.info(tr(
                "Bu araç Windows için üretildi. Tam işlev Windows'ta çalışır.",
                "This tool is built for Windows. Full features run on Windows.",
            ));
        } else {
            self.con.info(tr(
                "Hazır. Soldan bölüm seç; araçlar listede, çıktılar konsolda akar.",
                "Ready. Pick a section on the left; tools are listed, output streams to the console.",
            ));
        }
        self
    }

    // ---------- tema / ayar ----------

    fn overrides(&self) -> HashMap<String, String> {
        overrides_of(&self.config, &self.theme_mode)
    }

    fn rebuild_theme(&mut self) {
        self.config.save();
        self.theme = Theme::build(&self.theme_mode, &self.overrides());
        self.theme_dirty = true;
    }

    fn toggle_theme(&mut self) {
        self.theme_mode = if self.theme_mode == "dark" {
            "light".to_string()
        } else {
            "dark".to_string()
        };
        self.config.mode = self.theme_mode.clone();
        self.rebuild_theme();
    }

    fn toggle_language(&mut self) {
        self.lang = if self.lang == "en" {
            "tr".to_string()
        } else {
            "en".to_string()
        };
        i18n::set_en(self.lang == "en");
        self.config.lang = self.lang.clone();
        self.config.save();
        self.con.info(tr(
            "Dil: Türkçe",
            "Language: English",
        ));
    }

    // ---------- iş yürütme ----------

    fn busy(&self) -> bool {
        self.worker.is_some()
    }

    fn spawn_job<F>(&mut self, f: F)
    where
        F: FnOnce(Con, Arc<AtomicBool>) + Send + 'static,
    {
        if self.worker.is_some() {
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let done = Arc::new(AtomicBool::new(false));
        let con = self.con.clone();
        let (s2, d2) = (stop.clone(), done.clone());
        let handle = std::thread::spawn(move || {
            f(con, s2);
            d2.store(true, Ordering::SeqCst);
        });
        self.worker = Some(Worker {
            stop,
            done,
            handle: Some(handle),
        });
    }

    fn reap_worker(&mut self) {
        if let Some(w) = &mut self.worker {
            if w.done.load(Ordering::SeqCst) {
                if let Some(h) = w.handle.take() {
                    let _ = h.join();
                }
                self.worker = None;
            }
        }
    }

    pub fn run_action(&mut self, act: Action) {
        let con = self.con.clone();
        match act {
            Action::SysInfo => {
                let admin = if winutil::is_admin() {
                    tr("Evet", "Yes")
                } else {
                    tr(
                        "Hayır (bazı araçlar çalışmaz)",
                        "No (some tools won't work)",
                    )
                };
                let uac = if winutil::lua_enabled() {
                    tr("Açık", "On")
                } else {
                    tr("Kapalı", "Off")
                };
                let fb = actions::free_gb();
                let fb = match fb {
                    Some(v) => format!("{v:.2} GB"),
                    None => "?".to_string(),
                };
                con.big("=".repeat(52));
                con.info(tr("SİSTEM BİLGİSİ", "SYSTEM INFO"));
                con.big("=".repeat(52));
                con.plain(format!(
                    "  {} : {}",
                    tr("İşletim Sistemi", "Operating System"),
                    self.os_string_cached()
                ));
                con.plain(format!(
                    "  {}       : {}",
                    tr("Kullanıcı", "User"),
                    std::env::var("USERNAME").unwrap_or_default()
                ));
                con.plain(format!(
                    "  {}      : {}",
                    tr("Bilgisayar", "Computer"),
                    std::env::var("COMPUTERNAME").unwrap_or_default()
                ));
                con.plain(format!(
                    "  {}     : {fb}",
                    tr("C: boş alan", "C: free space")
                ));
                con.plain(format!(
                    "  {}     : {admin}",
                    tr("Yönetici mi", "Is admin")
                ));
                con.plain(format!("  UAC             : {uac}"));
                con.plain(format!(
                    "  {}            : {}",
                    tr("Tema", "Theme"),
                    if self.theme_mode == "dark" {
                        tr("Koyu", "Dark")
                    } else {
                        tr("Açık", "Light")
                    }
                ));
                con.big("=".repeat(52));
            }
            Action::About => {
                self.about_open = !self.about_open;
            }
            Action::CpuInfo => {
                self.spawn_job(|cx, _s| actions::cpu::microcode_status(&cx));
            }
            Action::KnownIssues => {
                self.spawn_job(|cx, _s| actions::fixes::known_issues(&cx));
            }
            Action::PowerSaver => {
                self.spawn_job(|cx, _s| actions::power::power_saver(&cx));
            }
            Action::PowerBalanced => {
                self.spawn_job(|cx, _s| actions::power::balanced(&cx));
            }
            Action::PowerHighPerf => {
                self.spawn_job(|cx, _s| actions::power::high_perf(&cx));
            }
            Action::PowerUltimate => {
                self.spawn_job(|cx, _s| actions::power::ultimate(&cx));
            }
            Action::TurboOff => {
                self.spawn_job(|cx, _s| actions::power::turbo_off(&cx));
            }
            Action::TurboOn => {
                self.spawn_job(|cx, _s| actions::power::turbo_on(&cx));
            }
            Action::QuickClean => {
                self.spawn_job(|c, s| actions::quick_clean(&c, &s));
            }
            Action::DeepClean => {
                self.spawn_job(|c, s| actions::deep_clean(&c, &s));
            }
            Action::Explorer => {
                self.spawn_job(|c, s| actions::restart_explorer(&c, &s));
            }
            Action::Recycle => {
                self.spawn_job(|c, s| actions::empty_recycle_dialog(&c, &s));
            }
            Action::WuCache => {
                self.spawn_job(|c, s| actions::wu_cache(&c, &s));
            }
            Action::BigFiles => {
                self.spawn_job(|c, s| actions::big_files(&c, &s));
            }
            Action::FullRepair => {
                self.spawn_job(|c, s| actions::full_repair(&c, &s));
            }
            Action::Sfc => self.spawn_job(|c, s| {
                c.header("SFC /scannow");
                actions::run_forward(&c, &s, "sfc", &["/scannow"]);
                c.ok(tr("Tamamlandı.", "Done."));
            }),
            Action::CheckHealth => {
                self.spawn_job(|c, s| actions::simple_dism(&c, &s, "CheckHealth", "/checkhealth"));
            }
            Action::ScanHealth => {
                self.spawn_job(|c, s| actions::simple_dism(&c, &s, "ScanHealth", "/scanhealth"));
            }
            Action::RestoreHealth => {
                self.spawn_job(|c, s| actions::restore_health(&c, &s));
            }
            Action::ComponentCleanup => self.spawn_job(|c, s| {
                actions::simple_dism(&c, &s, "ComponentCleanup", "/startcomponentcleanup")
            }),
            Action::AnalyzeStore => self.spawn_job(|c, s| {
                actions::simple_dism(&c, &s, "AnalyzeComponentStore", "/analyzecomponentstore")
            }),
            Action::Chkdsk => {
                self.spawn_job(|c, s| actions::chkdsk(&c, &s));
            }
            Action::DiskInfo => {
                self.spawn_job(|c, s| actions::disk_info(&c, &s));
            }
            Action::Logs => {
                self.spawn_job(|c, s| actions::copy_logs(&c, &s));
            }
            Action::RestoreIso => {
                #[cfg(windows)]
                let picked = rfd::FileDialog::new()
                    .add_filter(tr("ISO dosyası", "ISO file"), &["iso"])
                    .pick_file();
                #[cfg(not(windows))]
                let picked: Option<std::path::PathBuf> = None;
                match picked {
                    Some(p) => {
                        let iso = p.to_string_lossy().to_string();
                        self.spawn_job(move |c, s| actions::restore_iso(&c, &s, &iso));
                    }
                    None => con.info(tr("ISO seçilmedi.", "No ISO selected.")),
                }
            }
        }
    }

    fn os_string_cached(&mut self) -> String {
        if self.os_cache.is_empty() || self.os_cache_at.elapsed() > Duration::from_secs(5) {
            self.os_cache = actions::os_string();
            self.os_cache_at = Instant::now();
        }
        self.os_cache.clone()
    }
}

fn overrides_of(config: &Config, mode: &str) -> HashMap<String, String> {
    config.overrides.get(mode).cloned().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// egui render
// ---------------------------------------------------------------------------

impl eframe::App for SistemBakimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.theme_dirty {
            self.theme.apply(ctx);
            self.theme_dirty = false;
        }
        self.console.poll();
        self.reap_worker();

        let t = self.theme.clone();
        let nb = nav_items();

        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            self.console.visible = !self.console.visible;
        }

        // --- sol kenar paneli (moda özel Id => genişlik her geçişte sıfırlanır) ---
        let (side_w, side_id, resizable) = match self.sidebar_mode {
            SidebarMode::Full => (210.0, "sidebar_full", true),
            SidebarMode::Icons => (76.0, "sidebar_icons", false),
        };
        egui::SidePanel::left(side_id)
            .resizable(resizable)
            .default_width(side_w)
            .min_width(64.0)
            .width_range(64.0..=420.0)
            .frame(
                egui::Frame::NONE
                    .fill(t.panel)
                    .inner_margin(egui::Margin::symmetric(8, 10)),
            )
            .show(ctx, |ui| self.sidebar_ui(ui, &t, &nb));

        // --- konsol paneli (konuma özel Id) ---
        if self.console.visible {
            let frame = egui::Frame::NONE
                .fill(t.outbg)
                .inner_margin(egui::Margin::symmetric(8, 8));
            match self.console_pos {
                ConsolePos::Right => {
                    egui::SidePanel::right("console_right")
                        .resizable(true)
                        .default_width(430.0)
                        .min_width(180.0)
                        .width_range(180.0..=900.0)
                        .frame(frame)
                        .show(ctx, |ui| self.console_ui(ui, &t));
                }
                ConsolePos::Bottom => {
                    egui::TopBottomPanel::bottom("console_bottom")
                        .resizable(true)
                        .default_height(210.0)
                        .min_height(100.0)
                        .height_range(100.0..=640.0)
                        .frame(frame)
                        .show(ctx, |ui| self.console_ui(ui, &t));
                }
            }
        }

        // --- durum çubuğu ---
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::NONE
                    .fill(t.panel)
                    .inner_margin(egui::Margin::symmetric(12, 7)),
            )
            .show(ctx, |ui| self.status_ui(ui, &t));

        // --- orta alan ---
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(t.bg)
                    .inner_margin(egui::Margin::same(14)),
            )
            .show(ctx, |ui| self.main_ui(ui, &t));

        // --- pencereler ---
        if self.color_open {
            let mut open = true;
            egui::Window::new(tr("Renkleri Özelleştir", "Customize Colors"))
                .open(&mut open)
                .collapsible(false)
                .show(ctx, |ui| self.color_ui(ui));
            self.color_open = open;
        }
        if self.about_open {
            let mut open = true;
            egui::Window::new(tr("Hakkında", "About"))
                .open(&mut open)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(RichText::new("Sistem Bakım Aracı").size(17.0).strong());
                    ui.label(
                        RichText::new(format!("{} v{VERSION}", tr("Sürüm", "Version")))
                            .color(t.muted),
                    );
                    ui.separator();
                    ui.label(tr(
                        "Windows bakım ve onarım aracı.",
                        "Windows maintenance and repair tool.",
                    ));
                    ui.label(tr(
                        "Temizlik + SFC/DISM onarım + disk + log araçları.",
                        "Cleanup + SFC/DISM repair + disk + log tools.",
                    ));
                    ui.label(tr(
                        "Koyu/açık mod, liste görünümü, renk özelleştirme.",
                        "Dark/light mode, list view, color customization.",
                    ));
                    ui.label(tr(
                        "Renkli emojiler exe'ye gömülü - Windows sürümünden bağımsız.",
                        "Color emojis are embedded in the exe - independent of Windows version.",
                    ));
                    ui.separator();
                    ui.label(RichText::new(tr("Geliştirici: Resul Çelik", "Developer: Resul Çelik")).strong());
                    let link = "https://github.com/anotherphonker";
                    ui.horizontal(|ui| {
                        ui.label("GitHub:");
                        if ui
                            .add(egui::Link::new("github.com/anotherphonker"))
                            .clicked()
                        {
                            let _ = webbrowser_open(link);
                        }
                    });
                    let build = actions::os_build_string();
                    if !build.is_empty() {
                        ui.label(
                            RichText::new(format!(
                                "{}: {build}",
                                tr("İşletim sistemi derlemesi", "OS build")
                            ))
                            .color(t.muted),
                        );
                    }
                });
            self.about_open = open;
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(w) = &self.worker {
            w.stop.store(true, Ordering::SeqCst);
        }
        self.config.save();
    }
}

// ---------------------------------------------------------------------------
// alt paneller
// ---------------------------------------------------------------------------

fn nav_items() -> Vec<(SectionId, Emoji, &'static str)> {
    sections::all()
        .into_iter()
        .map(|s| {
            let title = if i18n::is_en() { s.title_en } else { s.title };
            (s.id, s.nav_emoji, title)
        })
        .collect()
}

impl SistemBakimApp {
    /// Sabit boyutlu kutuya emoji çiz (cache'ten).
    fn emoji_at(&mut self, ui: &mut egui::Ui, size: f32, e: Emoji) {
        if let Some(h) = self.emojis.handle(e) {
            let img = egui::Image::from_texture((h.id(), h.size_vec2()))
                .fit_to_exact_size(vec2(size, size));
            ui.add(img);
        } else {
            // doku daha yüklenmediyse şimdilik boş kutu (aynı boyutta)
            ui.allocate_exact_size(vec2(size, size), Sense::hover());
        }
    }

    /// Emoji + tıklama alanı (kartlar için sensörlü).
    fn emoji_btn(&mut self, ui: &mut egui::Ui, size: f32, e: Emoji) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
        if let Some(h) = self.emojis.handle(e) {
            let img = egui::Image::from_texture((h.id(), h.size_vec2()))
                .fit_to_exact_size(vec2(size, size));
            ui.put(rect, img);
        }
        resp
    }

    fn sidebar_ui(&mut self, ui: &mut egui::Ui, t: &Theme, nb: &[(SectionId, Emoji, &str)]) {
        let icons = self.sidebar_mode == SidebarMode::Icons;
        let n = self.emoji_size.max(24.0);
        if icons {
            ui.add_space(4.0);
            ui.vertical_centered(|ui| {
                self.emoji_at(ui, n + 6.0, Emoji::Wrench);
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                for (id, e, tip) in nb {
                    let active = *id == self.current;
                    let sz = if active { n + 6.0 } else { n };
                    let r = self.emoji_btn(ui, sz, *e).on_hover_text(*tip);
                    if r.clicked() {
                        self.current = *id;
                    }
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
                let (e, tip) = if self.theme_mode == "dark" {
                    (Emoji::Sun, tr("Açık moda geç", "Switch to light mode"))
                } else {
                    (Emoji::Moon, tr("Koyu moda geç", "Switch to dark mode"))
                };
                if self.emoji_btn(ui, n, e).on_hover_text(tip).clicked() {
                    self.toggle_theme();
                }
                if self
                    .emoji_btn(ui, n, Emoji::Palette)
                    .on_hover_text(tr("Renkleri özelleştir", "Customize colors"))
                    .clicked()
                {
                    self.color_open = true;
                }
                // Dil değiştirme bayrağı (icons modu)
                let flag = if i18n::is_en() { Emoji::FlagUs } else { Emoji::FlagTr };
                let lang_tip = if i18n::is_en() { "English" } else { "Türkçe" };
                if self.emoji_btn(ui, n, flag).on_hover_text(lang_tip).clicked() {
                    self.toggle_language();
                }
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("SİSTEM BAKIM")
                        .color(t.accent)
                        .size(14.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{} v{VERSION}", tr("ARACI", "TOOL")))
                        .color(t.muted)
                        .size(10.0),
                );
                ui.add_space(12.0);
            });

            for (id, e, title) in nb {
                let active = *id == self.current;
                if self.nav_item(ui, t, *e, title, active).clicked() {
                    self.current = *id;
                }
            }
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            let (e, theme_label) = if self.theme_mode == "dark" {
                (Emoji::Sun, tr("Açık Mod", "Light Mode"))
            } else {
                (Emoji::Moon, tr("Koyu Mod", "Dark Mode"))
            };
            if self.nav_item(ui, t, e, theme_label, false).clicked() {
                self.toggle_theme();
            }
            if self
                .nav_item(ui, t, Emoji::Palette, tr("Renkler", "Colors"), false)
                .clicked()
            {
                self.color_open = true;
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "{}: {}",
                    tr("Konsol", "Console"),
                    if self.console.visible {
                        tr("Açık", "On")
                    } else {
                        tr("Kapalı", "Off")
                    }
                ))
                .color(t.muted)
                .size(10.5),
            );
            if !winutil::is_admin() {
                ui.label(
                    RichText::new(tr("Yönetici değilsin", "Not running as admin"))
                        .color(t.warn)
                        .size(10.5),
                );
            }

            // Dil değiştirme: EN ALTA bayrak (basınca diğer dile geçer)
            let (flag, lang_label) = if i18n::is_en() {
                (Emoji::FlagUs, "English")
            } else {
                (Emoji::FlagTr, "Türkçe")
            };
            if self.nav_item(ui, t, flag, lang_label, false).clicked() {
                self.toggle_language();
            }
        }
    }

    /// Tam modda satır (emoji + etiket).
    fn nav_item(
        &mut self,
        ui: &mut egui::Ui,
        t: &Theme,
        e: Emoji,
        label: &str,
        active: bool,
    ) -> egui::Response {
        let h = 32.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
        let bg = if active {
            t.card
        } else if resp.hovered() {
            t.card_h
        } else {
            Color32::TRANSPARENT
        };
        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, 6.0, bg);
        }
        let fg = if active { t.accent } else { t.text };
        let n = self.emoji_size.max(22.0);
        let ix = rect.min.x + 10.0;
        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(ix, rect.center().y - n / 2.0),
            vec2(n, n),
        );
        if let Some(hdl) = self.emojis.handle(e) {
            let img = egui::Image::from_texture((hdl.id(), hdl.size_vec2()))
                .fit_to_exact_size(vec2(n, n));
            ui.put(icon_rect, img);
        }
        ui.painter().text(
            egui::pos2(ix + n + 10.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.5),
            fg,
        );
        resp
    }

    fn console_ui(&mut self, ui: &mut egui::Ui, t: &Theme) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(tr("ÇIKTI / KONSOL", "OUTPUT / CONSOLE"))
                    .color(t.muted)
                    .size(12.0)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let clr = egui::Button::new(
                    RichText::new(tr("Temizle", "Clear")).color(t.muted).size(11.0),
                )
                .fill(t.card)
                .corner_radius(4.0);
                if ui.add(clr).clicked() {
                    self.console.clear();
                    self.con.clear();
                }
                let scr_label = if self.console.autoscroll {
                    tr("Canlı kaydır", "Auto-scroll")
                } else {
                    tr("Duraklat", "Paused")
                };
                let scr = egui::Button::new(
                    RichText::new(scr_label)
                        .color(if self.console.autoscroll { t.ok } else { t.warn })
                        .size(11.0),
                )
                .fill(t.card)
                .corner_radius(4.0);
                if ui.add(scr).clicked() {
                    self.console.autoscroll = !self.console.autoscroll;
                }
            });
        });
        ui.add_space(4.0);
        self.console.show(ui, t);
    }

    fn status_ui(&mut self, ui: &mut egui::Ui, t: &Theme) {
        ui.horizontal(|ui| {
            let fb = match actions::free_gb() {
                Some(v) => format!("C: {v:.1} GB {}", tr("boş", "free")),
                None => tr("C: boş alan ?", "C: free space ?").to_string(),
            };
            let admin = if winutil::is_admin() {
                tr("Yönetici", "Admin")
            } else {
                tr("Yönetici değil", "Not admin")
            };
            let os = self.os_string_cached();
            let full = format!("{os}   |   {fb}   |   {admin}");

            // Durdur butonuna yer ayır, kalanına metni ölçerek sığdır.
            // (egui'nin kendi truncate'i U+2026 (uc nokta) kullandigi icin kutu cizer;
            //  bu yuzden kirpma elle ASCII "..." ile yapilir.)
            let btn_w = 78.0;
            let budget = (ui.available_width() - btn_w).max(60.0);
            let text = self.fit_status(ui.ctx(), &full, 11.5, t.muted, budget);

            ui.scope(|ui| {
                ui.set_max_width(budget);
                ui.label(RichText::new(text).color(t.muted).size(11.5));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let busy = self.busy();
                let btn = egui::Button::new(
                    RichText::new(tr("Durdur", "Stop")).color(t.err).size(12.0).strong(),
                )
                .fill(t.card);
                if ui.add_enabled(busy, btn).clicked() {
                    if let Some(w) = &self.worker {
                        w.stop.store(true, Ordering::SeqCst);
                    }
                }
            });
        });
    }

    /// Metni verilen piksel genişliğine ASCII "..." ile sığdırır (kutu yapmaz).
    fn fit_status(
        &self,
        ctx: &egui::Context,
        s: &str,
        size: f32,
        color: Color32,
        max_w: f32,
    ) -> String {
        let measure = |t: &str| {
            ctx.fonts(|f| {
                f.layout_no_wrap(t.to_string(), egui::FontId::proportional(size), color)
                    .size()
                    .x
            })
        };
        if measure(s) <= max_w {
            return s.to_string();
        }
        let mut lo = 0usize;
        let mut hi = s.len();
        while lo < hi {
            let mid = s.floor_char_boundary((lo + hi + 1) / 2).min(hi);
            if mid <= lo {
                break;
            }
            let cand = format!("{}...", &s[..mid]);
            if measure(&cand) <= max_w {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        if lo == 0 {
            "...".to_string()
        } else {
            format!("{}...", &s[..s.floor_char_boundary(lo)])
        }
    }

    fn main_ui(&mut self, ui: &mut egui::Ui, t: &Theme) {
        let sec = sections::all()
            .into_iter()
            .find(|s| s.id == self.current)
            .unwrap();
        let title = if i18n::is_en() { sec.title_en } else { sec.title };
        let sub = if i18n::is_en() { sec.sub_en } else { sec.sub };

        // --- üst bar ---
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(title).color(t.text).size(18.0).strong());
                ui.label(RichText::new(sub).color(t.muted).size(11.5));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let mlabel = match self.sidebar_mode {
                    SidebarMode::Full => tr("Menü: Tam", "Menu: Full"),
                    SidebarMode::Icons => tr("Menü: İkon", "Menu: Icons"),
                };
                if self.top_btn(ui, t, mlabel) {
                    self.sidebar_mode = match self.sidebar_mode {
                        SidebarMode::Full => SidebarMode::Icons,
                        SidebarMode::Icons => SidebarMode::Full,
                    };
                }
                if self.top_btn(
                    ui,
                    t,
                    if self.console_pos == ConsolePos::Bottom {
                        tr("Yerleşim: Alt", "Layout: Bottom")
                    } else {
                        tr("Yerleşim: Sağ", "Layout: Right")
                    },
                ) {
                    self.console_pos = if self.console_pos == ConsolePos::Bottom {
                        ConsolePos::Right
                    } else {
                        ConsolePos::Bottom
                    };
                }
                if self.top_btn(
                    ui,
                    t,
                    if self.console.visible {
                        tr("Konsol: Açık", "Console: On")
                    } else {
                        tr("Konsol: Kapalı", "Console: Off")
                    },
                ) {
                    self.console.visible = !self.console.visible;
                }
            });
        });
        ui.add_space(4.0);

        // --- gövde ---
        let items = sec.items;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| self.render_list(ui, t, &items));
    }

    fn top_btn(&self, ui: &mut egui::Ui, t: &Theme, label: &str) -> bool {
        ui.add(
            egui::Button::new(RichText::new(label).color(t.muted).size(12.0))
                .fill(t.card)
                .corner_radius(5.0),
        )
        .clicked()
    }

    fn render_list(&mut self, ui: &mut egui::Ui, t: &Theme, items: &[Item]) {
        let n = self.emoji_size.max(26.0);
        for it in items {
            let title = if i18n::is_en() { it.title_en } else { it.title };
            let desc = if i18n::is_en() { it.desc_en } else { it.desc };
            let frame = egui::Frame::NONE
                .fill(t.card)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .corner_radius(8.0);
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        self.emoji_at(ui, n, it.emoji);
                    });
                    ui.add_space(10.0);
                    let (fill, fg) = if it.accent {
                        (t.accent_d, t.accent_text)
                    } else {
                        (t.card_h, t.text)
                    };
                    let btn =
                        egui::Button::new(RichText::new(title).color(fg).size(14.0).strong())
                            .fill(fill)
                            .min_size(vec2(ui.available_width(), 28.0))
                            .corner_radius(6.0);
                    if ui.add(btn).clicked() {
                        self.run_action(it.action);
                    }
                });
                ui.add_space(4.0);
                ui.label(RichText::new(desc).color(t.muted).size(12.0));
            });
            ui.add_space(6.0);
        }
    }

    fn color_ui(&mut self, ui: &mut egui::Ui) {
        let mode = if self.theme_mode == "dark" {
            tr("Koyu", "Dark")
        } else {
            tr("Açık", "Light")
        };
        ui.label(
            RichText::new(format!(
                "{} ({}: {mode})",
                tr("Renkler", "Colors"),
                tr("aktif tema", "active theme")
            ))
            .color(self.theme.text)
            .strong(),
        );
        ui.separator();

        let mut working = self.overrides();
        let base = if self.theme_mode == "dark" {
            theme::dark()
        } else {
            theme::light()
        };

        for (key, label_tr, label_en) in theme::ROLES {
            let label = tr(label_tr, label_en);
            let cur = match working.get(key).and_then(|s| theme::parse_hex(s)) {
                Some(c) => c,
                None => base.role(key),
            };
            let mut c = cur;
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{label}:")));
                if ui.color_edit_button_srgba(&mut c).changed() {
                    working.insert(key.to_string(), Theme::to_hex(c));
                }
            });
        }

        if working != self.overrides() {
            self.config
                .overrides
                .insert(self.theme_mode.clone(), working);
            self.rebuild_theme();
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new(tr("Varsayılana Dön", "Reset to Default")))
                        .fill(self.theme.card),
                )
                .clicked()
            {
                self.config.overrides.remove(&self.theme_mode);
                self.rebuild_theme();
            }
        });
    }
}
