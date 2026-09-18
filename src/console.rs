//! Konsol: iş parçacıklarından gelen satırları toplar, egui'de renkli render eder.

use std::sync::mpsc::{self, Receiver, Sender};

use egui::RichText;

use crate::theme::Theme;

#[derive(Clone, Copy, Debug)]
pub enum LineKind {
    Ok,
    Err,
    Warn,
    Info,
    Dim,
    Big,
    Plain,
    Progress,
}

enum Line {
    Text(LineKind, String),
    Clear,
}

#[derive(Clone, Debug)]
enum UiLine {
    Text(LineKind, String),
    Clear,
}

/// İş parçacıklarına verilen hafif "konsol yazıcı" kolu (klonlanır).
#[derive(Clone)]
pub struct Con {
    tx: Sender<Line>,
}

impl Con {
    fn send(&self, k: LineKind, t: impl Into<String>) {
        let _ = self.tx.send(Line::Text(k, t.into()));
    }
    pub fn ok(&self, t: impl Into<String>) {
        self.send(LineKind::Ok, t);
    }
    pub fn err(&self, t: impl Into<String>) {
        self.send(LineKind::Err, t);
    }
    pub fn warn(&self, t: impl Into<String>) {
        self.send(LineKind::Warn, t);
    }
    pub fn info(&self, t: impl Into<String>) {
        self.send(LineKind::Info, t);
    }
    pub fn dim(&self, t: impl Into<String>) {
        self.send(LineKind::Dim, t);
    }
    pub fn big(&self, t: impl Into<String>) {
        self.send(LineKind::Big, t);
    }
    pub fn plain(&self, t: impl Into<String>) {
        self.send(LineKind::Plain, t);
    }
    pub fn progress(&self, t: impl Into<String>) {
        self.send(LineKind::Progress, t);
    }
    pub fn clear(&self) {
        let _ = self.tx.send(Line::Clear);
    }
    /// Bölüm başlığı çizgisi.
    pub fn header(&self, t: impl Into<String>) {
        self.big("-".repeat(52));
        self.info(t);
        self.dim("-".repeat(52));
    }
}

pub struct Console {
    pub visible: bool,
    pub autoscroll: bool,
    rx: Receiver<UiLine>,
    lines: Vec<UiLine>,
}

const MAX_LINES: usize = 5000;

impl Console {
    pub fn start() -> (Console, Con) {
        let (tx, rx) = mpsc::channel::<Line>();
        let (utx, urx) = mpsc::channel::<UiLine>();
        // İletim ipliği: worker satırlarını UI kuyruğuna taşır (UI ipliğini bekletmez).
        std::thread::spawn(move || {
            while let Ok(l) = rx.recv() {
                let ui = match l {
                    Line::Text(k, t) => UiLine::Text(k, t),
                    Line::Clear => UiLine::Clear,
                };
                if utx.send(ui).is_err() {
                    break;
                }
            }
        });
        (
            Console {
                visible: true,
                autoscroll: true,
                rx: urx,
                lines: Vec::new(),
            },
            Con { tx },
        )
    }

    /// Kuyruktaki beklemedeki satırları iç veriye al.
    pub fn poll(&mut self) {
        while let Ok(l) = self.rx.try_recv() {
            match l {
                UiLine::Clear => self.lines.clear(),
                UiLine::Text(LineKind::Progress, t) => {
                    // Progress satırını aynı satırda güncelle.
                    if matches!(
                        self.lines.last(),
                        Some(UiLine::Text(LineKind::Progress, _))
                    ) {
                        self.lines.pop();
                    }
                    self.lines.push(UiLine::Text(LineKind::Progress, t));
                }
                other => self.lines.push(other),
            }
        }
        if self.lines.len() > MAX_LINES {
            let n = self.lines.len() - MAX_LINES;
            self.lines.drain(..n);
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Konsol gövdesi (başlık satırı dışarıda çizilir).
    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        egui::ScrollArea::vertical()
            .stick_to_bottom(self.autoscroll)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for l in &self.lines {
                    if let UiLine::Text(kind, txt) = l {
                        let col = match *kind {
                            LineKind::Ok => theme.ok,
                            LineKind::Err => theme.err,
                            LineKind::Warn => theme.warn,
                            LineKind::Info => theme.accent,
                            LineKind::Dim => theme.muted,
                            LineKind::Big => theme.accent,
                            LineKind::Plain => theme.text,
                            LineKind::Progress => theme.muted,
                        };
                        let mut rt = RichText::new(txt).color(col).monospace().size(12.5);
                        if matches!(*kind, LineKind::Big) {
                            rt = rt.strong();
                        }
                        ui.label(rt);
                    }
                }
            });
    }
}
