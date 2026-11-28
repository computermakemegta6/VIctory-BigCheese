//! Frame pacing: the title sets a frame interval and waits for each frame
//! to begin. Ours is a clock started by the first interval set, ticking
//! once per interval; the token is the frame number. Each event kind
//! (origin, vblank, flip) happens once per tick, so two waits for the same
//! kind never end on the same tick: the title polls the origin
//! with "token on time-out" (starting its next frame at once) and is paced
//! by a blocking wait for the flip before each present.
//!
//! shortcut: event offsets are ignored (every event is at the tick); tie
//! the clock to presentation if a title paces by them.
//!
//! `VICTORY_FRAME_INTERVAL_MS=<ms>` ticks at that interval whatever the
//! title sets (16.667: 60 a second; 0: no wait at all, every wait ends at
//! once); unset, the title's interval as before. The loader gives the title
//! the matching frame rate too (`victory_loader::frame_rate`), as the title
//! would otherwise keep to its 30 with a limiter of its own.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::object::{HRESULT, S_FALSE, S_OK, This};

/// What a wait returns when its time is up first.
const TIMEOUT: HRESULT = S_FALSE;
/// The frame-origin event: the one that starts frames and makes tokens.
const ORIGIN: u32 = 0;
/// Make the frame's token even if the wait times out (the title starts the
/// frame early).
const TOKEN_ON_TIMEOUT: u32 = 2;
const SIXTIETH: Duration = Duration::from_micros(16_667);

/// Whether the loader let our interval apply ([`allow_interval`]): 0 not
/// said, 1 allowed, 2 refused. (From Neptune.)
static ALLOWED: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Said by the loader before the title runs: whether our interval
/// (`VICTORY_FRAME_INTERVAL_MS`) may apply. It may only when every read of
/// the title's frame-rate setting was patched to the matching rate, or the
/// user asked for the title to be left alone (`VICTORY_TITLE_FPS=off`, at
/// their own risk). Otherwise the title keeps its 30 (and its 1/30 s
/// simulation step) while our clock ticks faster: game time ran at twice
/// the wall clock. Unsaid, our interval does not apply either.
pub fn allow_interval(allowed: bool) {
    ALLOWED.store(if allowed { 1 } else { 2 }, Ordering::Relaxed);
}

/// `VICTORY_FRAME_INTERVAL_MS`: the interval asked for in place of the
/// title's (zero: no wait), if the loader allowed it ([`allow_interval`]).
/// Read once.
fn asked() -> Option<Duration> {
    static ASKED: std::sync::OnceLock<Option<Duration>> = std::sync::OnceLock::new();
    *ASKED.get_or_init(|| {
        let spec = std::env::var("VICTORY_FRAME_INTERVAL_MS").ok()?;
        if ALLOWED.load(Ordering::Relaxed) != 1 {
            tracing::warn!(
                spec,
                "VICTORY_FRAME_INTERVAL_MS refused: not every read of the title's frame rate was patched (VICTORY_TITLE_FPS=off to force it); the title's interval kept"
            );
            return None;
        }
        let ms = parse_ms(&spec);
        match ms {
            Some(d) => tracing::info!(interval = ?d, "frame interval of ours in place of the title's (VICTORY_FRAME_INTERVAL_MS)"),
            None => tracing::warn!(spec, "VICTORY_FRAME_INTERVAL_MS: expected milliseconds from 0 to 1000; the title's interval kept"),
        }
        ms
    })
}

/// `16.667` → 16,667 us; 0 → zero; negative, more than a second or not a
/// number → `None`.
fn parse_ms(spec: &str) -> Option<Duration> {
    let ms = spec.trim().parse::<f64>().ok().filter(|ms| ms.is_finite() && (0.0..=1000.0).contains(ms))?;
    Some(Duration::from_nanos((ms * 1e6).round() as u64))
}

struct Clock {
    start: Instant,
    interval: Duration,
    /// The tick number at `start` (ticks go on across interval changes).
    first: u64,
    /// The interval set (ours or the title's): `interval` is this but while
    /// the title plays a video ([`video_interval`]).
    base: Duration,
}

/// `VICTORY_FRAME_INTERVAL_VIDEO`: the interval our clock ticks at while the
/// title plays a Bink video, in place of `VICTORY_FRAME_INTERVAL_MS`'s (unset
/// or `1`: the title's own 33.333 ms; `0`: none, the patched interval always;
/// another number: those milliseconds). Only where our interval applies
/// ([`asked`]) and is shorter. (From Neptune.)
///
/// Why: the intro video was made for one decode a 33 ms frame. At 60 frames a
/// second the title draws it every 16.7 ms, and consecutive presents showed
/// two pictures in turn, seen as strobing; at the title's own 30 it does not.
/// A video playing is told by the title's calls into `Bink2Scarlett.dll`
/// ([`bink_called`], `victory_hle`'s stand-in in front of them): one of its
/// per-frame calls in the last [`VIDEO_HOLD`].
fn video_interval() -> Option<Duration> {
    static AT: std::sync::OnceLock<Option<Duration>> = std::sync::OnceLock::new();
    *AT.get_or_init(|| {
        let spec = std::env::var("VICTORY_FRAME_INTERVAL_VIDEO").ok();
        match spec.as_deref().map(str::trim) {
            None | Some("1") => Some(Duration::from_micros(33_333)),
            Some("0") => None,
            Some(ms) => match parse_ms(ms) {
                Some(d) if !d.is_zero() => Some(d),
                _ => {
                    tracing::warn!(spec = ms, "VICTORY_FRAME_INTERVAL_VIDEO: expected 0, 1 or milliseconds; the title's 33.333 ms taken");
                    Some(Duration::from_micros(33_333))
                }
            },
        }
    })
}

/// How long after the title's last per-frame call into Bink a video counts as playing.
const VIDEO_HOLD: Duration = Duration::from_millis(150);

/// The title's calls into Bink that the stand-in sees ([`bink_called`]).
#[derive(Clone, Copy, Debug)]
pub enum Bink {
    /// `BinkOpenWithOptions`.
    Open,
    /// `BinkClose`.
    Close,
    /// `BinkDoFrame`, `BinkDoFrameAsyncMulti`: a picture decoded (or begun).
    Decode,
    /// `BinkNextFrame`: on to the next picture.
    Next,
}

/// Microseconds since `epoch_us`'s start of the last per-frame Bink call (0:
/// none yet), and videos open (opened less closed).
static BINK_LAST: AtomicU64 = AtomicU64::new(0);
static BINK_OPEN: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
/// The title's per-frame Bink calls (decodes, nexts) in the whole run, said
/// when the clock goes back from the video's interval.
static BINK_COUNTS: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];

fn epoch_us() -> u64 {
    static EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    EPOCH.get_or_init(Instant::now).elapsed().as_micros() as u64 + 1
}

/// The title called into `Bink2Scarlett.dll` (`victory_hle`'s stand-in,
/// before the call goes on to the DLL unchanged).
pub fn bink_called(call: Bink) {
    match call {
        Bink::Open => {
            let open = BINK_OPEN.fetch_add(1, Ordering::Relaxed) + 1;
            tracing::info!(open, "the title opened a Bink video");
        }
        Bink::Close => {
            let open = BINK_OPEN.fetch_sub(1, Ordering::Relaxed) - 1;
            tracing::info!(open, "the title closed a Bink video");
            if open <= 0 {
                // Over at once, not 150 ms later.
                BINK_LAST.store(0, Ordering::Relaxed);
            }
        }
        Bink::Decode => {
            BINK_COUNTS[0].fetch_add(1, Ordering::Relaxed);
            BINK_LAST.store(epoch_us(), Ordering::Relaxed);
        }
        Bink::Next => {
            BINK_COUNTS[1].fetch_add(1, Ordering::Relaxed);
            BINK_LAST.store(epoch_us(), Ordering::Relaxed);
        }
    }
}

/// Whether the title plays a video now ([`VIDEO_HOLD`]).
fn video_playing() -> bool {
    let last = BINK_LAST.load(Ordering::Relaxed);
    last != 0 && epoch_us().saturating_sub(last) < VIDEO_HOLD.as_micros() as u64
}

/// Told when the clock goes to the video's interval (true) and back (false):
/// the loader gives the title's own frame time back to its data meanwhile.
static ON_VIDEO: std::sync::OnceLock<Box<dyn Fn(bool) + Send + Sync>> = std::sync::OnceLock::new();

/// Sets what is told when the clock goes to the video's interval and back.
pub fn on_video(f: Box<dyn Fn(bool) + Send + Sync>) {
    let _ = ON_VIDEO.set(f);
}

/// Microseconds the clock spent at the video's interval and at the other,
/// and when it last changed (`epoch_us`); whether it is at the video's now.
static IN_STATE: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];
static STATE_SINCE: AtomicU64 = AtomicU64::new(0);
static AT_VIDEO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The interval the clock should tick at now: `base`, or the video's while one plays.
fn wanted_interval(base: Duration, playing: bool) -> Duration {
    match (asked(), video_interval()) {
        (Some(a), Some(v)) if a < v && base == a && playing => v,
        _ => base,
    }
}

/// Starts the clock anew at `interval` from now, after the last tick handed out.
fn rebase(c: &mut Clock, interval: Duration) {
    c.first = LAST.iter().map(|t| t.load(Ordering::Relaxed)).max().unwrap_or(0);
    c.start = Instant::now();
    c.interval = interval;
}

/// Goes to the video's interval or back if that changed: said in the log,
/// the time in each counted, and the loader told ([`on_video`]).
fn follow_video(c: &mut Clock) {
    let want = wanted_interval(c.base, video_playing());
    if want == c.interval {
        return;
    }
    let video = want != c.base;
    rebase(c, want);
    // (An interval set anew by the title while a video plays changes nothing here.)
    if AT_VIDEO.load(Ordering::Relaxed) == video {
        return;
    }
    let now = epoch_us();
    let since = now.saturating_sub(STATE_SINCE.swap(now, Ordering::Relaxed));
    // The time since the last change was in the other state ([0]: the video's, [1]: ours).
    IN_STATE[usize::from(video)].fetch_add(since, Ordering::Relaxed);
    AT_VIDEO.store(video, Ordering::Relaxed);
    let ms = want.as_secs_f64() * 1e3;
    if video {
        tracing::info!("frame interval: a video is playing, the title's own {ms:.1} ms (VICTORY_FRAME_INTERVAL_VIDEO)");
    } else {
        let [at_video, at_ours] = [0, 1].map(|i| IN_STATE[i].load(Ordering::Relaxed) as f64 / 1e6);
        let [decodes, nexts] = [0, 1].map(|i| BINK_COUNTS[i].load(Ordering::Relaxed));
        tracing::info!(
            seconds_at_video = format_args!("{at_video:.1}"),
            seconds_at_ours = format_args!("{at_ours:.1}"),
            decodes,
            nexts,
            "frame interval: the video ended, {ms:.1} ms again (VICTORY_FRAME_INTERVAL_VIDEO)"
        );
    }
    if let Some(f) = ON_VIDEO.get() {
        f(video);
    }
}

static CLOCK: Mutex<Option<Clock>> = Mutex::new(None);
static FRAME: AtomicU64 = AtomicU64::new(0);

fn clock() -> std::sync::MutexGuard<'static, Option<Clock>> {
    CLOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// Device method: a frame every `length` microseconds. The period is how
/// many intervals a frame may take to reach the display (pipeline depth),
/// not part of the rate.
pub(crate) extern "system" fn set_interval(
    _: This,
    _output: This,
    length: u32,
    _period: u32,
    _flags: u32,
) -> HRESULT {
    let asked_by_title = match length {
        0 => SIXTIETH,
        us => Duration::from_micros(u64::from(us)),
    };
    let interval = asked().unwrap_or(asked_by_title);
    tracing::info!(?interval, title = ?asked_by_title, "frame interval");
    // A new interval starts from now, after the last tick handed out.
    let first = LAST
        .iter()
        .map(|t| t.load(Ordering::Relaxed))
        .max()
        .unwrap_or(0);
    *clock() = Some(Clock {
        start: Instant::now(),
        interval,
        first,
        base: interval,
    });
    S_OK
}

/// Device method: events are scheduled at every tick.
pub(crate) extern "system" fn schedule(
    _: This,
    _kind: u32,
    _offset: u32,
    _list: *const u8,
    _flags: u32,
) -> HRESULT {
    S_OK
}

/// Frames begun so far (frame waits that ended).
pub(crate) fn current() -> u64 {
    FRAME.load(Ordering::Relaxed)
}

/// For tests of what is done once a frame.
#[cfg(test)]
pub(crate) fn test_advance() {
    FRAME.fetch_add(1, Ordering::Relaxed);
}

/// Per event kind (origin, vblank, flip and later kinds): the last tick a
/// wait for it ended on.
static LAST: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];

fn last(kind: u32) -> &'static AtomicU64 {
    &LAST[(kind as usize).min(LAST.len() - 1)]
}

/// The next tick after now and after the last one a wait for `kind` ended
/// on (an event happens once per tick: a wait woken a little early must not
/// let the next one end on the same tick), and how long until it.
fn next_tick(kind: u32) -> (u64, Duration) {
    let mut clock = clock();
    let c = clock.get_or_insert_with(|| Clock {
        start: Instant::now(),
        interval: asked().unwrap_or(SIXTIETH),
        first: 0,
        base: asked().unwrap_or(SIXTIETH),
    });
    // The video's interval while the title plays one (`VICTORY_FRAME_INTERVAL_VIDEO`).
    follow_video(c);
    if c.interval.is_zero() {
        // No wait: every wait ends at once, on a tick of its own.
        let tick = last(kind).load(Ordering::Relaxed) + 1;
        return (tick, Duration::ZERO);
    }
    let elapsed = c.start.elapsed().as_nanos();
    let interval = c.interval.as_nanos().max(1);
    let tick =
        (c.first + (elapsed / interval + 1) as u64).max(last(kind).load(Ordering::Relaxed) + 1);
    let at = u128::from(tick - c.first) * interval;
    (
        tick,
        Duration::from_nanos(at.saturating_sub(elapsed) as u64),
    )
}

/// Parses `VICTORY_FPS_CAP` into a target frame interval (`None` = disabled).
fn parse_fps_cap(raw: Option<&str>) -> Option<std::time::Duration> {
    let fps: f64 = raw?.trim().parse().ok()?;
    if !(fps > 0.0 && fps <= 1000.0) {
        return None;
    }
    Some(std::time::Duration::from_secs_f64(1.0 / fps))
}

fn fps_cap_interval() -> Option<std::time::Duration> {
    static CAP: std::sync::OnceLock<Option<std::time::Duration>> = std::sync::OnceLock::new();
    *CAP.get_or_init(|| {
        let interval = parse_fps_cap(std::env::var("VICTORY_FPS_CAP").ok().as_deref());
        if let Some(interval) = interval {
            let ms = interval.as_secs_f64() * 1000.0;
            let fps = 1.0 / interval.as_secs_f64();
            tracing::info!(
                fps = format_args!("{fps:.2}"),
                interval_ms = format_args!("{ms:.3}"),
                "frame rate capped (VICTORY_FPS_CAP)"
            );
        }
        interval
    })
}

/// Paces the title's frame starts (its origin waits) to `VICTORY_FPS_CAP`
/// without carrying catch-up debt across slow frames, so dips never trigger
/// >target fps bursts afterward. On the title's thread: the host's submit
/// thread (where this was, in `Host::present`) never sleeps for it.
fn pace_fps_cap() {
    let Some(interval) = fps_cap_interval() else {
        return;
    };
    static LAST_PRESENT: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
    let mut last = LAST_PRESENT.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(prev) = *last {
        let deadline = prev + interval;
        while let Some(rem) = deadline.checked_duration_since(std::time::Instant::now()) {
            if rem > std::time::Duration::from_millis(2) {
                std::thread::sleep(rem - std::time::Duration::from_millis(2));
            } else if rem > std::time::Duration::from_micros(250) {
                std::thread::yield_now();
            } else {
                std::hint::spin_loop();
            }
        }
    }
    *last = Some(std::time::Instant::now());
}

/// Device method: blocks until the next tick (or the time-out); an origin
/// wait then hands out the new frame's token. Other events keep the token
/// the title passes. Ticks are absolute (start plus whole intervals), so
/// lateness never accumulates.
pub(crate) extern "system" fn wait(
    _: This,
    kind: u32,
    timeout_ms: u32,
    _list: *const u8,
    flags: u32,
    token: *mut u64,
) -> HRESULT {
    let origin = kind == ORIGIN;
    let (mut tick, mut wait) = next_tick(kind);
    let limit = Duration::from_millis(u64::from(timeout_ms));
    // A blocking wait for an event that has happened since the last wait
    // for it ended (a frame took longer than a tick) ends at once, on the
    // tick that passed: sleeping on to the next one cost 17 ms of every
    // 730 ms frame (run 58: "2.0 frame waits sleeping 16.8 ms"). Polls (a
    // time-out shorter than a tick) answer as before.
    // `VICTORY_LATE_WAITS=1` sleeps to the next tick, as before.
    static SLEEP: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*SLEEP.get_or_init(|| std::env::var_os("VICTORY_LATE_WAITS").is_some_and(|v| v == "1"))
        && limit >= wait
        && tick > last(kind).load(Ordering::Relaxed) + 1
    {
        tick -= 1;
        wait = Duration::ZERO;
    }
    let _called = crate::timeline::span("the title's frame wait");
    let hr = if wait > limit {
        std::thread::sleep(limit);
        TIMEOUT
    } else {
        if !wait.is_zero() {
            std::thread::sleep(wait);
        }
        last(kind).fetch_max(tick, Ordering::Relaxed);
        S_OK
    };
    #[cfg(all(windows, feature = "diagnostics"))]
    {
        use crate::copystat::{C, add};
        add(C::PaceWaits, 1);
        add(C::PaceUs, wait.min(limit).as_micros() as u64);
    }
    if origin && hr == S_OK {
        pace_fps_cap();
    }
    // A frame begun on a time-out takes no tick: its flip still does.
    if origin && (hr == S_OK || flags & TOKEN_ON_TIMEOUT != 0) {
        // Tokens count frames (consecutive), not ticks.
        let frame = FRAME.fetch_add(1, Ordering::Relaxed) + 1;
        pace::begun();
        if !token.is_null() {
            // SAFETY: checked non-null; the title passes its token.
            unsafe { token.write(frame) };
        }
    }
    #[cfg(feature = "diagnostics")]
    tracing::debug!(
        kind,
        timeout_ms,
        flags,
        hr,
        tick,
        wait_us = wait.as_micros() as u64,
        frame = current(),
        "frame wait"
    );
    hr
}

/// How even the frames are (`VICTORY_TIMER_LOG=1`): the times between frame
/// beginnings, logged every 5 s by bands, with the pairs of a short frame
/// next to a long one (a 16/33 alternation shows as many of them).
mod pace {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    /// Upper edges of the bands, in milliseconds (the last band is open).
    const BANDS: [f64; 6] = [12.0, 20.0, 28.0, 38.0, 55.0, 70.0];

    struct Stats {
        since: Instant,
        last: Option<Instant>,
        last_ms: f64,
        bands: [u32; 7],
        sum_ms: f64,
        max_ms: f64,
        n: u32,
        alternations: u32,
    }

    static STATS: Mutex<Option<Stats>> = Mutex::new(None);

    fn on() -> bool {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var("VICTORY_TIMER_LOG").is_ok_and(|v| v.trim() == "1"))
    }

    pub(super) fn begun() {
        if !on() {
            return;
        }
        let now = Instant::now();
        let mut stats = STATS.lock().unwrap_or_else(|e| e.into_inner());
        let s = stats.get_or_insert_with(|| Stats {
            since: now,
            last: None,
            last_ms: 0.0,
            bands: [0; 7],
            sum_ms: 0.0,
            max_ms: 0.0,
            n: 0,
            alternations: 0,
        });
        if let Some(last) = s.last.replace(now) {
            let ms = (now - last).as_secs_f64() * 1e3;
            let band = BANDS.iter().position(|&edge| ms < edge).unwrap_or(BANDS.len());
            s.bands[band] += 1;
            s.sum_ms += ms;
            s.max_ms = s.max_ms.max(ms);
            if s.n > 0 && (ms < 20.0 && s.last_ms > 28.0 || ms > 28.0 && s.last_ms < 20.0) {
                s.alternations += 1;
            }
            s.last_ms = ms;
            s.n += 1;
        }
        if now - s.since >= Duration::from_secs(5) && s.n > 0 {
            tracing::info!(
                frames = s.n,
                mean_ms = format_args!("{:.2}", s.sum_ms / f64::from(s.n)),
                max_ms = format_args!("{:.1}", s.max_ms),
                bands = ?s.bands,
                alternations = s.alternations,
                "frame pacing: times between frame beginnings (bands <12, <20, <28, <38, <55, <70, more ms)"
            );
            s.since = now;
            s.bands = [0; 7];
            s.sum_ms = 0.0;
            s.max_ms = 0.0;
            s.n = 0;
            s.alternations = 0;
        }
    }
}

/// Device method: no statistics are kept, so none are returned.
pub(crate) extern "system" fn statistics(
    _: This,
    _token: u64,
    _kinds: u32,
    count: *mut u32,
    _out: *mut u8,
) -> HRESULT {
    if !count.is_null() {
        // SAFETY: checked non-null.
        unsafe { count.write(0) };
    }
    S_OK
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_fps_cap_valid_and_disabled() {
        assert_eq!(super::parse_fps_cap(None), None);
        assert_eq!(super::parse_fps_cap(Some("")), None);
        assert_eq!(super::parse_fps_cap(Some("  ")), None);
        assert_eq!(super::parse_fps_cap(Some("0")), None);
        assert_eq!(super::parse_fps_cap(Some("-30")), None);
        assert_eq!(super::parse_fps_cap(Some("NaN")), None);
        assert_eq!(super::parse_fps_cap(Some("inf")), None);
        assert_eq!(super::parse_fps_cap(Some("1000.1")), None);
        assert_eq!(super::parse_fps_cap(Some("abc")), None);

        let d30 = super::parse_fps_cap(Some("30")).expect("30 fps should parse");
        assert!((d30.as_secs_f64() - (1.0 / 30.0)).abs() < 1e-6);

        let d60 = super::parse_fps_cap(Some(" 60 ")).expect("60 fps should parse");
        assert!((d60.as_secs_f64() - (1.0 / 60.0)).abs() < 1e-6);
    }

    use super::*;

    #[test]
    fn the_switch_reads_milliseconds() {
        assert_eq!(parse_ms("16.667"), Some(Duration::from_micros(16_667)));
        assert_eq!(parse_ms(" 8.333 "), Some(Duration::from_micros(8_333)));
        assert_eq!(parse_ms("0"), Some(Duration::ZERO));
        assert_eq!(parse_ms("-1"), None);
        assert_eq!(parse_ms("1001"), None);
        assert_eq!(parse_ms("NaN"), None);
        assert_eq!(parse_ms("fast"), None);
    }

    #[test]
    fn waits_end_on_ticks_and_count_frames() {
        let null = std::ptr::null();
        assert_eq!(set_interval(null, null, 2_000, 1, 0), S_OK);
        let mut first = 0;
        assert_eq!(wait(null, 0, 1_000, std::ptr::null(), 0, &mut first), S_OK);
        let before = Instant::now();
        let mut second = 0;
        assert_eq!(wait(null, 0, 1_000, std::ptr::null(), 0, &mut second), S_OK);
        assert!(second > first);
        // At the tick, not at the time-out (a busy PC may wake it late).
        assert!(before.elapsed() <= Duration::from_millis(500));
        assert_eq!(wait(null, 0, 0, std::ptr::null(), 0, &mut second), TIMEOUT);
        // Tokens count frames, a frame started early on a time-out too.
        let mut early = 0;
        let flags = TOKEN_ON_TIMEOUT;
        assert_eq!(
            wait(null, 0, 0, std::ptr::null(), flags, &mut early),
            TIMEOUT
        );
        let mut next = 0;
        assert_eq!(wait(null, 0, 1_000, std::ptr::null(), 0, &mut next), S_OK);
        assert!(early == second + 1 && next == early + 1);
        assert!(LAST[0].load(Ordering::Relaxed) >= 2);

        // The title's loop (the clock is global, so in the same test): an
        // origin poll that times out (and starts the frame), then a
        // blocking flip wait. Each flip takes its own tick.
        const FLIP: u32 = 2;
        // (The ticks are far apart, and it may take a few tries: a busy PC
        // wakes a wait tens of milliseconds late at times.)
        assert_eq!(set_interval(null, null, 60_000, 3, 0), S_OK);
        let mut token = 0;
        let mut four = || -> Vec<u64> {
            (0..4)
                .map(|_| {
                    let _ = wait(null, 0, 0, std::ptr::null(), TOKEN_ON_TIMEOUT, &mut token);
                    assert_eq!(
                        wait(null, FLIP, u32::MAX, std::ptr::null(), 0, &mut token),
                        S_OK
                    );
                    LAST[2].load(Ordering::Relaxed)
                })
                .collect()
        };
        let mut flips = four();
        for _ in 0..10 {
            if flips.windows(2).all(|w| w[1] == w[0] + 1) {
                break;
            }
            flips = four();
        }
        assert!(flips.windows(2).all(|w| w[1] == w[0] + 1), "{flips:?}");
        // Even when a flip wait begins on the tick another just ended on.
        let (tick, _) = next_tick(FLIP);
        assert!(tick > flips[3]);
    }
}
