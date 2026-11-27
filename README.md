# VIctory-BigCheese (amd branch)

This branch is **[eikkapine/victory-amd](https://github.com/eikkapine/victory-amd)**
with VIctory-BigCheese's changes on top, tuned for a PC with **16 GB of RAM**.
The releases from v2.0 on are built from it. The `main` branch is the older
cool_fox-based VIctory-BigCheese (releases v1.x).

**[Download the latest build](https://github.com/computermakemegta6/VIctory-BigCheese/releases/latest)**
(`VIctory-BigCheese-win64.zip`), extract the whole zip and double-click
`Play-16GB-RAM.cmd` (16 GB of RAM) or `Play.cmd` (victory-amd's normal settings,
32 GB or more; with that much RAM try both and keep the faster one); the first
start asks for your game folder. The zip has `victory.exe`, both launchers,
AMD's FidelityFX DLLs (`fsr\`) and [bigcheese/README.txt](bigcheese/README.txt),
which also has the steps for AI agents setting it up for a user. The
launchers' source: [bigcheese/Play-16GB-RAM.cmd](bigcheese/Play-16GB-RAM.cmd),
[bigcheese/Play.cmd](bigcheese/Play.cmd).

What this branch changes from victory-amd (2026-11-28):

- **16 GB of RAM:** Series S and victory-amd's memory settings for 16 GB.
- **Card window line at 90 %** (`VICTORY_WINDOW_CARD_PERCENT`, 50-95; the fork
  had 80 % fixed): buffers the CPU fills stay in the card's memory instead of
  spilling into PC RAM. In the open world that held 17 real fps instead of about
  10. Mapped buffers are let go of more slowly while the card has room.
- **Crash fix:** one shader (f16876929fa64f9f, leaving the prison) crashes AMD's
  shader compiler; `VICTORY_SKIP_PIPELINES` skips it in release builds too, and
  when pipelines are made early from `early-pipelines.txt`.
- **Keyboard steering** eases to full lock over 0.1 s instead of jumping there,
  and is back in the middle at once when A/D are let go
  (`VICTORY_KEY_STEER_MS`, 0 = as before).
- **The launcher:** menus at up to 60 fps (`VICTORY_FRAME_INTERVAL_MS=16.667`,
  `VICTORY_FPS_CAP=60`) with gameplay never held back on a 16 GB PC, FSR 4 with
  FSR 4 frame generation (`VICTORY_FIDELITYFX_FRAMEGEN=4`), easy setup.

Tested on an RX 9060 XT (16 GB), Ryzen 7 7800X3D, 16 GB of RAM. victory-amd's
own README follows, as it came.

---

# VIctory AMD

An AMD Radeon-tuned fork of [VIctory](https://github.com/Marvisak/VIctory), a
compatibility layer that runs Xbox Series titles natively on Windows, from
files you supply yourself. Built on the work of the community forks credited
below. Tuned and tested on RDNA4 (RX 9070 XT); RDNA2 and RDNA3 are supported by
the installer but less tested.

**This repository contains no game content of any kind**: no executables,
assets, shaders, or data from any title, and no instructions for obtaining
them. You need your own legally obtained copy of a title. See
[docs/asset-policy.md](docs/asset-policy.md).

## What this fork changes

- Performance on AMD: skipped residency waits for short-lived buffers (the
  main source of hitches), high process priority, a below-normal cleanup
  thread, frame pacing on the title thread instead of the submit thread.
- Texture streaming: oldest-first texture queue, mip upgrades on a background
  copy queue (2-4 workers), a texture size cap that scales with free VRAM.
- AMD FidelityFX: FSR upscaling and frame generation through the official
  FidelityFX SDK DLLs (FSR 4 on RDNA4; the SDK picks FSR 4.1 or 3.1 on RDNA2/3).
- Keyboard and mouse controls, F9 frame capture in diagnostics builds.

## Install (Windows, AMD RX 6000 / 7000 / 9000)

1. Download the zip from [Releases](../../releases) and extract it next to the
   title's executable.
2. Run `INSTALL.bat` (not as administrator). It detects your GPU and writes
   launchers tuned for it.

Requirements: 16 GB VRAM recommended, 32 GB RAM, a fast 8-core CPU (the layer
is CPU-bound), current Adrenalin driver, Visual C++ 2015-2022 x64 runtime.

## Build from source

```
cargo build --release --locked -p victory
```

The installer scripts are in [installer/](installer/).

## Credits

- [Marvisak/VIctory](https://github.com/Marvisak/VIctory) (original project)
- [DexNom/VIctory](https://github.com/DexNom/VIctory)
- [avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX)
- [Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)
- [fxceless/victory-fork](https://github.com/fxceless/victory-fork)

## License

GPL-3.0, see [LICENSE](LICENSE). The AMD FidelityFX DLLs shipped in releases
are AMD's, redistributed under [AMD's license](third_party/AMD-FidelityFX-SDK-LICENSE.md).
This project is not affiliated with or endorsed by Microsoft, AMD, or any game
publisher. All trademarks belong to their owners.
