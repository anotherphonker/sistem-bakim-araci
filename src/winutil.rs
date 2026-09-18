//! Windows'a özel yardımlar — SIFIR harici bağımlılık (saf FFI).
//! Linux'ta derleyebilmek için zararsız saplamalara düşer.

#[cfg(windows)]
mod imp {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    pub const MUTEX_NAME: &str = "Global\\SistemBakimAraci_v4_8b2e11";
    pub const ERROR_ALREADY_EXISTS: u32 = 183;
    pub const SW_RESTORE: i32 = 9;
    pub const SW_SHOWNORMAL: i32 = 1;
    pub const MB_OK: u32 = 0;
    pub const MB_ICONWARNING: u32 = 0x0000_0030;
    pub const KEY_READ: u32 = 0x0002_0019;
    pub const REG_DWORD: u32 = 4;
    pub const HKEY_LOCAL_MACHINE: isize = -2147483646; // 0x80000002

    pub type DWORD = u32;
    pub type BOOL = i32;
    pub type HANDLE = *mut core::ffi::c_void;
    pub type HWND = *mut core::ffi::c_void;
    pub type HKEY = *mut core::ffi::c_void;

    #[link(name = "kernel32")]
    extern "system" {
        pub fn CreateMutexW(a: *mut core::ffi::c_void, initial: BOOL, name: *const u16) -> HANDLE;
        pub fn GetLastError() -> DWORD;
        pub fn CloseHandle(h: HANDLE) -> BOOL;
        pub fn GetDiskFreeSpaceExW(
            dir: *const u16,
            free: *mut u64,
            total: *mut u64,
            totalfree: *mut u64,
        ) -> BOOL;
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn FindWindowW(class: *const u16, window: *const u16) -> HWND;
        pub fn ShowWindow(h: HWND, cmd: i32) -> BOOL;
        pub fn SetForegroundWindow(h: HWND) -> BOOL;
        pub fn SetProcessDpiAwarenessContext(value: isize) -> BOOL;
        pub fn SetProcessDPIAware() -> BOOL;
        pub fn MessageBoxW(h: HWND, text: *const u16, caption: *const u16, flags: u32) -> i32;
    }

    #[link(name = "shell32")]
    extern "system" {
        pub fn ShellExecuteW(
            h: HWND,
            op: *const u16,
            file: *const u16,
            params: *const u16,
            dir: *const u16,
            show: i32,
        ) -> usize;
        pub fn IsUserAnAdmin() -> BOOL;
    }

    #[link(name = "advapi32")]
    extern "system" {
        pub fn RegOpenKeyExW(
            key: HKEY,
            sub: *const u16,
            opt: DWORD,
            sam: DWORD,
            out: *mut HKEY,
        ) -> i32;
        pub fn RegQueryValueExW(
            key: HKEY,
            val: *const u16,
            reserved: *mut core::ffi::c_void,
            typ: *mut DWORD,
            data: *mut u8,
            len: *mut DWORD,
        ) -> i32;
        pub fn RegCloseKey(key: HKEY) -> i32;
        pub fn RegSetKeyValueW(
            key: HKEY,
            sub: *const u16,
            val: *const u16,
            typ: DWORD,
            data: *const u8,
            len: DWORD,
        ) -> i32;
    }

    pub fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn list2cmdline(args: &[String]) -> Vec<u16> {
        let mut out = String::new();
        for (i, a) in args.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            if a.contains(' ') || a.contains('"') || a.is_empty() {
                out.push('"');
                out.push_str(&a.replace('"', "\\\""));
                out.push('"');
            } else {
                out.push_str(a);
            }
        }
        wide(&out)
    }
}

/// Tek örneklik koruması: adlandırılmış mutex'i süreç ömrü boyunca tutar.
/// `None` = başka bir örnek zaten açık.
#[cfg(windows)]
pub struct SingleInstance {
    handle: imp::HANDLE,
}

#[cfg(windows)]
impl SingleInstance {
    pub fn acquire() -> Option<Self> {
        use imp::*;
        let name = wide(MUTEX_NAME);
        unsafe {
            let h = CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr());
            if h.is_null() {
                // Mutex alınamadı (hata); yine de devam et (hatalı "zaten açık" yok).
                return Some(Self { handle: h });
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(h);
                None
            } else {
                Some(Self { handle: h })
            }
        }
    }
}

#[cfg(windows)]
impl Drop for SingleInstance {
    fn drop(&mut self) {
        use imp::*;
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

#[cfg(not(windows))]
pub struct SingleInstance;

#[cfg(not(windows))]
impl SingleInstance {
    pub fn acquire() -> Option<Self> {
        Some(SingleInstance)
    }
}

#[cfg(windows)]
pub fn bring_to_front(title: &str) {
    use imp::*;
    unsafe {
        let t = wide(title);
        let h = FindWindowW(std::ptr::null(), t.as_ptr());
        if !h.is_null() {
            ShowWindow(h, SW_RESTORE);
            SetForegroundWindow(h);
        }
    }
}

#[cfg(windows)]
pub fn msgbox_warn(caption: &str, text: &str) {
    use imp::*;
    unsafe {
        let c = wide(caption);
        let t = wide(text);
        MessageBoxW(std::ptr::null_mut(), t.as_ptr(), c.as_ptr(), MB_OK | MB_ICONWARNING);
    }
}

/// Per-Monitor V2 DPI farkındalığı (denersek geriye eski API'ye düşer).
/// Win10 1607+ için manifest gömmeden programatik eşdeğer.
#[cfg(windows)]
pub fn enable_per_monitor_dpi() {
    use imp::*;
    unsafe {
        // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = -4
        let v2: isize = -4;
        if SetProcessDpiAwarenessContext(v2) == 0 {
            let _ = SetProcessDPIAware();
        }
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn enable_per_monitor_dpi() {}

#[cfg(windows)]
pub fn is_admin() -> bool {
    unsafe { imp::IsUserAnAdmin() != 0 }
}

/// Gerekirse "runas" ile yeniden başlat; `true` → mevcut süreç çıkmalı.
#[cfg(windows)]
pub fn elevate_if_needed() -> bool {
    if is_admin() {
        return false;
    }
    let exe = match std::env::current_exe() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => return false,
    };
    let params: Vec<String> = std::env::args().skip(1).collect();
    let exew = imp::wide(&exe);
    let runas = imp::wide("runas");
    let paramsw = imp::list2cmdline(&params);
    let ret = unsafe {
        imp::ShellExecuteW(
            std::ptr::null_mut(),
            runas.as_ptr(),
            exew.as_ptr(),
            paramsw.as_ptr(),
            std::ptr::null(),
            imp::SW_SHOWNORMAL,
        )
    };
    (ret as i64) > 32
}

#[cfg(windows)]
pub fn reg_get_string(subkey: &str, value: &str) -> Option<String> {
    use imp::*;
    let sub = wide(subkey);
    let val = wide(value);
    let mut key: HKEY = std::ptr::null_mut();
    unsafe {
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE as HKEY, sub.as_ptr(), 0, KEY_READ, &mut key) != 0 {
            return None;
        }
        let mut ty: DWORD = 0;
        let mut len: DWORD = 0;
        let _ = RegQueryValueExW(
            key,
            val.as_ptr(),
            std::ptr::null_mut(),
            &mut ty,
            std::ptr::null_mut(),
            &mut len,
        );
        if len == 0 {
            RegCloseKey(key);
            return None;
        }
        let mut buf: Vec<u16> = vec![0u16; (len as usize / 2) + 1];
        let mut len2: DWORD = len;
        if RegQueryValueExW(
            key,
            val.as_ptr(),
            std::ptr::null_mut(),
            &mut ty,
            buf.as_mut_ptr() as *mut u8,
            &mut len2,
        ) != 0
        {
            RegCloseKey(key);
            return None;
        }
        RegCloseKey(key);
        let n = (len2 as usize) / 2;
        if n == 0 {
            return None;
        }
        // REG_SZ genelde null-sonlandırıcı dahil okunur; `\0` karakterini ve
        // boşlukları temizle. Aksi halde GUID'lerin sonuna "\0" yapışır ve
        // powercfg/registry yolları "geçersiz parametre" ile başarısız olur.
        let s: String = String::from_utf16_lossy(&buf[..n])
            .trim_matches(|c: char| c == '\0' || c.is_whitespace())
            .to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

#[cfg(windows)]
pub fn lua_enabled() -> bool {
    let v = reg_get_string(
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System",
        "EnableLUA",
    );
    match v {
        Some(s) => s != "0",
        None => true,
    }
}

/// Kayıt defterinden ham bayt (REG_BINARY) okur. Örn. "Update Revision" (mikrokod).
#[cfg(windows)]
pub fn reg_get_bytes(subkey: &str, value: &str) -> Option<Vec<u8>> {
    use imp::*;
    let sub = wide(subkey);
    let val = wide(value);
    let mut key: HKEY = std::ptr::null_mut();
    unsafe {
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE as HKEY, sub.as_ptr(), 0, KEY_READ, &mut key) != 0 {
            return None;
        }
        let mut ty: DWORD = 0;
        let mut len: DWORD = 0;
        let _ = RegQueryValueExW(
            key,
            val.as_ptr(),
            std::ptr::null_mut(),
            &mut ty,
            std::ptr::null_mut(),
            &mut len,
        );
        if len == 0 {
            RegCloseKey(key);
            return None;
        }
        let mut buf: Vec<u8> = vec![0u8; len as usize];
        let mut len2: DWORD = len;
        if RegQueryValueExW(
            key,
            val.as_ptr(),
            std::ptr::null_mut(),
            &mut ty,
            buf.as_mut_ptr(),
            &mut len2,
        ) != 0
        {
            RegCloseKey(key);
            return None;
        }
        RegCloseKey(key);
        buf.truncate(len2 as usize);
        Some(buf)
    }
}

/// Kayıt defterinden 32 bitlik REG_DWORD okur.
#[cfg(windows)]
pub fn reg_get_dword(subkey: &str, value: &str) -> Option<u32> {
    use imp::*;
    let sub = wide(subkey);
    let val = wide(value);
    let mut key: HKEY = std::ptr::null_mut();
    unsafe {
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE as HKEY, sub.as_ptr(), 0, KEY_READ, &mut key) != 0 {
            return None;
        }
        let mut ty: DWORD = 0;
        let mut data: DWORD = 0;
        let mut len: DWORD = 4;
        let ok = RegQueryValueExW(
            key,
            val.as_ptr(),
            std::ptr::null_mut(),
            &mut ty,
            &mut data as *mut DWORD as *mut u8,
            &mut len,
        );
        RegCloseKey(key);
        if ok == 0 && ty == 4 {
            Some(data)
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
pub fn reg_get_dword(_subkey: &str, _value: &str) -> Option<u32> {
    None
}

/// HKLM anahtarına REG_DWORD yazar (yoksa oluşturur). Başarı = true.
#[cfg(windows)]
pub fn reg_set_dword(subkey: &str, value: &str, data: u32) -> bool {
    use imp::*;
    let sub = wide(subkey);
    let val = wide(value);
    unsafe {
        RegSetKeyValueW(
            HKEY_LOCAL_MACHINE as HKEY,
            sub.as_ptr(),
            val.as_ptr(),
            REG_DWORD,
            &data as *const u32 as *const u8,
            4,
        ) == 0
    }
}

#[cfg(windows)]
pub fn free_bytes(dir: &str) -> Option<u64> {
    use imp::*;
    let d = wide(dir);
    let mut free: u64 = 0;
    let mut total: u64 = 0;
    let mut tf: u64 = 0;
    unsafe {
        if GetDiskFreeSpaceExW(d.as_ptr(), &mut free, &mut total, &mut tf) != 0 {
            Some(free)
        } else {
            None
        }
    }
}

// ---------------- Linux (derleme testi) saplamaları ----------------
#[cfg(not(windows))]
pub fn bring_to_front(_title: &str) {}
#[cfg(not(windows))]
pub fn msgbox_warn(_caption: &str, _text: &str) {}
#[cfg(not(windows))]
pub fn is_admin() -> bool {
    true
}
#[cfg(not(windows))]
pub fn elevate_if_needed() -> bool {
    false
}
#[cfg(not(windows))]
pub fn reg_set_dword(_subkey: &str, _value: &str, _data: u32) -> bool {
    false
}

#[cfg(not(windows))]
pub fn reg_get_string(_subkey: &str, _value: &str) -> Option<String> {
    None
}
#[cfg(not(windows))]
pub fn lua_enabled() -> bool {
    true
}
#[cfg(not(windows))]
pub fn reg_get_bytes(_subkey: &str, _value: &str) -> Option<Vec<u8>> {
    None
}
#[cfg(not(windows))]
pub fn free_bytes(_dir: &str) -> Option<u64> {
    None
}
