@echo off
setlocal EnableExtensions
chcp 65001 >nul
title Sistem Bakim Araci - Github Yukleme
cd /d "%~dp0"

where git >nul 2>&1
if errorlevel 1 goto NO_GIT

REM Kimlik (sadece ilk seferde, yoksa ayarla)
git config user.name >nul 2>&1
if errorlevel 1 git config --global user.name "anotherphonker"
git config user.email >nul 2>&1
if errorlevel 1 git config --global user.email "anotherphonker@users.noreply.github.com"

if not exist ".git" git init
git add .
git commit -m "ilk surum" >nul 2>&1
git branch -M main
git remote remove origin >nul 2>&1
git remote add origin https://github.com/anotherphonker/sistem-bakim-araci.git

echo.
echo  Uzak depo ile birlesiyor (github'da README varsa)...
git fetch origin main >nul 2>&1
if not errorlevel 1 git merge origin/main --allow-unrelated-histories -X ours -m "github'dan birleştir" >nul 2>&1

echo.
echo  Yukleniyor (ilk seferde giris ekrani acilabilir)...
git push -u origin main
if errorlevel 1 goto PUSH_FAIL

echo.
echo  ==========================================
echo   BASARILI!
echo   https://github.com/anotherphonker/sistem-bakim-araci
echo  ==========================================
echo.
goto END

:NO_GIT
echo.
echo  [HATA] Git kurulu degil.
echo   1. https://git-scm.com adresinden indirip kur.
echo   2. Kurulumda her seyi varsayilan birak (Next Next).
echo   3. Bu dosyayi tekrar cift tikla.
echo.
goto END

:PUSH_FAIL
echo.
echo  [HATA] Push basarisiz oldu.
echo  Yukaridaki kirmizi/okunmamis hata satirini bana at.
echo.
goto END

:END
echo.
pause
