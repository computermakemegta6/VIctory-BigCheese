@echo off
rem The tuned settings of Play-cool_fox-16GB-RAM.cmd, kept apart. Not to be
rem started on its own: it calls this, then Play-cool_fox.cmd. Every setting can
rem be overridden by setting the variable before starting the launcher. No
rem setlocal here: what it sets must reach the launcher that called it.

rem cool_fox's pareto-residency settings, except VICTORY_LEVELS_ON_DEMAND, which
rem stays off: with it on, every texture took all of its sizes on the graphics
rem card (the card filled up, then textures waited).
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
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=1"
if not defined VICTORY_CARD_BUFFERS_MIN_KB set "VICTORY_CARD_BUFFERS_MIN_KB=64"

rem Light where the game's ray tracing would put it: without it, rooms and the
rem shadowed side of faces are black (the ray tracing is off; see the README).
if not defined VICTORY_AMBIENT set "VICTORY_AMBIENT=0.8"

rem AMD FSR (any DX12 card): the PC upscaler in place of the game's own, from the
rem fsr folder next to the launchers.
if not defined VICTORY_FSR_DLL if exist "%~dp0fsr\amd_fidelityfx_upscaler_dx12.dll" set "VICTORY_FSR_DLL=%~dp0fsr\amd_fidelityfx_upscaler_dx12.dll"

rem AMD FSR frame generation: a generated frame between each two real ones (on a
rem 16 GB PC with an RX 7900-class card: 14.6 real fps shown as 23.7). It doubles
rem what you see, not how fast the game runs or answers. VICTORY_FRAMEGEN=0 turns it off.
if not defined VICTORY_FRAMEGEN set "VICTORY_FRAMEGEN=1"

rem One compute shader (f16876929fa64f9f, met leaving the prison) crashes AMD's
rem shader compiler (amdxc64.dll) when its pipeline is made. Skipped, nothing
rem wrong showed in the frames. Set VICTORY_SKIP_PIPELINES=none to draw it (cards
rem other than AMD's may not need the skip).
if not defined VICTORY_SKIP_PIPELINES set "VICTORY_SKIP_PIPELINES=f16876929fa64f9f"
