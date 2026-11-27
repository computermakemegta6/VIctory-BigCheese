cool_fox - GTA VI (Xbox Series) on Windows, Series S mode - build 30e54b4
=========================================================================

This is cool_fox, codec_d's fork of the VIctory compatibility layer, built from
source, with fixes gathered from other VIctory forks and our own testing (see
"What is new" below).

This zip contains NO game files. You need your own dump of the game: the folder
with game_scarlett_master.exe and shaders.rpf in it. It is not affiliated with
or endorsed by the game's developer or publisher.


What is new since the first cool_fox zip
----------------------------------------
- Faces: no more noise or blocky faces (the game's character bakes get the time
  they need), plus the face fix from fxceless/victory-fork pull request #927.
- Colour: no more purple or blue skin, and no more blotchy "painted" walls and skin.
- Textures and memory: each texture keeps only the sizes the game is using on the
  graphics card. The card no longer fills up, textures load much sooner, and the
  game's memory peak dropped from about 41 GB to about 27 GB.
- Frame rate: the game's own 30 fps cap is lifted (up to 60).
- Keyboard and mouse controls (below).
- Optional: AMD's PC upscaler in place of the game's own (below).


What you need
-------------
- Windows 10 or 11, 64-bit, and a CPU with AVX2.
- A Direct3D 12 GPU with shader model 6.6 and mesh shaders, with a recent driver.
  Tested on an RTX 4090 (24 GB). Cards with much less video memory may not cope.
- Memory: the game used about 27 GB at its peak here (RAM plus page file). With
  32 GB of RAM, keep the Windows page file at "System managed size" on an SSD with
  at least 30 GB free.
- The Microsoft Visual C++ Redistributable (x64), which most PCs already have:
  https://aka.ms/vs/17/release/vc_redist.x64.exe


How to play
-----------
1. Extract the whole zip to a folder of its own (not into the game folder).
2. Drag your game folder onto Play-cool_fox.cmd. It remembers the folder in
   game-folder.txt, so after that you can just double-click Play-cool_fox.cmd.
3. On the first start it translates the game's shaders ahead once (a few minutes
   on a fast CPU, longer on slower ones). The first session stutters while the
   graphics driver builds its pipelines; later starts are much smoother.

Windows SmartScreen may warn about victory.exe because it is not signed:
"More info", then "Run anyway".

On our PC (RTX 4090, Ryzen 7 9800X3D, 32 GB RAM, Windows 11), a 17-minute session
ran at a median of about 30 fps, with the slowest tenth of the time around 24 fps;
light scenes reach the 40s.

Optional, a little faster: AMD's PC upscaler in place of the game's own. Make a
folder named fsr next to Play-cool_fox.cmd and put amd_fidelityfx_upscaler_dx12.dll
in it (AMD FidelityFX, from any PC game or the FidelityFX SDK that ships it). The
launcher uses it when it is there. It is not included in this zip.


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

  Driving: W accelerate, S brake, A / D steer.

The keys are in %LOCALAPPDATA%\cool_fox\data\keys.txt (made on the first start).
Change them there and save; the running game picks them up within a second.
VICTORY_MOUSE_SENSITIVITY (default 1.0) sets the mouse speed.


Where things are kept
---------------------
  Shader cache:  %LOCALAPPDATA%\cool_fox\cache   (safe to delete; it is rebuilt)
  Saves:         %LOCALAPPDATA%\cool_fox\data    (keep this to keep your saves)

To use other folders, set VICTORY_CACHE_DIR or VICTORY_DATA_DIR before starting
Play-cool_fox.cmd. All of its other settings are environment variables too;
VICTORY_LOG=info shows more in the console window.


Known problems
--------------
- Open water (the opening's boat ride) can turn white for half a minute or more,
  mostly after a change of scene, then looks right again.
- Busy scenes still run at 24 to 30 fps: one thread that hands every frame to the
  graphics card is the limit.
- Textures of what just came into view can appear late.
- Trees, plants and far buildings flicker outdoors in some places.


Licence and source
------------------
GPL-3.0 (LICENSE.txt). The source code of exactly this build is in
source\cool_fox-src-30e54b4.zip (commit 30e54b4). To build it yourself, install
Rust (the toolchain is pinned in rust-toolchain.toml) and the MSVC C++ build
tools with a Windows SDK, then run:  cargo build --release --locked -p victory

Credits: VIctory by Marvisak; fxceless/victory-fork; avatarmanz/VIctory-7900XTX
(including the frame-rate unlock); cool_fox by codec_d; the face fix (victory-fork
PR #927) by tippythetaidum; keyboard and mouse controls from proneonn/Vertex.
