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

/// The running total of live bytes, kept separate from the allocator itself so
/// the arithmetic can be tested on a local instance. Testing it through the
/// global counter is not possible: the whole test suite allocates into it
/// concurrently, and the interference is the same order of magnitude as any
/// probe a test could make.
pub(crate) struct LiveCounter(AtomicUsize);

impl LiveCounter {
    const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    fn added(&self, bytes: usize) {
        self.0.fetch_add(bytes, Ordering::Relaxed);
    }

    fn removed(&self, bytes: usize) {
        self.0.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Applies a reallocation as a delta. `realloc` may extend a block in place
    /// without ever calling `dealloc`, so a free-then-alloc pair would be wrong.
    fn resized(&self, from: usize, to: usize) {
        if to >= from {
            self.added(to - from);
        } else {
            self.removed(from - to);
        }
    }

    fn get(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}

static LIVE_BYTES: LiveCounter = LiveCounter::new();

/// The system allocator plus a live-bytes counter.
pub struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            LIVE_BYTES.added(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE_BYTES.removed(layout.size());
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            LIVE_BYTES.added(layout.size());
        }
        ptr
    }

    /// Counted as the delta rather than a free-then-alloc pair, so an in-place
    /// grow (which never calls `dealloc`) is still accounted correctly.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            LIVE_BYTES.resized(layout.size(), new_size);
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
    LIVE_BYTES.get()
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

    // The counter's arithmetic is exercised on a LOCAL instance. Driving the
    // global one from a test cannot work: 282 tests allocate into it in
    // parallel, and an earlier attempt to absorb that with tolerances failed
    // because the interference came from the sibling allocator test using an
    // identically-sized probe — noise the same magnitude as the signal.

    #[test]
    fn the_counter_rises_on_allocation_and_falls_on_release() {
        let counter = LiveCounter::new();
        assert_eq!(counter.get(), 0);
        counter.added(4096);
        counter.added(1024);
        assert_eq!(counter.get(), 5120);
        counter.removed(4096);
        assert_eq!(counter.get(), 1024, "releasing must bring the total back down");
    }

    #[test]
    fn a_reallocation_is_counted_as_a_delta() {
        // realloc may extend a block in place and never call dealloc, so
        // counting it as free-then-alloc would drift.
        let counter = LiveCounter::new();
        counter.added(1000);
        counter.resized(1000, 4000);
        assert_eq!(counter.get(), 4000, "a grow adds only the difference");
        counter.resized(4000, 1500);
        assert_eq!(counter.get(), 1500, "a shrink removes only the difference");
        counter.resized(1500, 1500);
        assert_eq!(counter.get(), 1500, "an unchanged size is a no-op");
    }

    #[test]
    fn the_global_counter_is_wired_up() {
        // Deliberately weak: the only claim that can be made about the global
        // counter while the suite runs in parallel is that it is counting.
        assert!(live_bytes() > 0, "the test binary itself has live allocations");
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

}
