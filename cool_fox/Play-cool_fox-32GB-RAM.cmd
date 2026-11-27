@echo off
rem cool_fox for a PC with 32 GB of RAM: the tuned settings (cool_fox-tuning.cmd)
rem with room for the game's upload bursts, and the card's memory used further.
rem Not tested on a 32 GB PC yet: the 16 GB file is the tested one. If the game
rem runs out of memory, use Play-cool_fox-16GB-RAM.cmd. Any setting can be
rem overridden by setting the variable before starting this file.
setlocal EnableExtensions DisableDelayedExpansion

rem Buffers the CPU fills go in the card's memory (needs Resizable BAR / Smart
rem Access Memory on; without it nothing changes), so they stay out of the PC's
rem memory. The prologue needs about 3.4 GB of them.
if not defined VICTORY_WINDOWS_BAR_MB set "VICTORY_WINDOWS_BAR_MB=6144"

rem How full the card's memory may get, in percent, before those buffers go to
rem the PC's memory instead (default 80; 90 held up in the open world on a 16 GB
rem card). Lower it if the card runs out, on cards with less than 12 GB above all.
if not defined VICTORY_WINDOW_CARD_PERCENT set "VICTORY_WINDOW_CARD_PERCENT=90"

rem Texture uploads sent ahead of the frames that need them, texture and
rem streaming staging at their defaults: the PC has the memory for the bursts.

call "%~dp0cool_fox-tuning.cmd"
call "%~dp0Play-cool_fox.cmd" %*
