# VIctory-BigCheese

A fork of **cool_fox** (codec_d's fork of [VIctory](https://github.com/Marvisak/VIctory),
the compatibility layer that runs an Xbox Series title on Windows from files you
supply), tuned to be playable on a PC with **16 GB of RAM** and an AMD card.

**This repository contains no game files, code or assets.** You need your own
dump of the game. Not affiliated with or endorsed by the game's developer or
publisher. Licensed GPL-3.0, like VIctory (see [LICENSE](LICENSE)).

## Download and play

**[Download the latest build](../../releases/latest)** (Windows x64 zip), then:

1. Extract the whole zip to a folder of its own (not into the game folder).
2. Drag your game folder (the one with `game_scarlett_master.exe`) onto a
   launcher. It remembers the folder, so after that a double-click is enough.
   - `Play-cool_fox-16GB-RAM.cmd`: the tuned one, tested on 16 GB of RAM with a
     Radeon 16 GB card. Start with this one.
   - `Play-cool_fox.cmd`: plain, the emulator's own defaults and no tuning (for
     trying other builds of `victory.exe`; rooms may look dark without the
     tuned launcher's ambient light)

   (In the source code the launchers are in the `cool_fox` folder; in the
   download zip they are next to `victory.exe`.)
3. The first start translates the game's shaders once (a few minutes).

Optional, AMD FSR upscaling and frame generation (any DX12 card): put
`amd_fidelityfx_upscaler_dx12.dll`, `amd_fidelityfx_framegeneration_dx12.dll` and
`amd_fidelityfx_loader_dx12.dll` from AMD's FidelityFX SDK in a folder named `fsr`
next to the launchers. The tuned launchers use them when they are there.

Needs Windows 10/11 x64, an AVX2 CPU, a DX12 GPU with shader model 6.6 and mesh
shaders, and the page file on "System managed size" (the game reserves 20-25 GB).

## What this fork changes (2026-11-27)

- **Two launchers:** `Play-cool_fox-16GB-RAM.cmd` (tuned, its shared settings in
  `cool_fox-tuning.cmd`) and a plain `Play-cool_fox.cmd` with no tuning.
- **16 GB PCs:** buffers the CPU fills go to the card's memory (Resizable BAR)
  and may fill it to 90 % (`VICTORY_WINDOW_CARD_PERCENT`) before spilling into PC
  RAM. In the open world that held 17 real fps instead of dropping to about 10.
- **AMD FSR frame generation** (`VICTORY_FRAMEGEN=1`, from the FidelityFX SDK's
  frame generation swap chain): about double the frames shown.
- **Crash fix:** one shader (f16876929fa64f9f, leaving the prison) crashes AMD's
  shader compiler; `VICTORY_SKIP_PIPELINES` now skips it in the normal build too,
  not only the diagnostics build.
- **Keyboard steering:** A/D move the emulated stick over 0.25 s instead of to
  full lock at once (`VICTORY_KEY_STEER_MS`, 0 = old behaviour); cars no longer
  swerve on every tap, above all at low frame rates.
- **Experimental, off by default:** `VICTORY_WINDOW_PROMOTE=1` moves buffers that
  spilled into PC RAM back to the card when it has room again.

The rest of this file is cool_fox's own README, as it came.

---

# VIctory on an RX 7900 XTX

## Run the codec_d Windows build

Download and extract the Windows x64 ZIP from the [private releases](https://github.com/codec-d/cool_fox/releases/latest). Open PowerShell in the extracted folder and point the launcher at your own complete Xbox dump.

```powershell
$env:VICTORY_GAME_DIR = 'D:\Games\MyDump'
.\Play-VIctory.cmd pareto-residency
```

The launcher selects Series S and the RTX-tested residency settings. It builds its shader cache locally under `%LOCALAPPDATA%\VIctory\cache-pareto`. Set `VICTORY_CACHE_DIR` to reuse an existing cache outside the checkout. Windows x64, an AVX2 CPU, and a Direct3D 12 GPU with shader model 6.6 and mesh shader support are required.

To build from this checkout, install the pinned Rust toolchain and the MSVC C++ build tools with a Windows SDK, then run `cargo build --release --locked -p victory`. The same launcher finds `target\release\victory.exe`. The sections below describe the upstream AMD fork and its separate launcher.

A fork of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer
that runs an Xbox Series title natively on Windows from files the user supplies.
This one is about **speed on AMD RDNA 3**: the world at 30 frames a second on
one PC, without giving up the picture. It is not there yet.

**This repository contains no game files, code or assets.** You need your own
copy of the game. It is not affiliated with or endorsed by the game's developer
or publisher.

## Where it stands (2026-11-25)

Tested on one machine only: Windows 11, Intel i9-12900K, 64 GB RAM, Radeon
RX 7900 XTX, 1440p, the Series X profile.

| | |
|---|---|
| Menus, videos, loading screens | 30 fps |
| Prologue, indoors | 50-75 ms a frame (13-20 fps) |
| First mission with the boat | 46-53 ms (19-22 fps) |
| Street chase with many cars and people | 100-165 ms (6-10 fps), single frames far longer |
| Outdoors on foot, the camp | 60-90 ms |
| Flying | 42-57 ms |
| Longest session so far | 41 minutes, no crash |
| Memory | 40-46 GB of private memory: **64 GB of RAM recommended** |

It started at 280 ms a frame in the same scenes two days earlier.

What is known to be wrong:

- Trees, plants and far buildings flicker outdoors in some places.
- Rooms with little direct light are too dark without `-RayTracing`.
- Textures of what just came into view appear late while the title streams.
- The first run with a new driver or a new build stutters while the driver makes
  pipelines (`victory warm` makes them ahead).
- Recording a frame in the world makes the title's own watchdog end it.

## How to run it

You need Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with shader
model 6.6 and mesh shaders, Rust (the toolchain is pinned), the MSVC build
tools, and your own dump of the title.

```powershell
git clone https://github.com/avatarmanz/VIctory-7900XTX
cd VIctory-7900XTX
copy victory.example.toml victory.toml      # then set your paths, outside this folder
cargo build --locked --profile diag -p victory --features diagnostics
scripts\play.ps1 -Exe <path of your copy's executable>
```

`scripts\play.ps1 -Safe` leaves out the newest, faster ways; try it first when
something is wrong. `-RayTracing` turns the title's ray-traced indirect light on.

Before the first run, ahead of time (both can be stopped and go on later):

```powershell
target\diag\victory.exe shaders <the shader archives>   # every shader translated, minutes
target\diag\victory.exe warm --used                     # pipelines earlier runs drew with, for a new driver or build
```

Commands, debugging switches and upstream's own description:
[docs/README-fxceless.md](docs/README-fxceless.md). Every `VICTORY_...` switch is
described in a comment where it is read.

## What is different here

In short; the reasons and the measurements are in [docs/findings.md](docs/findings.md).

- **List calls are written down, not carried out, on the title's threads**
  (`VICTORY_THIN`): eight threads of ours translate them.
- **Submissions are prepared by a thread of ours and joined**
  (`VICTORY_ASYNC_SUBMIT`, `VICTORY_JOIN_PARTS`), with what can be spread over
  threads spread: refreshes, compares (AVX2), descriptor records, landings.
- **What shaders wrote comes back beside the write guards**, by the dwords that
  changed, on a thread of its own (`VICTORY_RELAY`, `VICTORY_LAND`).
- **Buffer fills that need bytes still on the card are done on the card**
  (`VICTORY_CARD_STEPS`) instead of waiting for it.
- **The title's writes are caught more cheaply**: one view opened at a fault,
  blocks that rest, places in the title's code that look before they write.
- **Textures go up on a copy queue of their own** and a slot shows one only after
  its upload ran, which also ended a device loss that came every 15 minutes or so.
- **Boats float**: textures that shaders write and the title's CPU reads go back
  to title memory (from fxceless and Geckomacabre, on by default here).
- **Frame record and replay** with per-pass times and shader variants compared
  turn by turn ([docs/replay.md](docs/replay.md)).

## Credits

This is a tree of other people's work with more on top. The history is kept, and
commits taken from elsewhere name the commit and its author.

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its
  contributors: the loader, the runtime services, the Direct3D 12 back end and
  the shader recompiler.
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)**: the fork
  this one started from, and the people whose pull requests it carries.
- **[DexNom/VIctory](https://github.com/DexNom/VIctory)**: ray tracing fixes,
  cube gradients, sparse textures.
- **[Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)**:
  the ambient light idea, pipeline workers, and work by romainhedouin.
- **[FQY55kai/ZJOU-VICTORY](https://github.com/FQY55kai/ZJOU-VICTORY)**.
- **[myokex/victory-amd](https://github.com/myokex/victory-amd)**: the guard over
  every view of a page (with Spexivus, joroyt and tightfall), and Joshua Odom's
  reads into title memory.
- **Geckomacabre** and **cxy0072019-sketch**: the texture read-back.
- **[XCL](https://github.com/ivall/xcl)** (GPL-2.0-only) was read for how the same
  title behaves under another layer; no code of it is here.
- This fork's changes were made with [Claude Code](https://claude.com/claude-code).

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE); the one file that is not
covered is named in [docs/asset-policy.md](docs/asset-policy.md).
