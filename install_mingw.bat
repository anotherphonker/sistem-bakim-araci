@echo off
setlocal EnableExtensions
chcp 65001 >nul
title MinGW-w64 kurulumu (tek seferlik)
cd /d "%~dp0"

echo.
echo  ==================================================
echo    mingw-w64 KURULUMU  (build.bat icin tek seferlik)
echo  ==================================================
echo.

where winget >nul 2>&1
if errorlevel 1 goto NO_WINGET

echo   winget ile WinLibs kuruluyor...
echo   (kurulum penceresi kendiliginden kapanabilir, normaldir)
echo.
winget install -e --id BrechtSanders.WinLibs.POSIX.UCRT --accept-source-agreements --accept-package-agreements
if errorlevel 1 goto WINGET_FAIL

echo.
echo   Kurulum bitti. Onemli:
echo   Bu pencereyi kapat, YENI bir Komut Istemi ac,
echo   sonra build.bat calistir.
echo   (PATH ayari yalnizca yeni pencerelerde gecerli olur.)
echo.
goto END

:NO_WINGET
echo.
echo   [RIZIK] winget bulunamadi. Elle kurulum:
echo     1. https://winlibs.com  sitesine git.
echo     2. En guncel "x86_64-posix-seh" paketini indir (mingw64).
echo     3. .7z dosyasini 7-Zip ile ac.
echo     4. Icindeki mingw64\bin klasorunu PATH'e ekle:
echo        Baslat - "ortam degiskenleri" ara -
echo        Ortam Degiskenleri - Path - Duzenle - Yeni -
echo        ...\mingw64\bin  yolunu yapistir.
echo     5. Yeni bir Komut Istemi ac, build.bat calistir.
echo.
goto END

:WINGET_FAIL
echo.
echo   [RIZIK] winget kurulumu basaramadi.
echo   Elle kurulum icin ayni adimlar gecerli:
echo   https://winlibs.com  -  x86_64-posix-seh
echo.
goto END

:END
echo.
pause
