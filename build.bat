@echo off
setlocal EnableExtensions
chcp 65001 >nul
title Sistem Bakim Araci - EXE Derleyici
cd /d "%~dp0"

echo.
echo  ==================================================
echo    SISTEM BAKIM ARACI  (Rust)  -  .EXE DERLEYICI
echo  ==================================================
echo.

REM --- 1) Rust var mi? ---
where cargo >nul 2>&1
if errorlevel 1 goto NO_RUST

echo   [1/4] Rust bulundu:
cargo --version
echo.

REM --- 2) GNU alet zinciri (Visual Studio GEREKMEZ) ---
echo   [2/4] GNU alet zinciri ayarlaniyor...
echo         (MSVC varsayilanini GNU'ya ceviriyoruz)
echo         Ilk sefer yukleme yapar, internet gerekir.
echo.
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
echo.

REM --- 3) Linker (gcc) kontrolu ---
where gcc.exe >nul 2>&1
if errorlevel 1 goto NO_MINGW
echo   [3/4] Linker (mingw-w64) hazir.
echo.

REM --- 4) Derle ---
echo   [4/4] Derleniyor... (ilk derleme 5-15 dakika surer, sabir)
echo        Ilk derlemede ekran donmus gibi durabilir, kapatma.
echo.
cargo build --release --target x86_64-pc-windows-gnu
if errorlevel 1 goto BUILD_FAIL
echo.

REM --- Cikti ---
if not exist "Cikti" mkdir "Cikti"
if exist "target\x86_64-pc-windows-gnu\release\SistemBakimAraci.exe" goto COPY_OK
echo   [HATA] exe olusmadi. Yukaridaki hatalara bak.
goto END

:COPY_OK
copy /y "target\x86_64-pc-windows-gnu\release\SistemBakimAraci.exe" "Cikti\" >nul
echo.
echo  ==================================================
echo    BASARILI!
echo    Hazir exe:  Cikti\SistemBakimAraci.exe
echo    Bunu istedigin PC'ye kopyala, cift tikla calisir.
echo  ==================================================
echo.
echo   Cikti klasoru aciliyor...
start "" "%CD%\Cikti"
goto END

:NO_RUST
echo.
echo   [HATA] Rust bulunamadi.
echo.
echo   Tek seferlik kurulum:
echo     1. https://rustup.rs  adresini ac.
echo     2. "Download rustup-init.exe" indir ve calistir.
echo     3. Kurulumda 1 tusuna bas (varsayilan secenekler yeterli).
echo     4. Bu pencereyi kapat, YENI bir pencere ac,
echo        sonra build.bat'i tekrar calistir.
echo.
goto END

:NO_MINGW
echo.
echo   [HATA] mingw-w64 (gcc) bulunamadi.
echo.
echo   Tek seferlik: install_mingw.bat dosyasini calistir.
echo   Kurulum bitince YENI bir Komut Istemi ac,
echo   sonra build.bat'i tekrar calistir.
echo   (PATH ayari yalnizca yeni pencerelerde gecerli olur.)
echo.
goto END

:BUILD_FAIL
echo.
echo   [HATA] Derleme basarisiz oldu.
echo         Yukaridaki kirmizi hata satirlarini bana at.
echo.
where gcc.exe >nul 2>&1
if errorlevel 1 goto NO_MINGW
goto END

:END
echo.
pause
