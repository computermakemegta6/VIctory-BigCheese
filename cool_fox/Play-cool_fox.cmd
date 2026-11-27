@echo off
rem cool_fox (a VIctory fork) launcher, plain: Series S mode with the emulator's
rem own defaults and no tuning, so any build of victory.exe can be run with it.
rem The tuned launcher is Play-cool_fox-16GB-RAM.cmd: it sets its settings, then
rem calls this.
rem
rem Start it by dragging your game folder (the folder with game_scarlett_master.exe)
rem onto this file, or run:  Play-cool_fox.cmd "D:\path\to\your\game"
rem The folder is remembered in game-folder.txt, so after that a double-click is enough.
rem Every setting below can be overridden by setting the variable before starting this file.
setlocal EnableExtensions DisableDelayedExpansion
cd /d "%~dp0"
set "runner=%~dp0victory.exe"
if not exist "%runner%" (
  echo victory.exe is missing next to this file. Extract the whole zip first.
  goto :fail
)

rem The game folder: the one given or dropped on this file, else the one remembered.
if not "%~1"=="" set "VICTORY_GAME_DIR=%~1"
if /i "%~x1"==".exe" set "VICTORY_GAME_DIR=%~dp1"
if not defined VICTORY_GAME_DIR if exist "%~dp0game-folder.txt" set /p VICTORY_GAME_DIR=<"%~dp0game-folder.txt"
if not defined VICTORY_GAME_DIR goto :no_game
if "%VICTORY_GAME_DIR:~-1%"=="\" set "VICTORY_GAME_DIR=%VICTORY_GAME_DIR:~0,-1%"
if not exist "%VICTORY_GAME_DIR%\game_scarlett_master.exe" goto :no_game
>"%~dp0game-folder.txt" echo(%VICTORY_GAME_DIR%

rem The shader cache and the saves, kept apart from other VIctory builds.
if not defined VICTORY_CACHE_DIR set "VICTORY_CACHE_DIR=%LOCALAPPDATA%\cool_fox\cache"
if not defined VICTORY_DATA_DIR set "VICTORY_DATA_DIR=%LOCALAPPDATA%\cool_fox\data"
if not exist "%VICTORY_CACHE_DIR%" mkdir "%VICTORY_CACHE_DIR%"
if not exist "%VICTORY_DATA_DIR%" mkdir "%VICTORY_DATA_DIR%"

rem Only warnings in the console (the emulator's own default logs much more).
if not defined VICTORY_LOG if not defined RUST_LOG set "VICTORY_LOG=warn"

rem The game's own 30 fps cap lifted (up to 60). VICTORY_FRAME_INTERVAL_MS=33.333 plays at 30.
if not defined VICTORY_FRAME_INTERVAL_MS set "VICTORY_FRAME_INTERVAL_MS=16.667"

rem Once per build: translate all of the game's shaders ahead (a few minutes on a fast
rem CPU), so that they are not translated while playing.
set "mark=%VICTORY_CACHE_DIR%\cool_fox-30e54b4-shaders-done.txt"
if exist "%mark%" goto :play
if not exist "%VICTORY_GAME_DIR%\shaders.rpf" goto :play
echo First start: translating the game's shaders ahead. This happens once and takes a few minutes.
"%runner%" shaders "%VICTORY_GAME_DIR%\shaders.rpf"
if errorlevel 1 (
  echo Translating ahead failed. The shaders will be translated while playing instead.
) else (
  >"%mark%" echo The game's shaders were translated ahead by cool_fox 30e54b4.
)

:play
echo Starting the game. Closing the game window or this window quits.
"%runner%" run "%VICTORY_GAME_DIR%\game_scarlett_master.exe" --console series-s --execute
set "code=%errorlevel%"
if not "%code%"=="0" (
  echo The game ended with code %code%. The lines above may say why.
  pause
)
exit /b %code%

:no_game
echo Could not find game_scarlett_master.exe.
echo Drag your own game folder (the folder that contains game_scarlett_master.exe)
echo onto Play-cool_fox.cmd, or run:  Play-cool_fox.cmd "D:\path\to\your\game"
:fail
pause
exit /b 2
