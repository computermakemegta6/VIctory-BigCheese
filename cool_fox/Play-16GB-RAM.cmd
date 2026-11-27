@echo off
rem VIctory-BigCheese: starts the game, tuned for a PC with 16 GB of RAM.
rem Double-click it. If GAME_DIR below is empty, the first start asks for your
rem game folder (the one with game_scarlett_master.exe) and remembers it in
rem game-folder.txt. Every setting below can be overridden by setting the
rem variable before starting this file.
setlocal EnableExtensions DisableDelayedExpansion

rem ======================= SET YOUR GAME HERE (or leave empty) =======================
rem GAME_DIR: the folder with game_scarlett_master.exe and shaders.rpf in it,
rem   for example  set "GAME_DIR=D:\Games\GTA6"
rem GAME_EXE: only if the game's executable is not game_scarlett_master.exe in
rem   GAME_DIR: its full path, for example  set "GAME_EXE=D:\Games\GTA6\game_scarlett_master.exe"
set "GAME_DIR="
set "GAME_EXE="
rem ==================================================================================

title VIctory-BigCheese
cd /d "%~dp0"
set "runner=%~dp0victory.exe"
if not exist "%runner%" (
  echo victory.exe is missing next to this file. Extract the whole zip first.
  goto :fail
)

rem ---------------------------------------------------------------- game folder
rem Dragged onto this file, GAME_EXE / GAME_DIR above, remembered, next to this
rem file, or one folder up.
set "exe_name=game_scarlett_master.exe"
if not "%~1"=="" set "VICTORY_GAME_DIR=%~1"
if /i "%~x1"==".exe" (
  set "VICTORY_GAME_DIR=%~dp1"
  set "exe_name=%~nx1"
)
if not defined VICTORY_GAME_DIR if defined GAME_EXE call :from_game_exe
if not defined VICTORY_GAME_DIR if defined GAME_DIR call :from_game_dir
if defined VICTORY_GAME_DIR (
  call :check_game && goto :game_found
  echo There is no %exe_name% in "%VICTORY_GAME_DIR%".
  set "VICTORY_GAME_DIR="
)
if exist "%~dp0game-folder.txt" set /p VICTORY_GAME_DIR=<"%~dp0game-folder.txt"
if not defined VICTORY_GAME_DIR if exist "%~dp0game_scarlett_master.exe" set "VICTORY_GAME_DIR=%~dp0"
if not defined VICTORY_GAME_DIR if exist "%~dp0..\game_scarlett_master.exe" for %%G in ("%~dp0..") do set "VICTORY_GAME_DIR=%%~fG"
call :check_game && goto :game_found

echo First start: choose your game folder (the folder with game_scarlett_master.exe).
:ask
set "VICTORY_GAME_DIR="
for /f "usebackq delims=" %%D in (`powershell -NoProfile -STA -Command "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.FolderBrowserDialog; $d.Description = 'Choose your game folder (the folder with game_scarlett_master.exe)'; $d.ShowNewFolderButton = $false; if ($d.ShowDialog() -eq 'OK') { $d.SelectedPath }" 2^>nul`) do set "VICTORY_GAME_DIR=%%D"
if not defined VICTORY_GAME_DIR (
  echo No folder chosen. You can also type or paste the folder's path here.
  set /p "VICTORY_GAME_DIR=Game folder (or Enter to quit): "
)
if not defined VICTORY_GAME_DIR goto :fail
set "VICTORY_GAME_DIR=%VICTORY_GAME_DIR:"=%"
call :check_game && goto :game_found
echo There is no %exe_name% in "%VICTORY_GAME_DIR%". Choose the folder that has it.
goto :ask

:game_found
>"%~dp0game-folder.txt" echo(%VICTORY_GAME_DIR%
echo Game folder: %VICTORY_GAME_DIR%

rem ---------------------------------------------------------------- where things are kept
rem The shader cache and the saves, kept apart from other VIctory builds.
if not defined VICTORY_CACHE_DIR set "VICTORY_CACHE_DIR=%LOCALAPPDATA%\cool_fox\cache"
if not defined VICTORY_DATA_DIR set "VICTORY_DATA_DIR=%LOCALAPPDATA%\cool_fox\data"
if not exist "%VICTORY_CACHE_DIR%" mkdir "%VICTORY_CACHE_DIR%"
if not exist "%VICTORY_DATA_DIR%" mkdir "%VICTORY_DATA_DIR%"

rem ---------------------------------------------------------------- 16 GB of RAM
rem Buffers the CPU fills go in the card's memory (needs Resizable BAR / Smart
rem Access Memory on; without it nothing changes) instead of the PC's memory.
rem At 1024 the game ran out of memory in the prologue; at 6144 it ran 15
rem minutes without a crash.
if not defined VICTORY_WINDOWS_BAR_MB set "VICTORY_WINDOWS_BAR_MB=6144"
rem How full the card's memory may get, in percent, before those buffers go into
rem the PC's memory (default 80). At 80 the open world fell to about 10 real fps
rem as Windows paged; at 90 it held 17. Lower it if the card runs out.
if not defined VICTORY_WINDOW_CARD_PERCENT set "VICTORY_WINDOW_CARD_PERCENT=90"
rem Copies of the game's buffers kept in the card's memory, in MB (default 2048
rem on a 16 GB card, which filled in the prologue and spilled into PC memory).
if not defined VICTORY_CARD_BUFFERS_MB set "VICTORY_CARD_BUFFERS_MB=4096"
rem Smaller bursts of upload memory when a new scene loads.
if not defined VICTORY_TEXTURE_STAGE_MB set "VICTORY_TEXTURE_STAGE_MB=64"
rem Texture uploads not sent ahead of the frames that need them.
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=0"
rem Staging memory for the game's file streaming (default 256 MB).
if not defined VICTORY_DSTORAGE_STAGING_MB set "VICTORY_DSTORAGE_STAGING_MB=64"

rem ---------------------------------------------------------------- tuning
rem cool_fox's pareto-residency settings (VICTORY_LEVELS_ON_DEMAND stays off:
rem with it, every texture took all of its sizes on the card).
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
if not defined VICTORY_THIN set "VICTORY_THIN=1"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=8"
if not defined VICTORY_CARD_BUFFERS_MIN_KB set "VICTORY_CARD_BUFFERS_MIN_KB=64"

rem Light where the game's ray tracing would put it: without it, rooms and the
rem shadowed side of faces are black (the ray tracing is off).
if not defined VICTORY_AMBIENT set "VICTORY_AMBIENT=0.8"

rem AMD FSR (any DX12 card), used when its DLLs are in a folder named fsr next
rem to this file (see README.txt).
if not defined VICTORY_FSR_DLL if exist "%~dp0fsr\amd_fidelityfx_upscaler_dx12.dll" set "VICTORY_FSR_DLL=%~dp0fsr\amd_fidelityfx_upscaler_dx12.dll"
rem AMD FSR frame generation: a generated frame between each two real ones
rem (14.6 real fps shown as 23.7 on a 16 GB PC with an RX 9060 XT). It doubles
rem what you see, not how fast the game runs. VICTORY_FRAMEGEN=0 turns it off.
if not defined VICTORY_FRAMEGEN set "VICTORY_FRAMEGEN=1"

rem One compute shader (f16876929fa64f9f, met leaving the prison) crashes AMD's
rem shader compiler. Skipped; nothing wrong showed in the frames. Set
rem VICTORY_SKIP_PIPELINES=none to draw it (cards other than AMD's may not need it).
if not defined VICTORY_SKIP_PIPELINES set "VICTORY_SKIP_PIPELINES=f16876929fa64f9f"

rem Only warnings in this window.
if not defined VICTORY_LOG if not defined RUST_LOG set "VICTORY_LOG=warn"
rem The game's own 30 fps cap lifted (up to 60). VICTORY_FRAME_INTERVAL_MS=33.333 plays at 30.
if not defined VICTORY_FRAME_INTERVAL_MS set "VICTORY_FRAME_INTERVAL_MS=16.667"

rem ---------------------------------------------------------------- shaders, once
rem Translates all of the game's shaders ahead (a few minutes on a fast CPU), so
rem that they are not translated while playing.
set "mark=%VICTORY_CACHE_DIR%\cool_fox-30e54b4-shaders-done.txt"
if exist "%mark%" goto :play
if not exist "%VICTORY_GAME_DIR%\shaders.rpf" goto :play
echo First start: translating the game's shaders ahead. This happens once and takes a few minutes.
"%runner%" shaders "%VICTORY_GAME_DIR%\shaders.rpf"
if errorlevel 1 (
  echo Translating ahead failed. The shaders will be translated while playing instead.
) else (
  >"%mark%" echo The game's shaders were translated ahead.
)

:play
echo Starting the game. Keep this window open; closing it quits the game.
"%runner%" run "%VICTORY_GAME_DIR%\%exe_name%" --console series-s --execute
set "code=%errorlevel%"
if not "%code%"=="0" (
  echo The game ended with code %code%. The lines above may say why.
  pause
)
exit /b %code%

:from_game_exe
set "GAME_EXE=%GAME_EXE:"=%"
for %%E in ("%GAME_EXE%") do (
  set "VICTORY_GAME_DIR=%%~dpE"
  set "exe_name=%%~nxE"
)
exit /b 0

:from_game_dir
set "VICTORY_GAME_DIR=%GAME_DIR:"=%"
exit /b 0

:check_game
if not defined VICTORY_GAME_DIR exit /b 1
if "%VICTORY_GAME_DIR:~-1%"=="\" set "VICTORY_GAME_DIR=%VICTORY_GAME_DIR:~0,-1%"
if exist "%VICTORY_GAME_DIR%\%exe_name%" exit /b 0
exit /b 1

:fail
pause
exit /b 2
