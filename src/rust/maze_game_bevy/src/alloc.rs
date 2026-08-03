//! Live-allocation accounting.
//!
//! The diagnostics readout's `mem` figure is WebAssembly **linear memory**,
//! which can only ever grow — freeing every byte leaves it unchanged. That
//! makes it useless for the one question that matters when a game ends: was
//! anything actually released? This allocator answers it, by tracking the
//! running total of live bytes, which falls when memory is genuinely returned.
//!
//! Wrapping the system allocator costs two relaxed atomic operations per
//! allocation and deallocation. `Relaxed` is sufficient: the counter is a
//! diagnostic read from a single thread (the browser has no others here), and
//! nothing orders other memory against it.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

/// The system allocator plus a live-bytes counter.
pub struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    /// Counted as the delta rather than a free-then-alloc pair, so an in-place
    /// grow (which never calls `dealloc`) is still accounted correctly.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            if new_size >= layout.size() {
                LIVE_BYTES.fetch_add(new_size - layout.size(), Ordering::Relaxed);
            } else {
                LIVE_BYTES.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        new_ptr
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Bytes currently allocated and not yet freed.
///
/// Unlike the WebAssembly linear-memory size, this **falls** when memory is
/// returned, so it can show whether ending a game actually released the world.
/// Read it together with the linear-memory figure: the pair reads as "holding
/// this much of a ceiling that big".
///
/// # Examples
///
/// ```
/// use maze_game_bevy::live_bytes;
///
/// let before = live_bytes();
/// let held = vec![0u8; 1024 * 1024];
/// assert!(live_bytes() > before, "a live allocation raises the total");
/// drop(held);
/// ```
pub fn live_bytes() -> usize {
    LIVE_BYTES.load(Ordering::Relaxed)
}

/// Set when the host asks the game to end. `app.run()` moved the `App` into
/// winit's event loop and kept no handle, so nothing outside can reach it —
/// a flag polled from inside is the only route.
static STOP_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Asks the running game to shut down and release its world.
///
/// Nothing frees the game today: the browser only reclaims it when the document
/// itself is destroyed, which is asynchronous and unobservable from the app. This
/// gives the host an explicit way to end a session — and, paired with
/// [`live_bytes`], a way to check that ending it actually returned the memory.
///
/// Takes effect on the next frame, when [`stop_requested_system`] turns it into
/// an `AppExit`.
///
/// # Examples
///
/// ```
/// use maze_game_bevy::request_stop;
///
/// // Idempotent — asking twice is the same as asking once.
/// request_stop();
/// request_stop();
/// ```
pub fn request_stop() {
    STOP_REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Whether a stop has been asked for. Clears the flag, so a stop is consumed
/// once and a later run in the same module instance starts clean.
pub(crate) fn take_stop_request() -> bool {
    STOP_REQUESTED.swap(false, std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The counter is process-global while `cargo test` runs tests in parallel,
    // so another test's allocations and frees land between any two reads here.
    // These assertions therefore use a large allocation and generous margins:
    // they check the counter's *direction*, which is what it exists for, rather
    // than an exact delta it cannot promise. An earlier version demanded the
    // full allocation size and failed by 3 KB of concurrent noise.
    const PROBE_BYTES: usize = 16 * 1024 * 1024;
    const NOISE_MARGIN: usize = PROBE_BYTES / 2;

    #[test]
    fn an_allocation_raises_the_total_and_freeing_lowers_it() {
        // The whole point of this counter: unlike WebAssembly linear memory, it
        // comes back down.
        let before = live_bytes();
        let held: Vec<u8> = vec![7; PROBE_BYTES];
        let during = live_bytes();
        assert!(
            during >= before + NOISE_MARGIN,
            "a live allocation must raise the total: {before} -> {during}",
        );
        drop(held);
        let after = live_bytes();
        assert!(
            after + NOISE_MARGIN <= during,
            "freeing must lower the total: {during} -> {after}",
        );
    }

    #[test]
    fn a_stop_request_is_consumed_exactly_once() {
        // Consumed rather than latched, so a stop cannot leak into a later run
        // sharing the same module instance.
        take_stop_request(); // clear any residue from another test
        assert!(!take_stop_request(), "nothing asked for yet");
        request_stop();
        assert!(take_stop_request(), "the request is seen");
        assert!(!take_stop_request(), "and not seen twice");
    }

    #[test]
    fn growing_a_buffer_in_place_is_accounted_as_a_delta() {
        // A Vec grow goes through `realloc`, which may extend in place without
        // ever calling `dealloc` — counting it as a delta keeps that honest.
        let mut buffer: Vec<u8> = Vec::with_capacity(1024 * 1024);
        buffer.resize(1024 * 1024, 1);
        let small = live_bytes();
        buffer.resize(PROBE_BYTES, 2);
        let large = live_bytes();
        assert!(
            large >= small + NOISE_MARGIN,
            "a grow must raise the total: {small} -> {large}",
        );
        drop(buffer);
        let after = live_bytes();
        assert!(after + NOISE_MARGIN <= large, "dropping must lower it: {large} -> {after}");
    }
}
