@echo off
rem Translate shaders ahead, then warm the driver's pipeline cache, with Play-cool_fox.cmd's settings.
setlocal EnableExtensions DisableDelayedExpansion
cd /d "%~dp0"
set "runner=%~dp0victory.exe"
if not defined VICTORY_GAME_DIR set /p VICTORY_GAME_DIR=<"%~dp0game-folder.txt"
rem The shader cache and the saves, kept apart from other VIctory builds.
if not defined VICTORY_CACHE_DIR set "VICTORY_CACHE_DIR=%LOCALAPPDATA%\cool_fox\cache"
if not defined VICTORY_DATA_DIR set "VICTORY_DATA_DIR=%LOCALAPPDATA%\cool_fox\data"
if not exist "%VICTORY_CACHE_DIR%" mkdir "%VICTORY_CACHE_DIR%"
if not exist "%VICTORY_DATA_DIR%" mkdir "%VICTORY_DATA_DIR%"

rem cool_fox's pareto-residency settings, as in its own Play-VIctory.cmd, except
rem VICTORY_LEVELS_ON_DEMAND, which stays off: with it on, every texture took all of
rem its sizes on the graphics card (the card filled up, then textures waited).
if not defined VICTORY_AMBIENT_AIR set "VICTORY_AMBIENT_AIR=1.5"
if not defined VICTORY_TEXTURE_NO_WAIT set "VICTORY_TEXTURE_NO_WAIT=1"
if not defined VICTORY_RELAY set "VICTORY_RELAY=2"
if not defined VICTORY_WIDE_BACK set "VICTORY_WIDE_BACK=1"
if not defined VICTORY_CARD_REHASH_RUNS set "VICTORY_CARD_REHASH_RUNS=1"
if not defined VICTORY_JOIN_COPIES set "VICTORY_JOIN_COPIES=1"
if not defined VICTORY_INDIRECT_WINDOW set "VICTORY_INDIRECT_WINDOW=1"
if not defined VICTORY_RELAY_COPY set "VICTORY_RELAY_COPY=1"
if not defined VICTORY_CARD_STEPS set "VICTORY_CARD_STEPS=2"
if not defined VICTORY_INDIRECT_BATCH set "VICTORY_INDIRECT_BATCH=2"
if not defined VICTORY_MAP_THREADS set "VICTORY_MAP_THREADS=1"
if not defined VICTORY_CARD_AWAKE set "VICTORY_CARD_AWAKE=1"
if not defined VICTORY_SHOW_AFTER set "VICTORY_SHOW_AFTER=1"
if not defined VICTORY_AMBIENT set "VICTORY_AMBIENT=0.8"
if not defined VICTORY_THIN set "VICTORY_THIN=1"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=8"
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=1"
if not defined VICTORY_CARD_BUFFERS_MIN_KB set "VICTORY_CARD_BUFFERS_MIN_KB=64"
if not defined VICTORY_LOG if not defined RUST_LOG set "VICTORY_LOG=warn"

rem The game's own 30 fps cap lifted (up to 60). VICTORY_FRAME_INTERVAL_MS=33.333 plays at 30.
if not defined VICTORY_FRAME_INTERVAL_MS set "VICTORY_FRAME_INTERVAL_MS=16.667"

rem Optional: AMD's PC upscaler in place of the game's own (a little faster here, same
rem picture). Put amd_fidelityfx_upscaler_dx12.dll in a folder named fsr next to this file.
if not defined VICTORY_FSR_DLL if exist "%~dp0fsr\amd_fidelityfx_upscaler_dx12.dll" set "VICTORY_FSR_DLL=%~dp0fsr\amd_fidelityfx_upscaler_dx12.dll"

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
echo Warming pipelines (resumes where it stopped).
"%runner%" warm %*
exit /b %errorlevel%

