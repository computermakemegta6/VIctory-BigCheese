//! (Neptune) `Bink2Scarlett.dll`, the title's video player, watched: whether the title
//! plays a video now, for our frame clock (`victory_gpu::frame`, `VICTORY_FRAME_INTERVAL_VIDEO`).
//!
//! The title's own DLL stays: it is mapped and bound as before, and every call goes on to it unchanged. [`Watch`] stands in front of the
//! resolver and gives the title, for five of the DLL's exports, a few instructions of ours
//! that tell `victory_gpu::frame::bink_called` which call it is and then jump to the DLL's
//! own export: every register and the stack are as the title left them (its arguments,
//! however many), and the DLL returns to the title itself.
//!
//! Of the DLL's 74 exports the title imports 23 (game_scarlett_master.exe 1.0.2.1). Watched:
//! `BinkOpenWithOptions` and `BinkClose`, which say when a video is opened and closed, and
//! the per-frame `BinkDoFrame`, `BinkDoFrameAsyncMulti` (a picture decoded) and
//! `BinkNextFrame` (on to the next one).

use std::sync::atomic::{AtomicUsize, Ordering};

use victory_gpu::frame::{Bink, bink_called};
use victory_loader::{ImportResolver, Provenance, Resolution};
use victory_pe::ImportSymbol;

const DLL: &str = "bink2scarlett.dll";

/// Told by the stand-ins: which call (the order of [`exports`]).
extern "system" fn note(which: u32) {
    let call = match which {
        0 => Bink::Open,
        1 => Bink::Close,
        2 | 3 => Bink::Decode,
        _ => Bink::Next,
    };
    // Nothing may unwind out of here into the stand-in, which has no unwind data.
    let _ = std::panic::catch_unwind(|| bink_called(call));
}

/// A stand-in for export `$i`: the four argument registers kept, [`note`] told, the registers
/// put back, then a jump to the DLL's own export (`$real`). At the entry the stack is 8 off
/// 16 (the return address); four pushes and 40 bytes (the callee's 32 of home space and 8)
/// align it for the call.
macro_rules! stand_in {
    ($name:ident, $real:ident, $i:literal) => {
        static $real: AtomicUsize = AtomicUsize::new(0);
        #[unsafe(naked)]
        unsafe extern "system" fn $name() {
            core::arch::naked_asm!(
                "push rcx",
                "push rdx",
                "push r8",
                "push r9",
                "sub rsp, 40",
                "mov ecx, {which}",
                "call {note}",
                "add rsp, 40",
                "pop r9",
                "pop r8",
                "pop rdx",
                "pop rcx",
                "jmp qword ptr [rip + {real}]",
                which = const $i,
                note = sym note,
                real = sym $real,
            )
        }
    };
}

stand_in!(open, REAL_OPEN, 0);
stand_in!(close, REAL_CLOSE, 1);
stand_in!(do_frame, REAL_DO_FRAME, 2);
stand_in!(do_frame_async_multi, REAL_DO_FRAME_ASYNC_MULTI, 3);
stand_in!(next_frame, REAL_NEXT_FRAME, 4);

/// A stand-in's address is all that is used of it (it is never called from Rust).
type Stand = unsafe extern "system" fn();

/// The exports taken: ours for each, and where the DLL's own goes.
fn exports() -> [(&'static str, usize, &'static AtomicUsize); 5] {
    [
        ("BinkOpenWithOptions", open as Stand as usize, &REAL_OPEN),
        ("BinkClose", close as Stand as usize, &REAL_CLOSE),
        ("BinkDoFrame", do_frame as Stand as usize, &REAL_DO_FRAME),
        ("BinkDoFrameAsyncMulti", do_frame_async_multi as Stand as usize, &REAL_DO_FRAME_ASYNC_MULTI),
        ("BinkNextFrame", next_frame as Stand as usize, &REAL_NEXT_FRAME),
    ]
}

/// Whether the title's calls into Bink go through us: with `VICTORY_FRAME_INTERVAL_MS` set,
/// unless `VICTORY_FRAME_INTERVAL_VIDEO=0`.
pub fn on() -> bool {
    std::env::var_os("VICTORY_FRAME_INTERVAL_MS").is_some()
        && std::env::var("VICTORY_FRAME_INTERVAL_VIDEO").map_or(true, |v| v.trim() != "0")
}

/// Stands in front of `inner`: what `inner` binds to one of the five exports of the title's
/// `Bink2Scarlett.dll` is remembered, and ours is bound in its place. Everything else is
/// `inner`'s answer.
pub struct Watch<'a> {
    pub inner: &'a dyn ImportResolver,
}

impl ImportResolver for Watch<'_> {
    fn resolve(&self, dll: &str, symbol: &ImportSymbol) -> Resolution {
        let found = self.inner.resolve(dll, symbol);
        let Resolution::Found {
            address,
            provenance: Provenance::Title,
        } = found
        else {
            return found;
        };
        let ImportSymbol::Name { name, .. } = symbol else {
            return found;
        };
        if !dll.eq_ignore_ascii_case(DLL) {
            return found;
        }
        let Some(&(_, ours, real)) = exports().iter().find(|(n, ..)| *n == name.as_str()) else {
            return found;
        };
        let was = real.swap(address, Ordering::AcqRel);
        if was != 0 && was != address {
            // Ours jumps to the last one bound.
            tracing::warn!(export = name.as_str(), "bink: an export bound again, to another place");
        }
        tracing::info!(export = name.as_str(), real = format_args!("{address:#x}"), "bink: the title's call into its video player goes through us, then to its own");
        Resolution::Found {
            address: ours,
            provenance: Provenance::Hle,
        }
    }
}
