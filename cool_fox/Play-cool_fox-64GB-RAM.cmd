@echo off
rem cool_fox for a PC with 64 GB of RAM (or more): the tuned settings
rem (cool_fox-tuning.cmd) with every memory setting at its default, as cool_fox
rem ran on the 32-64 GB PCs it was first tuned on (the game's private memory
rem reached 27-46 GB there). Any setting can be overridden by setting the
rem variable before starting this file.
setlocal EnableExtensions DisableDelayedExpansion

call "%~dp0cool_fox-tuning.cmd"
call "%~dp0Play-cool_fox.cmd" %*
