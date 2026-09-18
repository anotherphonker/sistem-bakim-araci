use std::path::Path;
use std::process::Command;

fn main() {
    // build.rs HER ZAMAN host üzerinde çalışır; hedefi env'den öğren.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();

    // ------------------------------------------------------------------
    // 1) İKON + VERSİYON BİLGİSİ (kendi .rc'si, kendi .o'su)
    // ------------------------------------------------------------------
    let icon = rc_path(&format!("{manifest_dir}/assets/icon.ico"));
    if !Path::new(&icon).is_file() {
        println!("cargo:warning=assets/icon.ico bulunamadı; exe ikonu gömülemedi");
    } else {
        let rc_path_f = format!("{out_dir}/resource.rc");
        let rc_content = format!(
            "APP_ICON ICON \"{icon}\"\n1 VERSIONINFO\nFILEVERSION 0,1,0,0\nPRODUCTVERSION 0,1,0,0\nBEGIN\n  BLOCK \"StringFileInfo\"\n  BEGIN\n    BLOCK \"041904B0\"\n    BEGIN\n      VALUE \"FileDescription\", \"Sistem Bakim Araci\\0\"\n      VALUE \"ProductName\", \"Sistem Bakim Araci\\0\"\n      VALUE \"FileVersion\", \"0.1.0\\0\"\n      VALUE \"ProductVersion\", \"0.1.0\\0\"\n    END\n  END\n  BLOCK \"VarFileInfo\"\n  BEGIN\n    VALUE \"Translation\", 0x0419, 1200\n  END\nEND\n"
        );
        if std::fs::write(&rc_path_f, rc_content).is_err() {
            println!("cargo:warning=resource.rc yazılamadı; exe ikonu gömülemedi");
        } else if let Some(obj) = compile_rc(&out_dir, &manifest_dir, &rc_path_f, "resource") {
            println!("cargo:rustc-link-arg={obj}");
        }
    }

    // ------------------------------------------------------------------
    // 2) MANIFEST (requireAdministrator -> yönetici logosu + otomatik UAC)
    //    Ayrı bir .rc/.o olarak eklenir.
    // ------------------------------------------------------------------
    let manifest_src = format!("{manifest_dir}/assets/app.manifest");
    let manifest_xml = format!("{out_dir}/manifest.xml");
    if !Path::new(&manifest_src).is_file() {
        println!("cargo:warning=assets/app.manifest bulunamadı; yönetici manifesti gömülemedi");
    } else if std::fs::copy(&manifest_src, &manifest_xml).is_err() {
        println!("cargo:warning=manifest.xml kopyalanamadı; yönetici manifesti gömülemedi");
    } else {
        let xml_for_rc = rc_path(&manifest_xml);
        let rc_path_m = format!("{out_dir}/manifest.rc");
        let rc_content = format!("1 24 \"{xml_for_rc}\"\n");
        if std::fs::write(&rc_path_m, rc_content).is_err() {
            println!("cargo:warning=manifest.rc yazılamadı; yönetici manifesti gömülemedi");
        } else if let Some(obj) = compile_rc(&out_dir, &manifest_dir, &rc_path_m, "manifest") {
            println!("cargo:rustc-link-arg={obj}");
        }
    }

    // ------------------------------------------------------------------
    // 3) GNU zincirinde gcc, kendi lib klasöründe bulursa otomatik olarak
    //    asInvoker içeren `default-manifest.o` ekler (WinLibs böyle). Bu,
    //    bizim requireAdministrator manifestimizle çakışıp
    //      ld.exe: .rsrc merge failure: multiple non-default manifests
    //    uyarısını doğurur ve asInvoker kazanırsa yönetici logosu yine
    //    çıkmaz. Çözüm: gcc'nin spec dosyasını kopyalayıp yalnızca
    //    `default-manifest.o` enjeksiyonunu çıkarır, `-specs=` ile bağlarız.
    //    Böylece exe'de TEK manifest kalır: bizim requireAdministrator.
    // ------------------------------------------------------------------
    if std::env::var("CARGO_CFG_TARGET_ENV").map(|v| v == "gnu").unwrap_or(false) {
        if let Some(gcc) = pick_any(&[
            "gcc.exe",
            "x86_64-w64-mingw32-gcc.exe",
            "x86_64-w64-mingw32-gcc",
            "gcc",
        ]) {
            // gcc exe yolu bulunamıyorsa bile PATH üzerinden çalışır;
            // yalnızca spec üretebiliyorsak devam et.
            let dumpspecs = Command::new(&gcc).arg("-dumpspecs").output();
            if let Ok(o) = dumpspecs {
                if o.status.success() {
                    let text = String::from_utf8_lossy(&o.stdout).into_owned();
                    if text.contains("default-manifest.o") {
                        let stripped = text.replace(
                            "%{!shared:%:if-exists(default-manifest.o%s)}",
                            "",
                        );
                        let spec_file = format!("{out_dir}/no-default-manifest.spec");
                        if std::fs::write(&spec_file, stripped).is_ok() {
                            println!(
                                "cargo:rustc-link-arg=-specs={}",
                                rc_path(&spec_file)
                            );
                        }
                    }
                }
            }
        }
    }

    println!("cargo:rerun-if-changed=assets/app.manifest");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
}

/// .rc dosyasını windres (GNU) veya rc.exe (MSVC) ile derleyip .o/.res yolunu döndür.
/// Başarısız olursa uyarı basıp None döner.
fn compile_rc(out_dir: &str, manifest_dir: &str, rc_file: &str, name: &str) -> Option<String> {
    if let Some(windres) = pick_any(&[
        "windres.exe",
        "x86_64-w64-mingw32-windres.exe",
        "x86_64-w64-mingw32-windres",
        "windres",
    ]) {
        let obj = format!("{out_dir}/{name}.o");
        let status = Command::new(&windres)
            .args([
                "--output-format=coff",
                "-i",
                rc_file,
                "-o",
                &obj,
                "-I",
                &rc_path(manifest_dir),
            ])
            .output();
        match status {
            Ok(o) if o.status.success() => return Some(obj),
            Ok(o) => println!(
                "cargo:warning=windres {name} derleyemedi: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            ),
            Err(e) => println!("cargo:warning=windres {name} çalıştırılamadı: {e}"),
        }
        return None;
    }

    if let Some(rc) = pick_any(&["rc.exe", "rc"]) {
        let res = format!("{out_dir}/{name}.res");
        let status = Command::new(&rc)
            .args(["/fo", &res, "/I", &rc_path(manifest_dir), rc_file])
            .status();
        match status {
            Ok(s) if s.success() => return Some(res),
            _ => {}
        }
    }

    println!("cargo:warning=windres/rc.exe bulunamadı; {name} gömülemedi");
    None
}

/// .rc dosyası içinde güvenli yol: ters slash escape karakteri olduğu için
/// (ör. \U -> U, \i) Windows yolları ters slasha çevrilir. `C:\a\b` değil
/// `C:/a/b` yazılır; Windows bunu da kabul eder.
fn rc_path(p: &str) -> String {
    p.replace('\\', "/")
}

/// Boş olmayan, dosya sisteminde gerçekten var olan ilk adayı döndür.
/// 1) Çalışma dizini/absolute yol, 2) PATH içindeki dizinler.
fn pick_any(candidates: &[&str]) -> Option<String> {
    for c in candidates {
        if Path::new(c).is_file() {
            return Some(c.to_string());
        }
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            for c in candidates {
                if dir.join(c).is_file() {
                    return Some(c.to_string());
                }
            }
        }
    }
    None
}
