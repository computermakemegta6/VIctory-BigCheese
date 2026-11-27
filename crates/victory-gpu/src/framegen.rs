//! AMD FSR frame generation (`VICTORY_FRAMEGEN=1`; off by default), ported
//! from rodrion-vi's `ffx.rs`.
//!
//! It needs the title's upscale redirected to the PC upscaler
//! (`VICTORY_FSR_DLL`, `victory-hle`'s `fidelityfx`): the frame generation
//! context is fed by that upscale's depth, motion vectors, jitter and camera
//! ([`prepare`]), on the host list the upscale recorded onto. The window
//! presents through AMD's frame generation swap chain ([`swap_chain`]) in
//! place of a plain one, which shows a generated frame between each two
//! real ones; the console's interface plane is handed to it alone
//! ([`before_present`]) so that it is laid over real and generated frames
//! and not interpolated.
//!
//! The DLLs: `amd_fidelityfx_framegeneration_dx12.dll` and, if beside it,
//! `amd_fidelityfx_loader_dx12.dll` (FidelityFX SDK 2.x `signedbin`), in
//! `VICTORY_FFX_FG_DIR` or else the folder of `VICTORY_FSR_DLL`. The
//! loader is used when there; else the frame generation DLL's own exports.
//! Anything that fails turns frame generation off for the run, logged once,
//! and the game presents as without it.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use windows::Win32::Graphics::Direct3D12::{ID3D12CommandQueue, ID3D12Resource};
use windows::Win32::Graphics::Dxgi::{DXGI_SWAP_CHAIN_DESC1, IDXGIFactory2, IDXGISwapChain3, IDXGISwapChain4};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::core::{Interface, PCSTR, PCWSTR};

type ReturnCode = u32;
const OK: ReturnCode = 0;
const ERROR: ReturnCode = 1;

/// `ffxApiHeader`.
#[repr(C)]
struct Header {
    kind: u64,
    next: *mut Header,
}

fn header(kind: u64) -> Header {
    Header {
        kind,
        next: std::ptr::null_mut(),
    }
}

/// `FfxApiResource`: the resource, its description (type, format, width,
/// height, depth, levels, flags, usage), its state; 48 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Resource {
    pub resource: *mut c_void,
    pub description: [u32; 8],
    pub state: u32,
}
const _: () = assert!(size_of::<Resource>() == 48);

const NO_RESOURCE: Resource = Resource {
    resource: std::ptr::null_mut(),
    description: [0; 8],
    state: 0,
};

/// What [`prepare`] takes from an upscale dispatch the PC upscaler ran.
#[derive(Clone, Copy, Debug)]
pub struct Upscale {
    /// The host command list the upscale recorded onto (open).
    pub command_list: *mut c_void,
    pub render_size: [u32; 2],
    pub jitter: [f32; 2],
    pub motion_vector_scale: [f32; 2],
    pub frame_time_delta: f32,
    pub reset: bool,
    /// Near, far, vertical field of view, metres per unit.
    pub camera: [f32; 4],
    /// The host textures, described and in the state the upscaler left them.
    pub depth: Resource,
    pub motion_vectors: Resource,
}

const BACKEND_DX12: u64 = 2;
const CREATE_FRAMEGEN: u64 = 0x0002_0001;
const CONFIGURE_FRAMEGEN: u64 = 0x0002_0002;
const DISPATCH_FRAMEGEN_PREPARE_V2: u64 = 0x0002_000c;
const CREATE_SWAP_CHAIN_FOR_HWND: u64 = 0x0003_0006;
const CONFIGURE_SWAP_CHAIN_UI: u64 = 0x0003_0002;
const DISPATCH_SWAP_CHAIN_WAIT_FOR_PRESENTS: u64 = 0x0003_0007;
const CREATE_SWAP_CHAIN_VERSION: u64 = 0x0003_000b;
/// `FFX_FRAMEGENERATION_SWAPCHAIN_DX12_VERSION` of SDK 2.4 (4.0.0).
const SWAP_CHAIN_VERSION: u32 = 4 << 22;
const UI_PREMULTIPLIED: u32 = 1 << 0;
const UI_DOUBLE_BUFFERED: u32 = 1 << 1;
const FFX_COMMON: u32 = 1 << 0;

const FRAMEGEN_DLL: &str = "amd_fidelityfx_framegeneration_dx12.dll";
const LOADER_DLL: &str = "amd_fidelityfx_loader_dx12.dll";

/// `ffxCreateBackendDX12Desc`.
#[repr(C)]
struct BackendDx12 {
    header: Header,
    device: *mut c_void,
}

/// `ffxCreateContextDescFrameGenerationSwapChainVersionDX12`.
#[repr(C)]
struct Version {
    header: Header,
    version: u32,
}

/// `ffxCreateContextDescFrameGenerationSwapChainForHwndDX12`.
#[repr(C)]
struct CreateSwapChainForHwnd {
    header: Header,
    swap_chain: *mut *mut c_void,
    hwnd: *mut c_void,
    desc: *const c_void,
    fullscreen: *const c_void,
    factory: *mut c_void,
    queue: *mut c_void,
}

/// `ffxCreateContextDescFrameGeneration`.
#[repr(C)]
struct CreateFramegen {
    header: Header,
    flags: u32,
    display: [u32; 2],
    max_render: [u32; 2],
    format: u32,
}

type GenerateCallback = unsafe extern "C" fn(*mut Header, *mut c_void) -> ReturnCode;

/// `ffxConfigureDescFrameGeneration`.
#[repr(C)]
struct ConfigureFramegen {
    header: Header,
    swap_chain: *mut c_void,
    present_callback: *const c_void,
    present_context: *mut c_void,
    generate_callback: Option<GenerateCallback>,
    generate_context: *mut c_void,
    enabled: bool,
    async_workloads: bool,
    hudless: Resource,
    flags: u32,
    only_generated: bool,
    rect: [i32; 4],
    frame: u64,
}
const _: () = assert!(size_of::<ConfigureFramegen>() == 144);

/// `ffxDispatchDescFrameGenerationPrepareV2`.
#[repr(C)]
struct PrepareV2 {
    header: Header,
    frame: u64,
    flags: u32,
    command_list: *mut c_void,
    render_size: [u32; 2],
    jitter: [f32; 2],
    motion_vector_scale: [f32; 2],
    frame_time_delta: f32,
    reset: bool,
    camera: [f32; 4],
    depth: Resource,
    motion_vectors: Resource,
    /// Camera position, up, right and forward: unknown here, left zero.
    camera_vectors: [f32; 12],
}
const _: () = assert!(size_of::<PrepareV2>() == 232);

/// `ffxConfigureDescFrameGenerationSwapChainRegisterUiResourceDX12`.
#[repr(C)]
struct RegisterUi {
    header: Header,
    ui: Resource,
    flags: u32,
}

type Farproc = unsafe extern "system" fn() -> isize;
type Create = unsafe extern "C" fn(*mut *mut c_void, *mut Header, *const c_void) -> ReturnCode;
type Destroy = unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> ReturnCode;
type Call = unsafe extern "C" fn(*mut *mut c_void, *const Header) -> ReturnCode;

struct Api {
    create: Create,
    destroy: Destroy,
    configure: Call,
    dispatch: Call,
}

/// Whether `VICTORY_FRAMEGEN=1` asks for frame generation and the PC
/// upscaler (`VICTORY_FSR_DLL`) is there to feed it.
pub fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        let asked = std::env::var_os("VICTORY_FRAMEGEN").is_some_and(|v| v == "1");
        let fsr = std::env::var_os("VICTORY_FSR_DLL").is_some_and(|v| !v.is_empty());
        if asked && !fsr {
            tracing::error!("framegen: off: VICTORY_FRAMEGEN=1 needs the PC upscaler (VICTORY_FSR_DLL)");
        } else if asked {
            tracing::info!("framegen: asked (VICTORY_FRAMEGEN=1)");
        }
        asked && fsr
    })
}

/// The folder of the frame generation DLLs.
fn folder() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("VICTORY_FFX_FG_DIR").filter(|d| !d.is_empty()) {
        return Some(dir.into());
    }
    let fsr = PathBuf::from(std::env::var_os("VICTORY_FSR_DLL")?);
    let fsr = std::path::absolute(&fsr).unwrap_or(fsr);
    fsr.parent().map(PathBuf::from)
}

fn api() -> Option<&'static Api> {
    static API: OnceLock<Option<Api>> = OnceLock::new();
    API.get_or_init(|| {
        if !on() {
            return None;
        }
        let dir = folder()?;
        let load = |name: &str| {
            use std::os::windows::ffi::OsStrExt;
            let path: Vec<u16> = dir.join(name).as_os_str().encode_wide().chain([0]).collect();
            // SAFETY: a NUL-terminated path.
            unsafe { LoadLibraryW(PCWSTR(path.as_ptr())) }.ok()
        };
        // The loader finds its effect DLLs by name: loaded first by full
        // path, they are found wherever they lie.
        let Some(framegen) = load(FRAMEGEN_DLL) else {
            tracing::error!(?dir, "framegen: off: {FRAMEGEN_DLL} not found there (VICTORY_FFX_FG_DIR names its folder)");
            FAILED.store(true, Ordering::Relaxed);
            return None;
        };
        let (module, which) = match dir.join(LOADER_DLL).is_file().then(|| load(LOADER_DLL)).flatten() {
            Some(loader) => (loader, LOADER_DLL),
            None => (framegen, FRAMEGEN_DLL),
        };
        let get = |name: &[u8]| {
            // SAFETY: a loaded module and a NUL-terminated name.
            unsafe { GetProcAddress(module, PCSTR(name.as_ptr())) }
        };
        // SAFETY: the SDK's exports, of these types (`ffx_api.h`).
        let api = unsafe {
            Some(Api {
                create: std::mem::transmute::<Farproc, Create>(get(b"ffxCreateContext\0")?),
                destroy: std::mem::transmute::<Farproc, Destroy>(get(b"ffxDestroyContext\0")?),
                configure: std::mem::transmute::<Farproc, Call>(get(b"ffxConfigure\0")?),
                dispatch: std::mem::transmute::<Farproc, Call>(get(b"ffxDispatch\0")?),
            })
        };
        match &api {
            Some(_) => tracing::info!(?dir, through = which, "framegen: AMD frame generation loaded"),
            None => {
                tracing::error!(?dir, which, "framegen: off: the DLL lacks the FidelityFX exports");
                FAILED.store(true, Ordering::Relaxed);
            }
        }
        api
    })
    .as_ref()
}

fn host_device() -> Option<*mut c_void> {
    crate::host::host().map(|h| h.device.as_raw())
}

/// What the frame generation context is made with, learnt from the upscale
/// context and the swap chain.
struct Framegen {
    upscale_flags: u32,
    max_render: [u32; 2],
    display: [u32; 2],
    /// An FfxApiSurfaceFormat.
    format: u32,
}

static FRAMEGEN: Mutex<Framegen> = Mutex::new(Framegen {
    upscale_flags: 0,
    max_render: [0; 2],
    display: [0; 2],
    format: 0,
});
static SWAP_CHAIN_CONTEXT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
/// The swap chain the SDK made (the presenter holds its reference).
static SWAP_CHAIN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static CONTEXT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
/// Set on the first failure: frame generation stays off for the run.
static FAILED: AtomicBool = AtomicBool::new(false);
/// The frame identifier of the next present.
static FRAME: AtomicU64 = AtomicU64::new(0);
/// The last frame prepared, plus one.
static PREPARED: AtomicU64 = AtomicU64::new(0);
/// The presented frame rate in tenths, for the caption (0: not measured).
static PRESENTED_TENTHS: AtomicU32 = AtomicU32::new(0);

/// The upscale context the title made on the PC upscaler: its creation
/// flags and largest render size.
pub fn upscale_context(flags: u32, max_render: [u32; 2]) {
    if !on() {
        return;
    }
    let mut fg = FRAMEGEN.lock().unwrap_or_else(|e| e.into_inner());
    (fg.upscale_flags, fg.max_render) = (flags, max_render);
}

/// The frame generation creation flags for an upscale context's: the motion
/// vector and depth bits, at the same positions in both.
fn flags(upscale_flags: u32) -> u32 {
    upscale_flags & 0b1_1110
}

/// The FfxApiSurfaceFormat of a swap chain format.
fn surface_format(dxgi: i32) -> u32 {
    match dxgi {
        28 => 10,
        87 => 14,
        24 => 17,
        10 => 4,
        _ => 0,
    }
}

/// The frame generation swap chain for `hwnd`, made as `desc` says on
/// `queue`; `None` (logged once, frame generation off for the run) if the
/// SDK cannot make it.
pub(crate) fn swap_chain(
    hwnd: windows::Win32::Foundation::HWND,
    desc: &DXGI_SWAP_CHAIN_DESC1,
    factory: &IDXGIFactory2,
    queue: &ID3D12CommandQueue,
) -> Option<IDXGISwapChain3> {
    let api = api()?;
    if FAILED.load(Ordering::Relaxed) {
        return None;
    }
    let mut out = std::ptr::null_mut();
    let mut version = Version {
        header: header(CREATE_SWAP_CHAIN_VERSION),
        version: SWAP_CHAIN_VERSION,
    };
    let mut create = CreateSwapChainForHwnd {
        header: Header {
            kind: CREATE_SWAP_CHAIN_FOR_HWND,
            next: (&raw mut version).cast(),
        },
        swap_chain: &raw mut out,
        hwnd: hwnd.0,
        desc: std::ptr::from_ref(desc).cast(),
        fullscreen: std::ptr::null(),
        factory: factory.as_raw(),
        queue: queue.as_raw(),
    };
    // SAFETY: the SDK's call with a chain that outlives it.
    let r = unsafe { (api.create)(SWAP_CHAIN_CONTEXT.as_ptr(), (&raw mut create).cast(), std::ptr::null()) };
    if r != OK || out.is_null() {
        tracing::error!(r, "framegen: off: the SDK made no frame generation swap chain; presenting as before");
        FAILED.store(true, Ordering::Relaxed);
        return None;
    }
    // SAFETY: the swap chain the SDK made, with a reference for us.
    let swap_chain = unsafe { IDXGISwapChain4::from_raw(out) };
    SWAP_CHAIN.store(out, Ordering::Release);
    tracing::info!("framegen: presenting through AMD's frame generation swap chain");
    swap_chain.cast().ok()
}

/// The swap chain's size and format, at creation and after each resize; a
/// frame generation context made for another is let go (made again at the
/// next upscale).
pub(crate) fn display(width: u32, height: u32, format: i32) {
    let mut fg = FRAMEGEN.lock().unwrap_or_else(|e| e.into_inner());
    let format = surface_format(format);
    if (fg.display, fg.format) == ([width, height], format) {
        return;
    }
    (fg.display, fg.format) = ([width, height], format);
    let Some(api) = api() else { return };
    if CONTEXT.load(Ordering::Acquire).is_null() {
        return;
    }
    configure(api, &fg, FRAME.load(Ordering::Acquire), false);
    let wait = header(DISPATCH_SWAP_CHAIN_WAIT_FOR_PRESENTS);
    // SAFETY: the SDK's calls on the contexts it made.
    unsafe {
        (api.dispatch)(SWAP_CHAIN_CONTEXT.as_ptr(), (&raw const wait).cast());
        (api.destroy)(CONTEXT.as_ptr(), std::ptr::null());
    }
    CONTEXT.store(std::ptr::null_mut(), Ordering::Release);
}

/// Records the frame generation prepare for the next present on the
/// upscale's host list, with the upscale's inputs; the first upscale after
/// a present only.
pub fn prepare(d: &Upscale) {
    if !on() || FAILED.load(Ordering::Relaxed) || SWAP_CHAIN_CONTEXT.load(Ordering::Acquire).is_null() {
        return;
    }
    if d.depth.resource.is_null() || d.motion_vectors.resource.is_null() || d.command_list.is_null() {
        return;
    }
    let Some(api) = api() else { return };
    let frame = FRAME.load(Ordering::Acquire);
    if PREPARED.swap(frame + 1, Ordering::AcqRel) == frame + 1 {
        return;
    }
    let fg = FRAMEGEN.lock().unwrap_or_else(|e| e.into_inner());
    if CONTEXT.load(Ordering::Acquire).is_null() && !create(api, &fg) {
        PREPARED.store(0, Ordering::Release);
        return;
    }
    let prepare = PrepareV2 {
        header: header(DISPATCH_FRAMEGEN_PREPARE_V2),
        frame,
        flags: 0,
        command_list: d.command_list,
        render_size: d.render_size,
        jitter: d.jitter,
        motion_vector_scale: d.motion_vector_scale,
        frame_time_delta: d.frame_time_delta,
        reset: d.reset,
        camera: d.camera,
        depth: d.depth,
        motion_vectors: d.motion_vectors,
        camera_vectors: [0.0; 12],
    };
    // SAFETY: the SDK's call on an open host list with live textures.
    let r = unsafe { (api.dispatch)(CONTEXT.as_ptr(), (&raw const prepare).cast()) };
    if r != OK {
        PREPARED.store(0, Ordering::Release);
        static ERR: std::sync::Once = std::sync::Once::new();
        ERR.call_once(|| tracing::warn!(frame, render = ?d.render_size, result = r, "framegen: prepare returned an error"));
    }
    static FIRST: std::sync::Once = std::sync::Once::new();
    FIRST.call_once(|| {
        tracing::info!(frame, render = ?d.render_size, result = r, depth = ?d.depth.description, motion = ?d.motion_vectors.description, "framegen: first prepare recorded");
    });
}

/// Makes the frame generation context; `false` if it cannot be made yet
/// (no swap chain size or upscale context known) or at all (logged once,
/// frame generation off for the run: the swap chain presents as is).
fn create(api: &Api, fg: &Framegen) -> bool {
    let Some(device) = host_device() else { return false };
    if fg.display.contains(&0) || fg.max_render.contains(&0) {
        return false;
    }
    let mut dx12 = BackendDx12 {
        header: header(BACKEND_DX12),
        device,
    };
    let mut create = CreateFramegen {
        header: Header {
            kind: CREATE_FRAMEGEN,
            next: (&raw mut dx12).cast(),
        },
        flags: flags(fg.upscale_flags),
        display: fg.display,
        max_render: fg.max_render,
        format: fg.format,
    };
    // SAFETY: the SDK's call with a chain that outlives it.
    let r = unsafe { (api.create)(CONTEXT.as_ptr(), (&raw mut create).cast(), std::ptr::null()) };
    if r != OK {
        tracing::error!(r, display = ?fg.display, render = ?fg.max_render, "framegen: off: no frame generation context; presenting as before");
        FAILED.store(true, Ordering::Relaxed);
        CONTEXT.store(std::ptr::null_mut(), Ordering::Release);
        return false;
    }
    tracing::info!(display = ?fg.display, render = ?fg.max_render, flags = create.flags, format = fg.format, "framegen: frame generation context on the host GPU");
    true
}

fn configure(api: &Api, fg: &Framegen, frame: u64, enabled: bool) {
    let c = ConfigureFramegen {
        header: header(CONFIGURE_FRAMEGEN),
        swap_chain: SWAP_CHAIN.load(Ordering::Acquire),
        present_callback: std::ptr::null(),
        present_context: std::ptr::null_mut(),
        generate_callback: Some(generate),
        generate_context: CONTEXT.as_ptr().cast(),
        enabled,
        async_workloads: false,
        hudless: NO_RESOURCE,
        flags: 0,
        only_generated: false,
        rect: [0, 0, fg.display[0] as i32, fg.display[1] as i32],
        frame,
    };
    // SAFETY: the SDK's call on the context it made.
    let r = unsafe { (api.configure)(CONTEXT.as_ptr(), (&raw const c).cast()) };
    if r != OK {
        static REPORTED: std::sync::Once = std::sync::Once::new();
        REPORTED.call_once(|| tracing::warn!(r, "framegen: configuration refused"));
    }
}

/// The SDK's call back to generate a frame: its dispatch on our context.
unsafe extern "C" fn generate(params: *mut Header, context: *mut c_void) -> ReturnCode {
    let Some(api) = api() else { return ERROR };
    // SAFETY: the SDK's description, for the context it was configured with.
    unsafe { (api.dispatch)(context.cast(), params) }
}

/// Before a present through the frame generation swap chain: `ui` (the
/// window's size and format, in the common state, premultiplied; `None`:
/// the interface is in the image) is registered for composition, and frame
/// generation is on for the frame if an upscale prepared it.
pub(crate) fn before_present(ui: Option<&ID3D12Resource>) {
    let Some(api) = api() else { return };
    let fg = FRAMEGEN.lock().unwrap_or_else(|e| e.into_inner());
    let ui = ui.map_or(NO_RESOURCE, |t| {
        const FFX_TEXTURE_2D: u32 = 2;
        const FFX_USAGE_RENDER_TARGET: u32 = 1;
        Resource {
            resource: t.as_raw(),
            description: [FFX_TEXTURE_2D, fg.format, fg.display[0], fg.display[1], 1, 1, 0, FFX_USAGE_RENDER_TARGET],
            state: FFX_COMMON,
        }
    });
    let register = RegisterUi {
        header: header(CONFIGURE_SWAP_CHAIN_UI),
        ui,
        flags: UI_DOUBLE_BUFFERED | UI_PREMULTIPLIED,
    };
    // SAFETY: the SDK's call on the swap chain context it made.
    unsafe { (api.configure)(SWAP_CHAIN_CONTEXT.as_ptr(), (&raw const register).cast()) };
    if CONTEXT.load(Ordering::Acquire).is_null() {
        return;
    }
    let frame = FRAME.load(Ordering::Acquire);
    configure(api, &fg, frame, PREPARED.load(Ordering::Acquire) == frame + 1);
}

/// After a present through the frame generation swap chain, given the
/// swap chain's count of presents (real and generated): once a second, the
/// real frame rate and the presented one, logged.
pub(crate) fn after_present(presented: Option<u32>) {
    let real = FRAME.fetch_add(1, Ordering::AcqRel) + 1;
    static SINCE: Mutex<Option<(std::time::Instant, u64, u32)>> = Mutex::new(None);
    let mut since = SINCE.lock().unwrap_or_else(|e| e.into_inner());
    let presented = presented.unwrap_or(0);
    let Some((at, real_then, presented_then)) = *since else {
        *since = Some((std::time::Instant::now(), real, presented));
        return;
    };
    let seconds = at.elapsed().as_secs_f64();
    if seconds < 1.0 {
        return;
    }
    let real_fps = (real - real_then) as f64 / seconds;
    let presented_fps = f64::from(presented.wrapping_sub(presented_then)) / seconds;
    let active = !CONTEXT.load(Ordering::Acquire).is_null() && !FAILED.load(Ordering::Relaxed);
    PRESENTED_TENTHS.store(
        if active { (presented_fps * 10.0).round().clamp(0.0, 9999.0) as u32 } else { 0 },
        Ordering::Relaxed,
    );
    static LOGGED: Mutex<Option<std::time::Instant>> = Mutex::new(None);
    let mut logged = LOGGED.lock().unwrap_or_else(|e| e.into_inner());
    if logged.is_none_or(|t| t.elapsed() >= std::time::Duration::from_secs(5)) {
        *logged = Some(std::time::Instant::now());
        tracing::info!(active, "framegen: {real_fps:.1} real fps, {presented_fps:.1} fps presented");
    }
    *since = Some((std::time::Instant::now(), real, presented));
}

/// The presented frame rate in tenths, while frame generation runs.
pub(crate) fn presented_tenths() -> Option<u32> {
    if CONTEXT.load(Ordering::Acquire).is_null() || FAILED.load(Ordering::Relaxed) {
        return None;
    }
    let tenths = PRESENTED_TENTHS.load(Ordering::Relaxed);
    (tenths > 0).then_some(tenths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_generation_takes_the_upscale_depth_and_motion_vector_flags() {
        assert_eq!(flags(41), 8);
        assert_eq!(flags(0b1_0110), 0b1_0110);
        assert_eq!(flags(0x3ff), 0b1_1110);
    }
}
