@echo off
rem cool_fox for a PC with 16 GB of RAM: the tuned settings (cool_fox-tuning.cmd)
rem plus settings that keep less in the PC's memory. Tested on 16 GB of RAM with
rem an RX 7900-class card (16 GB). Any setting can be overridden by setting the
rem variable before starting this file.
setlocal EnableExtensions DisableDelayedExpansion

rem Buffers the CPU fills go in the card's memory (needs Resizable BAR / Smart
rem Access Memory on; without it nothing changes) instead of the PC's memory.
rem The prologue needs about 3.4 GB of them: at 1024 the rest filled the 7 GB
rem of PC memory Windows lets the card use, and the game ran out of memory in
rem 2.5 minutes. At 6144 auto-tests ran 15 minutes without a crash.
if not defined VICTORY_WINDOWS_BAR_MB set "VICTORY_WINDOWS_BAR_MB=6144"

rem How full the card's memory may get, in percent, before new buffers the
rem CPU fills go into the PC's memory instead (default 80). At 80 the card
rem stopped at about 11.5 of 14.5 GB in the open world, the rest went to PC
rem memory, Windows paged, and the game fell to about 10 real fps. At 90 an
rem auto-test used 12.7 GB of the card and held 17 real fps (30 shown) after
rem 10 minutes, 15 minutes without a crash. Lower it if the card runs out.
if not defined VICTORY_WINDOW_CARD_PERCENT set "VICTORY_WINDOW_CARD_PERCENT=90"

rem Copies of the game's buffers kept in the card's memory, in MB. The default
rem for a 16 GB card is 2048; the prologue filled it in 3 minutes and the rest
rem went into the 7 GB of PC memory Windows lets the card use, which ran out.
rem The card had about 4 GB free then. Lower it if textures start loading late.
if not defined VICTORY_CARD_BUFFERS_MB set "VICTORY_CARD_BUFFERS_MB=4096"

rem Textures uploaded per frame, in MB (default here 128): smaller bursts of
rem upload memory when a new scene loads, which is when the PC ran out.
if not defined VICTORY_TEXTURE_STAGE_MB set "VICTORY_TEXTURE_STAGE_MB=64"

rem Texture uploads sent ahead of the frames that need them (up to 96 MB in
rem flight): off, as in the original VIctory.
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=0"

rem Staging memory for the game's file streaming (default 256 MB; the game asks for 32).
if not defined VICTORY_DSTORAGE_STAGING_MB set "VICTORY_DSTORAGE_STAGING_MB=64"

call "%~dp0cool_fox-tuning.cmd"
call "%~dp0Play-cool_fox.cmd" %*
