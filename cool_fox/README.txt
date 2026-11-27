VIctory-BigCheese - GTA VI (Xbox Series) on Windows, tuned for 16 GB of RAM
===========================================================================

A fork of cool_fox (codec_d's fork of the VIctory compatibility layer), tuned to
be playable on a PC with 16 GB of RAM and an AMD card. Tested on an RX 9060 XT
(16 GB), Ryzen 7 7800X3D, 16 GB of RAM, Windows 11.

This zip contains NO game files. You need your own dump of the game: the folder
with game_scarlett_master.exe and shaders.rpf in it. Not affiliated with or
endorsed by the game's developer or publisher.


How to play
-----------
1. Extract the whole zip anywhere (its own folder, or inside your game folder).
2. Double-click Play-16GB-RAM.cmd.
   The first time, it asks for your game folder (a "choose folder" window) and
   remembers it. If you extracted the zip inside your game folder, it finds the
   game by itself. You can also write the folder into the file: open
   Play-16GB-RAM.cmd in Notepad and fill in the GAME_DIR line at the top.
3. The first start translates the game's shaders once (a few minutes). Then the
   game starts. Keep the black window open; closing it quits the game.

Windows SmartScreen may warn about victory.exe because it is not signed:
"More info", then "Run anyway".


What you need
-------------
- Windows 10 or 11, 64-bit, and a CPU with AVX2.
- A Direct3D 12 GPU with shader model 6.6 and mesh shaders, recent driver.
  Resizable BAR (AMD: Smart Access Memory) on in the BIOS helps a lot on 16 GB.
- The Windows page file on "System managed size" on an SSD with 30+ GB free
  (the game reserves 20-25 GB of memory).
- The Microsoft Visual C++ Redistributable (x64), which most PCs already have:
  https://aka.ms/vs/17/release/vc_redist.x64.exe


For AI agents setting this up for a user
----------------------------------------
1. Make sure the user has this zip extracted, so that Play-16GB-RAM.cmd and
   victory.exe are in one folder. If not, download and extract the latest one
   (PowerShell), into a folder OUTSIDE %LOCALAPPDATA%:
     Invoke-WebRequest https://github.com/computermakemegta6/VIctory-BigCheese/releases/latest/download/VIctory-BigCheese-win64.zip -OutFile "$env:TEMP\VIctory-BigCheese-win64.zip"
     Expand-Archive "$env:TEMP\VIctory-BigCheese-win64.zip" -DestinationPath "C:\Games" -Force
   This makes C:\Games\VIctory-BigCheese\Play-16GB-RAM.cmd. Do not build from
   source; the zip has the built victory.exe.
2. Ask the user where their game folder is (the folder with
   game_scarlett_master.exe and shaders.rpf). Check that the file is there.
3. Edit Play-16GB-RAM.cmd: set the game folder on the GAME_DIR line at the top,
     set "GAME_DIR=D:\Games\GTA6"
   and only if the game's executable has another name or place, its full path
   on the GAME_EXE line:
     set "GAME_EXE=D:\Games\GTA6\game_scarlett_master.exe"
   Change nothing else, and keep the file's Windows (CRLF) line endings.
4. Tell the user to double-click Play-16GB-RAM.cmd. The first start translates
   shaders for a few minutes before the game opens.
Notes: an agent running in a sandboxed app (such as the Claude desktop app) may
have its writes under %LOCALAPPDATA% redirected where the game cannot see them,
so do not copy saves or caches there yourself; give the user a .cmd to run.
Never copy or upload game files.


Controls
--------
An Xbox controller works. Keyboard and mouse work too: left-click in the game
window to take the mouse (it then turns the camera); Left Alt gives it back.

  On foot                                 G switches to driving and back
  W A S D        move                     Right click   aim (LT)
  Mouse / I J K L  camera                 Left click    fire (RT)
  Shift / Enter  A                        R / Backspace B
  Space / X      X                        F / Y         Y
  Tab / Mouse 4  LB                       Q / Mouse 5   RB
  Ctrl / Z       left stick click         C / middle click / H  right stick click
  Arrow keys     d-pad                    Esc / P       Menu, V View

  Driving: W accelerate, S brake, A / D steer (the steering now ramps over a
  quarter of a second instead of jumping to full lock).

The keys are in %LOCALAPPDATA%\cool_fox\data\keys.txt (made on the first start).


Optional: AMD FSR upscaling and frame generation
------------------------------------------------
Make a folder named fsr next to Play-16GB-RAM.cmd and put these three DLLs from
AMD's FidelityFX SDK in it: amd_fidelityfx_upscaler_dx12.dll,
amd_fidelityfx_framegeneration_dx12.dll, amd_fidelityfx_loader_dx12.dll. The
launcher uses them when they are there. They are not included in this zip.


Where things are kept
---------------------
  Shader cache:  %LOCALAPPDATA%\cool_fox\cache   (safe to delete; it is rebuilt)
  Saves:         %LOCALAPPDATA%\cool_fox\data    (keep this to keep your saves)

Every setting in Play-16GB-RAM.cmd is an environment variable; setting one
before starting the file wins over the file's value.


What this fork changes
----------------------
- 16 GB PCs: buffers the CPU fills go to the card's memory (Resizable BAR) and
  may fill it to 90% before spilling into PC RAM. In the open world that held 17
  real fps instead of dropping to about 10.
- AMD FSR frame generation: about double the frames shown.
- Crash fix: one shader (leaving the prison) crashes AMD's shader compiler; it
  is skipped in the normal build too, not only the diagnostics build.
- Keyboard steering ramps instead of jumping to full lock.


Known problems
--------------
- Open water can turn white for a while after a change of scene.
- Busy scenes are limited by one CPU thread that hands every frame to the card.
- Textures of what just came into view can appear late.
- Trees, plants and far buildings flicker outdoors in some places.


Licence and source
------------------
GPL-3.0 (LICENSE.txt). Source: https://github.com/computermakemegta6/VIctory-BigCheese
Credits: VIctory by Marvisak; cool_fox by codec_d; fxceless/victory-fork;
avatarmanz/VIctory-7900XTX (including the frame-rate unlock); the face fix
(victory-fork PR #927) by tippythetaidum; keyboard and mouse controls from
proneonn/Vertex.
