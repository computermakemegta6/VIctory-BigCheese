@echo off
rem VIctory-BigCheese-AMD: eikkapine's victory-amd with VIctory-BigCheese's changes,
rem tuned for a PC with 16 GB of RAM. With 32 GB or more, also try Play.cmd
rem (victory-amd's normal settings) and keep the faster one.
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

title VIctory-BigCheese-AMD
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
for /f "usebackq delims=" %%D in (`powershell -NoProfile -STA -Command "Add-Type -AssemblyName System.Windows.Forms; $o = New-Object System.Windows.Forms.Form -Property @{TopMost = $true; ShowInTaskbar = $false}; $d = New-Object System.Windows.Forms.FolderBrowserDialog; $d.Description = 'Choose your game folder (the folder with game_scarlett_master.exe)'; $d.ShowNewFolderButton = $false; if ($d.ShowDialog($o) -eq 'OK') { $d.SelectedPath }; $o.Dispose()" 2^>nul`) do set "VICTORY_GAME_DIR=%%D"
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
rem victory-amd's shader cache and saves: the shader translator is the same,
rem so its cache stays good, and the saves are shared with it.
if not defined VICTORY_CACHE_DIR set "VICTORY_CACHE_DIR=%LOCALAPPDATA%\victory-amd\cache"
if not defined VICTORY_DATA_DIR set "VICTORY_DATA_DIR=%LOCALAPPDATA%\victory-amd\data"
if not exist "%VICTORY_CACHE_DIR%" mkdir "%VICTORY_CACHE_DIR%"
if not exist "%VICTORY_DATA_DIR%" mkdir "%VICTORY_DATA_DIR%"

rem The console model the game sees: series-s gives it about 7.6 GB, which a
rem 16 GB PC needs (series-x: about 13.5). set VICTORY_CONSOLE=series-x before
rem this file to use it anyway.
if not defined VICTORY_CONSOLE set "VICTORY_CONSOLE=series-s"

rem --- 16 GB of RAM (measured on cool_fox, which shares this memory code) ---
rem Buffers the CPU fills go in the card's memory (needs Resizable BAR / Smart
rem Access Memory on) instead of the PC's memory; at 1024 the game ran out of
rem memory in the prologue, at 6144 it ran 15 minutes without a crash.
if not defined VICTORY_WINDOWS_BAR_MB set "VICTORY_WINDOWS_BAR_MB=6144"
rem Copies of the game's buffers kept in the card's memory, in MB (default 2048
rem on a 16 GB card, which filled in the prologue and spilled into PC memory).
if not defined VICTORY_CARD_BUFFERS_MB set "VICTORY_CARD_BUFFERS_MB=4096"
rem Smaller bursts of upload memory when a new scene loads.
if not defined VICTORY_TEXTURE_STAGE_MB set "VICTORY_TEXTURE_STAGE_MB=64"
rem Texture uploads not sent ahead (the installer has 1: up to 96 MB more in flight).
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=0"
rem Staging memory for the game's file streaming (default 256 MB).
if not defined VICTORY_DSTORAGE_STAGING_MB set "VICTORY_DSTORAGE_STAGING_MB=64"
rem How full the card's memory may get, in percent, before those buffers go into
rem the PC's memory (default 80). At 80 the open world fell to about 10 real fps
rem as Windows paged; at 90 it held 17. Lower it if the card runs out.
if not defined VICTORY_WINDOW_CARD_PERCENT set "VICTORY_WINDOW_CARD_PERCENT=90"

rem --- victory-amd's installer settings (eikkapine) ---
if not defined VICTORY_LOG set "VICTORY_LOG=info"
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
if not defined VICTORY_TITLE_PRIORITIES set "VICTORY_TITLE_PRIORITIES=0"
if not defined VICTORY_CARD_AWAKE set "VICTORY_CARD_AWAKE=1"
if not defined VICTORY_SHOW_AFTER set "VICTORY_SHOW_AFTER=1"
if not defined VICTORY_AMBIENT set "VICTORY_AMBIENT=0.8"
if not defined VICTORY_AMBIENT_REFLECT set "VICTORY_AMBIENT_REFLECT=1.5,0.35"
if not defined VICTORY_D3D12_DRED set "VICTORY_D3D12_DRED=light"
rem One compute shader (f16876929fa64f9f, met leaving the prison) crashed AMD's
rem shader compiler; it is skipped (never made, early or at a draw). Set
rem VICTORY_SKIP_PIPELINES=none to draw it.
if not defined VICTORY_SKIP_PIPELINES set "VICTORY_SKIP_PIPELINES=f16876929fa64f9f"
if not defined VICTORY_THIN set "VICTORY_THIN=1"
if not defined VICTORY_LATE_WAIT_LISTS set "VICTORY_LATE_WAIT_LISTS=256"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=8"
if not defined VICTORY_PORT_CONSTANTS set "VICTORY_PORT_CONSTANTS=1"
if not defined VICTORY_PORT_ALL set "VICTORY_PORT_ALL=1"
if not defined VICTORY_WINDOW_KB set "VICTORY_WINDOW_KB=512"
if not defined VICTORY_CARD_BUFFERS_MIN_MB set "VICTORY_CARD_BUFFERS_MIN_MB=1"
if not defined VICTORY_PORT_GUARD set "VICTORY_PORT_GUARD=1"
if not defined VICTORY_PORT_JOURNAL set "VICTORY_PORT_JOURNAL=1"
rem (VIctory-BigCheese) The game's 30 fps cap lifted to 60: menus and light
rem scenes run at up to 60, and gameplay as fast as the PC can (under 60 on a
rem 16 GB PC, so it is never held back). victory-amd's installer has 33.333 and
rem VICTORY_FPS_CAP=30: set those before this file to lock everything to 30.
if not defined VICTORY_FRAME_INTERVAL_MS set "VICTORY_FRAME_INTERVAL_MS=16.667"
if not defined VICTORY_FPS_CAP set "VICTORY_FPS_CAP=60"
if not defined VICTORY_PIPELINE_WAIT_MS set "VICTORY_PIPELINE_WAIT_MS=50"
if not defined VICTORY_PC_CONTROLS set "VICTORY_PC_CONTROLS=1"

rem AMD FidelityFX SDK DLLs (FSR 4 on an RX 9000 card), from the fsr (or
rem fidelityfx-sdk) folder next to this file. The installer's "BEST" launcher also sets
rem VICTORY_FSR_4K=performance (a 3840x2160 output from 1920x1080): left off
rem here, for a 1080p screen and the card's memory.
if not defined VICTORY_FIDELITYFX_DIR if exist "%~dp0fsr\amd_fidelityfx_upscaler_dx12.dll" set "VICTORY_FIDELITYFX_DIR=%~dp0fsr"
if not defined VICTORY_FIDELITYFX_DIR if exist "%~dp0fidelityfx-sdk\amd_fidelityfx_upscaler_dx12.dll" set "VICTORY_FIDELITYFX_DIR=%~dp0fidelityfx-sdk"
rem FSR 4 frame generation (the installer's "FSR4 + Frame Gen 4" launcher): a
rem generated frame between each two real ones. It doubles what you see, not
rem how fast the game runs. 1 is FSR 3's on older cards; 0 turns it off.
if not defined VICTORY_FIDELITYFX_FRAMEGEN set "VICTORY_FIDELITYFX_FRAMEGEN=4"


rem victory-amd's list of the pipelines the game needs first (faster first
rem loading), put in the cache once.
if exist "%~dp0early-pipelines.txt" if not exist "%VICTORY_CACHE_DIR%\pipelines\early-pipelines.txt" (
  if not exist "%VICTORY_CACHE_DIR%\pipelines" mkdir "%VICTORY_CACHE_DIR%\pipelines"
  copy /y "%~dp0early-pipelines.txt" "%VICTORY_CACHE_DIR%\pipelines\early-pipelines.txt" >nul
)

rem ---------------------------------------------------------------- shaders, once
rem Translates all of the game's shaders ahead (a few minutes, every CPU core),
rem so that they are not translated while playing.
set "mark=%VICTORY_CACHE_DIR%\shaders-done.txt"
if exist "%mark%" goto :play
if not exist "%VICTORY_GAME_DIR%\shaders.rpf" goto :play
echo First start: translating the game's shaders ahead. This happens once and takes a few minutes.
"%runner%" shaders "%VICTORY_GAME_DIR%\shaders.rpf" "%VICTORY_GAME_DIR%\%exe_name%"
if errorlevel 1 (
  echo Translating ahead failed. The shaders will be translated while playing instead.
) else (
  >"%mark%" echo The game's shaders were translated ahead.
)

:play
echo Starting the game (%VICTORY_CONSOLE%). Keep this window open; closing it quits the game. F3: FPS and VRAM.
"%runner%" run "%VICTORY_GAME_DIR%\%exe_name%" --console %VICTORY_CONSOLE% --execute
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
