use data_model::{Maze, MazeDefinition, MazePoint};
use maze::{Generator, GenerationAlgorithm, GeneratorOptions, MazeSolution, MazeSolver};
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// ──────────────────────────────────────────────────────────────────────────────
// Opaque object wrappers
// ──────────────────────────────────────────────────────────────────────────────

/// Wrapper around a [`Maze`] object, exposed to C# via P/Invoke handles.
///
/// Created via [`maze_c_new_maze`] and freed via [`maze_c_free_maze`].
/// All operations on a `MazeC` are performed through the `maze_c_maze_*`
/// family of functions.
///
/// # Examples
///
/// Create a new maze, resize it to 10 rows × 5 columns, and assert its dimensions.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// assert!(!ptr.is_null());
/// maze_c_maze_resize(ptr, 10, 5);
/// assert_eq!(maze_c_maze_get_row_count(ptr), 10);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 5);
/// maze_c_free_maze(ptr);
/// ```
pub struct MazeC {
    pub maze: Maze,
}

/// Options used to drive maze generation.
///
/// Created via [`maze_c_new_generator_options`], mutated via setter functions,
/// passed to [`maze_c_maze_generate`], freed via [`maze_c_free_generator_options`].
///
/// Sentinel values for optional fields:
/// - `start_row` / `start_col` / `finish_row` / `finish_col`: `u32::MAX` = use default
/// - `min_spine_length`: `0` = use default (`(row_count + col_count) / 2`)
/// - `max_retries`: `0` = use default (100)
/// - `branch_from_finish`: `0` = false (default), `1` = true
///
/// # Examples
///
/// Create generator options for a 10 × 10 maze, set an optional start cell, then free.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// assert!(!opts.is_null());
/// maze_c_generator_options_set_start(opts, 0, 0);
/// maze_c_generator_options_set_finish(opts, 9, 9);
/// maze_c_free_generator_options(opts);
/// ```
pub struct MazeCGeneratorOptions {
    pub row_count: u32,
    pub col_count: u32,
    pub algorithm: u32,
    pub seed: u64,
    pub start_row: u32,
    pub start_col: u32,
    pub finish_row: u32,
    pub finish_col: u32,
    pub min_spine_length: u32,
    pub max_retries: u32,
    pub branch_from_finish: u8,
}

// ──────────────────────────────────────────────────────────────────────────────
// Thread-local last-error storage
// ──────────────────────────────────────────────────────────────────────────────

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(msg).ok();
    });
}

fn clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

// ──────────────────────────────────────────────────────────────────────────────
// Object allocation counter
// ──────────────────────────────────────────────────────────────────────────────

static mut TOTAL_NUM_OBJECTS_ALLOCATED: i64 = 0;

fn increment_num_objects_allocated() {
    unsafe {
        TOTAL_NUM_OBJECTS_ALLOCATED += 1;
    }
}

fn decrement_num_objects_allocated() {
    unsafe {
        TOTAL_NUM_OBJECTS_ALLOCATED -= 1;
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Error / string helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Returns a pointer to the last error message set by a `maze_c_*` call,
/// or `null` if no error has been set since the last successful call.
///
/// The returned pointer is valid until the next `maze_c_*` call on this thread.
/// **Do not free** this pointer — it is owned by the thread-local storage.
///
/// # Examples
///
/// Trigger an out-of-bounds error, then retrieve and print the error message.
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CStr;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
///
/// // Request a cell outside the maze bounds to trigger an error.
/// let mut cell_type: u32 = 0;
/// let ok = unsafe { maze_c_maze_get_cell_type(ptr, 99, 0, &mut cell_type) };
/// assert_eq!(ok, 0);
///
/// let err_ptr = maze_c_get_last_error();
/// assert!(!err_ptr.is_null());
/// let msg = unsafe { CStr::from_ptr(err_ptr) }.to_string_lossy();
/// assert!(msg.contains("row index"));
///
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| match e.borrow().as_ref() {
        Some(cstr) => cstr.as_ptr(),
        None => std::ptr::null(),
    })
}

/// Frees a `*mut c_char` string that was returned by a `maze_c_*` function
/// (e.g. [`maze_c_maze_to_json`]).
///
/// # Safety
///
/// `ptr` must be a non-null pointer previously returned by a `maze_c_*` function
/// that allocates a string (e.g. [`maze_c_maze_to_json`]).
/// Calling this function twice on the same pointer is undefined behaviour.
/// Passing a null pointer is safe and has no effect.
///
/// # Examples
///
/// Serialise a maze to JSON, read the string, then free it.
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CStr;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 2, 2);
///
/// let json_ptr = maze_c_maze_to_json(ptr);
/// assert!(!json_ptr.is_null());
///
/// let json = unsafe { CStr::from_ptr(json_ptr) }.to_string_lossy().into_owned();
/// assert!(json.contains("grid"));
///
/// unsafe { maze_c_free_string(json_ptr) };
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub unsafe extern "C" fn maze_c_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Memory tracking (mirrors maze_wasm API; sized-memory always 0 for native)
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the total sized memory currently allocated.
///
/// Always returns `0` for `maze_c` — sized memory is a wasm-specific concept.
///
/// # Examples
///
/// Assert that sized memory is always zero regardless of how many mazes are allocated.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 10, 10);
/// assert_eq!(maze_c_get_sized_memory_used(), 0);
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_get_sized_memory_used() -> i64 {
    0
}

/// Returns the number of heap-allocated maze objects currently alive.
///
/// # Examples
///
/// Assert that the object count increments when a maze is created and
/// decrements when it is freed.
///
/// ```rust
/// use maze_c::*;
///
/// let before = maze_c_get_num_objects_allocated();
/// let ptr = maze_c_new_maze();
/// assert_eq!(maze_c_get_num_objects_allocated(), before + 1);
/// maze_c_free_maze(ptr);
/// assert_eq!(maze_c_get_num_objects_allocated(), before);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_get_num_objects_allocated() -> i64 {
    unsafe { TOTAL_NUM_OBJECTS_ALLOCATED }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeC — lifecycle
// ──────────────────────────────────────────────────────────────────────────────

/// Creates a new, empty [`MazeC`].
///
/// Returns a non-null pointer on success. The caller must eventually free it
/// with [`maze_c_free_maze`].
///
/// # Examples
///
/// Create a new maze and assert it is initially empty (0 × 0).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// assert!(!ptr.is_null());
/// assert!(maze_c_maze_is_empty(ptr));
/// assert_eq!(maze_c_maze_get_row_count(ptr), 0);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 0);
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_new_maze() -> *mut MazeC {
    let mw = Box::new(MazeC {
        maze: Maze::new(MazeDefinition::new(0, 0)),
    });
    increment_num_objects_allocated();
    Box::into_raw(mw)
}

/// Frees a [`MazeC`] pointer previously returned by [`maze_c_new_maze`].
///
/// Passing `null` is safe and has no effect.
///
/// # Examples
///
/// Create a maze, use it, then free it. Freeing `null` is also safe.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 5, 5);
/// maze_c_free_maze(ptr);
///
/// // Freeing null is a no-op.
/// maze_c_free_maze(std::ptr::null_mut());
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_free_maze(ptr: *mut MazeC) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
        decrement_num_objects_allocated();
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeWasm — queries
// ──────────────────────────────────────────────────────────────────────────────

/// Returns `true` if the maze has no cells (0 × 0).
///
/// # Examples
///
/// Assert that a newly created maze is empty, and no longer empty after resizing.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// assert!(maze_c_maze_is_empty(ptr));
/// maze_c_maze_resize(ptr, 1, 2);
/// assert!(!maze_c_maze_is_empty(ptr));
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_is_empty(ptr: *mut MazeC) -> bool {
    let mw = unsafe { &*ptr };
    mw.maze.definition.is_empty()
}

/// Returns the number of rows.
///
/// # Examples
///
/// Create a new maze and assert the row count is 0. Then resize it to
/// 10 rows × 5 columns and assert the row count is 10.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// assert_eq!(maze_c_maze_get_row_count(ptr), 0);
/// maze_c_maze_resize(ptr, 10, 5);
/// assert_eq!(maze_c_maze_get_row_count(ptr), 10);
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_get_row_count(ptr: *mut MazeC) -> u32 {
    let mw = unsafe { &*ptr };
    mw.maze.definition.row_count() as u32
}

/// Returns the number of columns.
///
/// # Examples
///
/// Create a new maze and assert the column count is 0. Then resize it to
/// 10 rows × 5 columns and assert the column count is 5.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// assert_eq!(maze_c_maze_get_col_count(ptr), 0);
/// maze_c_maze_resize(ptr, 10, 5);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 5);
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_get_col_count(ptr: *mut MazeC) -> u32 {
    let mw = unsafe { &*ptr };
    mw.maze.definition.col_count() as u32
}

/// Gets the cell type at `(row, col)`.
///
/// Writes the cell-type value into `*out_cell_type` and returns `1` on success,
/// or `0` on error (out-of-bounds) with the error message stored via
/// [`maze_c_get_last_error`].
///
/// Cell-type values mirror `MazeCellTypeWasm` in `maze_wasm`:
/// `0` = Empty, `1` = Start, `2` = Finish, `3` = Wall.
///
/// # Safety
///
/// `ptr` must be a valid non-null pointer to a `MazeC` previously returned by
/// [`maze_c_new_maze`]. `out_cell_type` must be a valid writable pointer.
///
/// # Examples
///
/// Resize a maze to 10 × 5 and assert that a cell is initially of type Empty (0).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 10, 5);
///
/// let mut cell_type: u32 = 99;
/// let ok = unsafe { maze_c_maze_get_cell_type(ptr, 1, 2, &mut cell_type) };
/// assert_eq!(ok, 1);
/// assert_eq!(cell_type, 0); // Empty
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_get_cell_type(
    ptr: *mut MazeC,
    row: u32,
    col: u32,
    out_cell_type: *mut u32,
) -> u8 {
    clear_last_error();
    let mw = &*ptr;
    let r = row as usize;
    let c = col as usize;
    if r >= mw.maze.definition.row_count() {
        set_last_error(&format!("row index ({r}) out of bounds"));
        return 0;
    }
    if c >= mw.maze.definition.col_count() {
        set_last_error(&format!("column index ({c}) out of bounds"));
        return 0;
    }
    let cell_type: u32 = match mw.maze.definition.grid[r][c] {
        'S' => 1,
        'F' => 2,
        'W' => 3,
        _ => 0,
    };
    if !out_cell_type.is_null() {
        *out_cell_type = cell_type;
    }
    1
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeWasm — start / finish cells
// ──────────────────────────────────────────────────────────────────────────────

/// Sets the start cell. Returns `1` on success, `0` on error.
///
/// # Safety
///
/// `ptr` must be a valid non-null pointer to a `MazeC` previously returned by
/// [`maze_c_new_maze`].
///
/// # Examples
///
/// Resize a maze to 10 × 5, set the start cell at (1, 2), and assert the
/// cell type changes to Start (1).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 10, 5);
///
/// let ok = unsafe { maze_c_maze_set_start_cell(ptr, 1, 2) };
/// assert_eq!(ok, 1);
///
/// let mut cell_type: u32 = 0;
/// unsafe { maze_c_maze_get_cell_type(ptr, 1, 2, &mut cell_type) };
/// assert_eq!(cell_type, 1); // Start
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_set_start_cell(
    ptr: *mut MazeC,
    row: u32,
    col: u32,
) -> u8 {
    clear_last_error();
    let mw = &mut *ptr;
    match mw.maze.definition.set_start(Some(MazePoint {
        row: row as usize,
        col: col as usize,
    })) {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Gets the start cell, writing its row/col into `*out_row` / `*out_col`.
/// Returns `1` on success, `0` if no start cell is defined.
///
/// # Safety
///
/// `ptr` must be a valid non-null pointer to a `MazeC` previously returned by
/// [`maze_c_new_maze`]. `out_row` and `out_col` may be null; non-null pointers
/// must be valid writable locations.
///
/// # Examples
///
/// Set the start cell at (1, 2), then retrieve it and assert the coordinates.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 10, 5);
/// unsafe { maze_c_maze_set_start_cell(ptr, 1, 2) };
///
/// let mut row: u32 = 99;
/// let mut col: u32 = 99;
/// let ok = unsafe { maze_c_maze_get_start_cell(ptr, &mut row, &mut col) };
/// assert_eq!(ok, 1);
/// assert_eq!(row, 1);
/// assert_eq!(col, 2);
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_get_start_cell(
    ptr: *mut MazeC,
    out_row: *mut u32,
    out_col: *mut u32,
) -> u8 {
    clear_last_error();
    let mw = &*ptr;
    match mw.maze.definition.get_start() {
        Some(pt) => {
            if !out_row.is_null() {
                *out_row = pt.row as u32;
            }
            if !out_col.is_null() {
                *out_col = pt.col as u32;
            }
            1
        }
        None => {
            set_last_error("no start cell defined");
            0
        }
    }
}

/// Sets the finish cell. Returns `1` on success, `0` on error.
///
/// # Safety
///
/// `ptr` must be a valid non-null pointer to a `MazeC` previously returned by
/// [`maze_c_new_maze`].
///
/// # Examples
///
/// Resize a maze to 10 × 5, set the finish cell at (3, 4), and assert the
/// cell type changes to Finish (2).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 10, 5);
///
/// let ok = unsafe { maze_c_maze_set_finish_cell(ptr, 3, 4) };
/// assert_eq!(ok, 1);
///
/// let mut cell_type: u32 = 0;
/// unsafe { maze_c_maze_get_cell_type(ptr, 3, 4, &mut cell_type) };
/// assert_eq!(cell_type, 2); // Finish
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_set_finish_cell(
    ptr: *mut MazeC,
    row: u32,
    col: u32,
) -> u8 {
    clear_last_error();
    let mw = &mut *ptr;
    match mw.maze.definition.set_finish(Some(MazePoint {
        row: row as usize,
        col: col as usize,
    })) {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Gets the finish cell, writing its row/col into `*out_row` / `*out_col`.
/// Returns `1` on success, `0` if no finish cell is defined.
///
/// # Safety
///
/// `ptr` must be a valid non-null pointer to a `MazeC` previously returned by
/// [`maze_c_new_maze`]. `out_row` and `out_col` may be null; non-null pointers
/// must be valid writable locations.
///
/// # Examples
///
/// Set the finish cell at (9, 4), then retrieve it and assert the coordinates.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 10, 5);
/// unsafe { maze_c_maze_set_finish_cell(ptr, 9, 4) };
///
/// let mut row: u32 = 99;
/// let mut col: u32 = 99;
/// let ok = unsafe { maze_c_maze_get_finish_cell(ptr, &mut row, &mut col) };
/// assert_eq!(ok, 1);
/// assert_eq!(row, 9);
/// assert_eq!(col, 4);
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_get_finish_cell(
    ptr: *mut MazeC,
    out_row: *mut u32,
    out_col: *mut u32,
) -> u8 {
    clear_last_error();
    let mw = &*ptr;
    match mw.maze.definition.get_finish() {
        Some(pt) => {
            if !out_row.is_null() {
                *out_row = pt.row as u32;
            }
            if !out_col.is_null() {
                *out_col = pt.col as u32;
            }
            1
        }
        None => {
            set_last_error("no finish cell defined");
            0
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeWasm — cell mutations
// ──────────────────────────────────────────────────────────────────────────────

fn set_cell_range(
    ptr: *mut MazeC,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
    ch: char,
) -> u8 {
    let mw = unsafe { &mut *ptr };
    match mw.maze.definition.set_value(
        MazePoint {
            row: start_row as usize,
            col: start_col as usize,
        },
        MazePoint {
            row: end_row as usize,
            col: end_col as usize,
        },
        ch,
    ) {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Sets a rectangular range of cells to walls. Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Resize a maze to 10 × 5, set cells (0,1) to (0,3) as walls, and assert
/// their cell types are Wall (3).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 10, 5);
///
/// let ok = maze_c_maze_set_wall_cells(ptr, 0, 1, 0, 3);
/// assert_eq!(ok, 1);
///
/// for col in 1u32..=3 {
///     let mut ct: u32 = 0;
///     unsafe { maze_c_maze_get_cell_type(ptr, 0, col, &mut ct) };
///     assert_eq!(ct, 3, "expected Wall at (0, {col})");
/// }
///
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_maze_set_wall_cells(
    ptr: *mut MazeC,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> u8 {
    clear_last_error();
    set_cell_range(ptr, start_row, start_col, end_row, end_col, 'W')
}

/// Clears (empties) a rectangular range of cells. Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Set a 3 × 3 maze entirely to walls, then clear all cells and assert they
/// become Empty (0).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
/// maze_c_maze_set_wall_cells(ptr, 0, 0, 2, 2);
///
/// let ok = maze_c_maze_clear_cells(ptr, 0, 0, 2, 2);
/// assert_eq!(ok, 1);
///
/// for r in 0u32..3 {
///     for c in 0u32..3 {
///         let mut ct: u32 = 99;
///         unsafe { maze_c_maze_get_cell_type(ptr, r, c, &mut ct) };
///         assert_eq!(ct, 0, "expected Empty at ({r}, {c})");
///     }
/// }
///
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_maze_clear_cells(
    ptr: *mut MazeC,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> u8 {
    clear_last_error();
    set_cell_range(ptr, start_row, start_col, end_row, end_col, ' ')
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeWasm — resize / reset
// ──────────────────────────────────────────────────────────────────────────────

/// Resizes the maze to `new_row_count` × `new_col_count`.
///
/// # Examples
///
/// Create a new maze, print its dimensions (0 × 0), resize it to 10 × 5,
/// and assert the new dimensions.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// assert_eq!(maze_c_maze_get_row_count(ptr), 0);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 0);
///
/// maze_c_maze_resize(ptr, 10, 5);
/// assert_eq!(maze_c_maze_get_row_count(ptr), 10);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 5);
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_resize(
    ptr: *mut MazeC,
    new_row_count: u32,
    new_col_count: u32,
) {
    let mw = unsafe { &mut *ptr };
    mw.maze
        .definition
        .resize(new_row_count as usize, new_col_count as usize);
}

/// Resets the maze to an empty (0 × 0) state.
///
/// # Examples
///
/// Resize a maze to 10 × 5 and then reset it, asserting it returns to empty.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 10, 5);
/// assert!(!maze_c_maze_is_empty(ptr));
///
/// maze_c_maze_reset(ptr);
/// assert!(maze_c_maze_is_empty(ptr));
/// assert_eq!(maze_c_maze_get_row_count(ptr), 0);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 0);
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_reset(ptr: *mut MazeC) {
    let mw = unsafe { &mut *ptr };
    mw.maze.definition.reset();
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeWasm — row / column operations
// ──────────────────────────────────────────────────────────────────────────────

/// Inserts `count` rows starting at `start_row`. Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Resize a maze to 3 × 3, insert 2 rows at index 1, and assert the row count
/// increases to 5.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
/// assert_eq!(maze_c_maze_get_row_count(ptr), 3);
///
/// let ok = maze_c_maze_insert_rows(ptr, 1, 2);
/// assert_eq!(ok, 1);
/// assert_eq!(maze_c_maze_get_row_count(ptr), 5);
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_insert_rows(
    ptr: *mut MazeC,
    start_row: u32,
    count: u32,
) -> u8 {
    clear_last_error();
    let mw = unsafe { &mut *ptr };
    match mw
        .maze
        .definition
        .insert_rows(start_row as usize, count as usize)
    {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Deletes `count` rows starting at `start_row`. Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Resize a maze to 5 × 3, delete 2 rows starting at index 1, and assert the
/// row count decreases to 3.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 5, 3);
/// assert_eq!(maze_c_maze_get_row_count(ptr), 5);
///
/// let ok = maze_c_maze_delete_rows(ptr, 1, 2);
/// assert_eq!(ok, 1);
/// assert_eq!(maze_c_maze_get_row_count(ptr), 3);
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_delete_rows(
    ptr: *mut MazeC,
    start_row: u32,
    count: u32,
) -> u8 {
    clear_last_error();
    let mw = unsafe { &mut *ptr };
    match mw
        .maze
        .definition
        .delete_rows(start_row as usize, count as usize)
    {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Inserts `count` columns starting at `start_col`. Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Resize a maze to 3 × 3, insert 3 columns at index 1, and assert the column
/// count increases to 6.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 3);
///
/// let ok = maze_c_maze_insert_cols(ptr, 1, 3);
/// assert_eq!(ok, 1);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 6);
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_insert_cols(
    ptr: *mut MazeC,
    start_col: u32,
    count: u32,
) -> u8 {
    clear_last_error();
    let mw = unsafe { &mut *ptr };
    match mw
        .maze
        .definition
        .insert_cols(start_col as usize, count as usize)
    {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Deletes `count` columns starting at `start_col`. Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Resize a maze to 3 × 5, delete 2 columns starting at index 1, and assert
/// the column count decreases to 3.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 5);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 5);
///
/// let ok = maze_c_maze_delete_cols(ptr, 1, 2);
/// assert_eq!(ok, 1);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 3);
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_delete_cols(
    ptr: *mut MazeC,
    start_col: u32,
    count: u32,
) -> u8 {
    clear_last_error();
    let mw = unsafe { &mut *ptr };
    match mw
        .maze
        .definition
        .delete_cols(start_col as usize, count as usize)
    {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeWasm — JSON serialisation
// ──────────────────────────────────────────────────────────────────────────────

/// Reinitialises a maze from a null-terminated UTF-8 JSON string.
/// Returns `1` on success, `0` on error.
///
/// # Safety
///
/// `ptr` must be a valid non-null pointer to a `MazeC` previously returned by
/// [`maze_c_new_maze`]. `json` must be a valid non-null pointer to a
/// null-terminated UTF-8 string for the lifetime of the call.
///
/// # Examples
///
/// Initialise a maze from a JSON string and assert the resulting dimensions.
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let ptr = maze_c_new_maze();
/// let json = CString::new(
///     r#"{"id":"","name":"","definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#
/// ).unwrap();
///
/// let ok = unsafe { maze_c_maze_from_json(ptr, json.as_ptr()) };
/// assert_eq!(ok, 1);
/// assert_eq!(maze_c_maze_get_row_count(ptr), 2);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 3);
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_from_json(
    ptr: *mut MazeC,
    json: *const c_char,
) -> u8 {
    clear_last_error();
    if json.is_null() {
        set_last_error("json pointer is null");
        return 0;
    }
    let json_str = match CStr::from_ptr(json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&e.to_string());
            return 0;
        }
    };
    let mw = &mut *ptr;
    match mw.maze.from_json(json_str) {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

/// Converts a maze to a JSON string.
///
/// Returns a null-terminated UTF-8 string on success, or `null` on error.
/// The caller must free the returned string with [`maze_c_free_string`].
///
/// # Examples
///
/// Resize a maze, set a wall, serialise to JSON and assert the output contains
/// the `"grid"` key.
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CStr;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 6, 5);
/// maze_c_maze_set_wall_cells(ptr, 0, 1, 2, 4);
///
/// let json_ptr = maze_c_maze_to_json(ptr);
/// assert!(!json_ptr.is_null());
///
/// let json = unsafe { CStr::from_ptr(json_ptr) }.to_string_lossy().into_owned();
/// assert!(json.contains("grid"));
///
/// unsafe { maze_c_free_string(json_ptr) };
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_to_json(ptr: *mut MazeC) -> *mut c_char {
    clear_last_error();
    let mw = unsafe { &*ptr };
    match mw.maze.to_json() {
        Ok(s) => match CString::new(s) {
            Ok(cs) => cs.into_raw(),
            Err(e) => {
                set_last_error(&e.to_string());
                std::ptr::null_mut()
            }
        },
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeSolution — solve / path points / free
// ──────────────────────────────────────────────────────────────────────────────

/// Solves the maze.
///
/// Returns a non-null `*mut MazeSolution` on success.
/// Returns `null` on error (check [`maze_c_get_last_error`] for the message).
/// The returned pointer must be freed with [`maze_c_free_maze_solution`].
///
/// # Examples
///
/// Build a solvable 3 × 3 maze (start at top-left, finish at bottom-right)
/// and assert the solve succeeds.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
/// unsafe { maze_c_maze_set_start_cell(ptr, 0, 0) };
/// unsafe { maze_c_maze_set_finish_cell(ptr, 2, 2) };
///
/// let sol = maze_c_maze_solve(ptr);
/// assert!(!sol.is_null());
///
/// maze_c_free_maze_solution(sol);
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_solve(ptr: *mut MazeC) -> *mut MazeSolution {
    clear_last_error();
    let mw = unsafe { &*ptr };
    match mw.maze.solve() {
        Ok(solution) => {
            let boxed = Box::new(solution);
            increment_num_objects_allocated();
            Box::into_raw(boxed)
        }
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Frees a `*mut MazeSolution` returned by [`maze_c_maze_solve`].
///
/// Passing `null` is safe and has no effect.
///
/// # Examples
///
/// Solve a maze, use the solution, then free it. Freeing `null` is also safe.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
/// unsafe { maze_c_maze_set_start_cell(ptr, 0, 0) };
/// unsafe { maze_c_maze_set_finish_cell(ptr, 2, 2) };
///
/// let sol = maze_c_maze_solve(ptr);
/// assert!(!sol.is_null());
/// maze_c_free_maze_solution(sol);
///
/// // Freeing null is a no-op.
/// maze_c_free_maze_solution(std::ptr::null_mut());
///
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_free_maze_solution(ptr: *mut MazeSolution) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
        decrement_num_objects_allocated();
    }
}

/// Returns the solution path points as a flat `u32` array `[row0, col0, row1, col1, …]`.
///
/// Sets `*out_count` to the number of points (each point occupies two consecutive `u32` values).
/// Returns a non-null pointer when `count > 0`, or `null` when the path is empty.
/// The caller must free the returned array with [`maze_c_free_path_points`].
///
/// # Safety
///
/// `solution_ptr` must be a valid non-null pointer to a `MazeSolution` previously returned by
/// [`maze_c_maze_solve`]. `out_count` may be null; if non-null it must be a valid writable
/// location. The returned pointer must be freed with [`maze_c_free_path_points`] using the
/// same `count` value.
///
/// # Examples
///
/// Solve a solvable 3 × 3 maze, get the path points, and assert the first point
/// is the start cell (0, 0) and the last point is the finish cell (2, 2).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
/// unsafe { maze_c_maze_set_start_cell(ptr, 0, 0) };
/// unsafe { maze_c_maze_set_finish_cell(ptr, 2, 2) };
///
/// let sol = maze_c_maze_solve(ptr);
/// assert!(!sol.is_null());
///
/// let mut count: u32 = 0;
/// let pts = unsafe { maze_c_maze_solution_get_path_points(sol, &mut count) };
/// assert!(count > 0);
/// assert!(!pts.is_null());
///
/// // First point is the start cell; last point is the finish cell.
/// let first_row = unsafe { *pts };
/// let first_col = unsafe { *pts.add(1) };
/// assert_eq!(first_row, 0);
/// assert_eq!(first_col, 0);
///
/// unsafe { maze_c_free_path_points(pts, count); }
/// maze_c_free_maze_solution(sol);
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_solution_get_path_points(
    solution_ptr: *mut MazeSolution,
    out_count: *mut u32,
) -> *mut u32 {
    if !out_count.is_null() {
        *out_count = 0;
    }
    if solution_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let solution = &*solution_ptr;
    let points = &solution.path.points;
    let n = points.len();
    if !out_count.is_null() {
        *out_count = n as u32;
    }
    if n == 0 {
        return std::ptr::null_mut();
    }
    let mut data: Vec<u32> = Vec::with_capacity(2 * n);
    for p in points {
        data.push(p.row as u32);
        data.push(p.col as u32);
    }
    let raw = data.as_mut_ptr();
    std::mem::forget(data);
    raw
}

/// Frees a path-points array returned by [`maze_c_maze_solution_get_path_points`].
///
/// `count` must be the value written into `out_count` by that call.
///
/// # Safety
///
/// `ptr` must be a non-null pointer previously returned by
/// [`maze_c_maze_solution_get_path_points`], and `count` must be the exact value
/// written into `out_count` by that call. Calling this function twice on the same
/// pointer, or with a mismatched `count`, is undefined behaviour.
///
/// # Examples
///
/// Get the solution path points from a solved maze, iterate over them, then
/// free the array.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
/// unsafe { maze_c_maze_set_start_cell(ptr, 0, 0) };
/// unsafe { maze_c_maze_set_finish_cell(ptr, 2, 2) };
///
/// let sol = maze_c_maze_solve(ptr);
/// let mut count: u32 = 0;
/// let pts = unsafe { maze_c_maze_solution_get_path_points(sol, &mut count) };
/// assert!(count > 0);
///
/// // Iterate over the flat [row, col, row, col, ...] array.
/// for i in 0..count as usize {
///     let _row = unsafe { *pts.add(2 * i) };
///     let _col = unsafe { *pts.add(2 * i + 1) };
/// }
///
/// unsafe { maze_c_free_path_points(pts, count); }
/// maze_c_free_maze_solution(sol);
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub unsafe extern "C" fn maze_c_free_path_points(ptr: *mut u32, count: u32) {
    if !ptr.is_null() && count > 0 {
        drop(Vec::from_raw_parts(ptr, 2 * count as usize, 2 * count as usize));
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeCGeneratorOptions — lifecycle + setters
// ──────────────────────────────────────────────────────────────────────────────

/// Creates new generator options with the required fields and default optional fields.
///
/// Returns a non-null pointer; the caller must free it with [`maze_c_free_generator_options`].
///
/// # Examples
///
/// Create generator options for a 10 × 10 maze with algorithm 0
/// (RecursiveBacktracking) and seed 42, and assert the pointer is non-null.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// assert!(!opts.is_null());
/// maze_c_free_generator_options(opts);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_new_generator_options(
    row_count: u32,
    col_count: u32,
    algorithm: u32,
    seed: u64,
) -> *mut MazeCGeneratorOptions {
    let opts = Box::new(MazeCGeneratorOptions {
        row_count,
        col_count,
        algorithm,
        seed,
        start_row: u32::MAX,
        start_col: u32::MAX,
        finish_row: u32::MAX,
        finish_col: u32::MAX,
        min_spine_length: 0,
        max_retries: 0,
        branch_from_finish: 0,
    });
    increment_num_objects_allocated();
    Box::into_raw(opts)
}

/// Frees a [`MazeCGeneratorOptions`] pointer.
///
/// Passing `null` is safe and has no effect.
///
/// # Examples
///
/// Create generator options then free them. Freeing `null` is also safe.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_free_generator_options(opts);
///
/// // Freeing null is a no-op.
/// maze_c_free_generator_options(std::ptr::null_mut());
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_free_generator_options(ptr: *mut MazeCGeneratorOptions) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
        decrement_num_objects_allocated();
    }
}

/// Sets the start cell on generator options.
///
/// # Examples
///
/// Create generator options and set the start cell to (0, 0).
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_start(opts, 0, 0);
///
/// let o = unsafe { &*opts };
/// assert_eq!(o.start_row, 0);
/// assert_eq!(o.start_col, 0);
///
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_start(
    ptr: *mut MazeCGeneratorOptions,
    row: u32,
    col: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.start_row = row;
    opts.start_col = col;
}

/// Sets the finish cell on generator options.
///
/// # Examples
///
/// Create generator options and set the finish cell to (9, 9).
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_finish(opts, 9, 9);
///
/// let o = unsafe { &*opts };
/// assert_eq!(o.finish_row, 9);
/// assert_eq!(o.finish_col, 9);
///
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_finish(
    ptr: *mut MazeCGeneratorOptions,
    row: u32,
    col: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.finish_row = row;
    opts.finish_col = col;
}

/// Sets the minimum spine length on generator options.
///
/// # Examples
///
/// Create generator options and set the minimum spine length to 8.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_min_spine_length(opts, 8);
/// assert_eq!(unsafe { (*opts).min_spine_length }, 8);
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_min_spine_length(
    ptr: *mut MazeCGeneratorOptions,
    value: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.min_spine_length = value;
}

/// Sets the maximum retries on generator options.
///
/// # Examples
///
/// Create generator options and set maximum retries to 50.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_max_retries(opts, 50);
/// assert_eq!(unsafe { (*opts).max_retries }, 50);
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_max_retries(
    ptr: *mut MazeCGeneratorOptions,
    value: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.max_retries = value;
}

/// Sets the `branch_from_finish` flag on generator options (`0` = false, `1` = true).
///
/// # Examples
///
/// Create generator options and enable branching from the finish cell.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_branch_from_finish(opts, 1);
/// assert_eq!(unsafe { (*opts).branch_from_finish }, 1);
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_branch_from_finish(
    ptr: *mut MazeCGeneratorOptions,
    value: u8,
) {
    let opts = unsafe { &mut *ptr };
    opts.branch_from_finish = value;
}

// ──────────────────────────────────────────────────────────────────────────────
// Maze generation
// ──────────────────────────────────────────────────────────────────────────────

/// Generates a maze into `*ptr` using the supplied options.
/// Returns `1` on success, `0` on error (check [`maze_c_get_last_error`]).
///
/// # Examples
///
/// Generate a 10 × 10 maze with seed 42, assert the dimensions, and verify
/// the result is solvable.
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
///
/// let ok = maze_c_maze_generate(ptr, opts);
/// assert_eq!(ok, 1);
/// assert_eq!(maze_c_maze_get_row_count(ptr), 10);
/// assert_eq!(maze_c_maze_get_col_count(ptr), 10);
///
/// // Generated mazes are always solvable.
/// let sol = maze_c_maze_solve(ptr);
/// assert!(!sol.is_null());
///
/// maze_c_free_maze_solution(sol);
/// maze_c_free_generator_options(opts);
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_generate(
    ptr: *mut MazeC,
    opts_ptr: *mut MazeCGeneratorOptions,
) -> u8 {
    clear_last_error();
    let mw = unsafe { &mut *ptr };
    let opts = unsafe { &*opts_ptr };

    let start = if opts.start_row == u32::MAX || opts.start_col == u32::MAX {
        None
    } else {
        Some(MazePoint {
            row: opts.start_row as usize,
            col: opts.start_col as usize,
        })
    };
    let finish = if opts.finish_row == u32::MAX || opts.finish_col == u32::MAX {
        None
    } else {
        Some(MazePoint {
            row: opts.finish_row as usize,
            col: opts.finish_col as usize,
        })
    };
    let min_spine_length = if opts.min_spine_length == 0 {
        None
    } else {
        Some(opts.min_spine_length as usize)
    };
    let max_retries = if opts.max_retries == 0 {
        None
    } else {
        Some(opts.max_retries as usize)
    };
    let branch_from_finish = Some(opts.branch_from_finish != 0);

    let algorithm = match opts.algorithm {
        0 => GenerationAlgorithm::RecursiveBacktracking,
        _ => GenerationAlgorithm::RecursiveBacktracking,
    };

    let generator_options = GeneratorOptions {
        row_count: opts.row_count as usize,
        col_count: opts.col_count as usize,
        algorithm,
        start,
        finish,
        min_spine_length,
        max_retries,
        branch_from_finish,
        seed: Some(opts.seed),
    };

    let generator = Generator {
        options: generator_options,
    };
    match generator.generate() {
        Ok(maze) => {
            mw.maze = maze;
            1
        }
        Err(e) => {
            set_last_error(&e.to_string());
            0
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(unused_unsafe)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::ffi::CString;

    // ── helpers ────────────────────────────────────────────────────────────────

    fn last_error_str() -> Option<String> {
        let ptr = maze_c_get_last_error();
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
        }
    }

    fn new_maze() -> *mut MazeC {
        maze_c_new_maze()
    }

    /// Builds a solvable 3×3 maze:  S _ _
    ///                               _ _ _
    ///                               _ _ F
    fn solvable_maze() -> *mut MazeC {
        let ptr = new_maze();
        unsafe {
            maze_c_maze_resize(ptr, 3, 3);
            maze_c_maze_set_start_cell(ptr, 0, 0);
            maze_c_maze_set_finish_cell(ptr, 2, 2);
        }
        ptr
    }

    /// Builds an unsolvable 3×3 maze where the finish is walled off:
    ///  S _ _
    ///  _ W W
    ///  _ W F
    fn unsolvable_maze() -> *mut MazeC {
        let ptr = solvable_maze();
        unsafe {
            maze_c_maze_set_wall_cells(ptr, 1, 1, 2, 2);
            maze_c_maze_set_finish_cell(ptr, 2, 2);
        }
        ptr
    }

    // ── lifecycle ──────────────────────────────────────────────────────────────

    #[test]
    fn can_create_new_maze() {
        let ptr = new_maze();
        assert!(!ptr.is_null());
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn new_maze_is_empty() {
        let ptr = new_maze();
        assert!(maze_c_maze_is_empty(ptr));
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn free_null_maze_is_safe() {
        maze_c_free_maze(std::ptr::null_mut());
    }

    // ── resize / reset ─────────────────────────────────────────────────────────

    #[test]
    fn can_resize_maze() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 4, 5);
        assert_eq!(maze_c_maze_get_row_count(ptr), 4);
        assert_eq!(maze_c_maze_get_col_count(ptr), 5);
        assert!(!maze_c_maze_is_empty(ptr));
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn can_reset_maze() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        assert!(!maze_c_maze_is_empty(ptr));
        maze_c_maze_reset(ptr);
        assert!(maze_c_maze_is_empty(ptr));
        unsafe { maze_c_free_maze(ptr) };
    }

    // ── row / col counts ───────────────────────────────────────────────────────

    #[test]
    fn get_row_count_returns_correct_value() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 7, 3);
        assert_eq!(maze_c_maze_get_row_count(ptr), 7);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn get_col_count_returns_correct_value() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 11);
        assert_eq!(maze_c_maze_get_col_count(ptr), 11);
        unsafe { maze_c_free_maze(ptr) };
    }

    // ── cell type ──────────────────────────────────────────────────────────────

    #[test]
    fn get_cell_type_empty_cell() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let mut ct: u32 = 99;
        let ok = unsafe { maze_c_maze_get_cell_type(ptr, 0, 0, &mut ct) };
        assert_eq!(ok, 1);
        assert_eq!(ct, 0); // Empty
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn get_cell_type_start_cell() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        unsafe { maze_c_maze_set_start_cell(ptr, 0, 0) };
        let mut ct: u32 = 99;
        let ok = unsafe { maze_c_maze_get_cell_type(ptr, 0, 0, &mut ct) };
        assert_eq!(ok, 1);
        assert_eq!(ct, 1); // Start
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn get_cell_type_finish_cell() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        unsafe { maze_c_maze_set_finish_cell(ptr, 2, 2) };
        let mut ct: u32 = 99;
        let ok = unsafe { maze_c_maze_get_cell_type(ptr, 2, 2, &mut ct) };
        assert_eq!(ok, 1);
        assert_eq!(ct, 2); // Finish
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn get_cell_type_wall_cell() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        unsafe { maze_c_maze_set_wall_cells(ptr, 1, 1, 1, 1) };
        let mut ct: u32 = 99;
        let ok = unsafe { maze_c_maze_get_cell_type(ptr, 1, 1, &mut ct) };
        assert_eq!(ok, 1);
        assert_eq!(ct, 3); // Wall
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn get_cell_type_error_row_out_of_bounds() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let mut ct: u32 = 0;
        let ok = unsafe { maze_c_maze_get_cell_type(ptr, 3, 0, &mut ct) };
        assert_eq!(ok, 0);
        assert!(last_error_str().unwrap().contains("row index"));
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn get_cell_type_error_col_out_of_bounds() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let mut ct: u32 = 0;
        let ok = unsafe { maze_c_maze_get_cell_type(ptr, 0, 3, &mut ct) };
        assert_eq!(ok, 0);
        assert!(last_error_str().unwrap().contains("column index"));
        unsafe { maze_c_free_maze(ptr) };
    }

    // ── start / finish cells ───────────────────────────────────────────────────

    #[test]
    fn can_set_and_get_start_cell() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 5, 5);
        let ok = unsafe { maze_c_maze_set_start_cell(ptr, 1, 2) };
        assert_eq!(ok, 1);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let ok2 = unsafe { maze_c_maze_get_start_cell(ptr, &mut row, &mut col) };
        assert_eq!(ok2, 1);
        assert_eq!(row, 1);
        assert_eq!(col, 2);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn get_start_cell_error_no_start() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let mut row: u32 = 0;
        let mut col: u32 = 0;
        let ok = unsafe { maze_c_maze_get_start_cell(ptr, &mut row, &mut col) };
        assert_eq!(ok, 0);
        assert!(last_error_str().unwrap().contains("no start cell"));
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn can_set_and_get_finish_cell() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 5, 5);
        let ok = unsafe { maze_c_maze_set_finish_cell(ptr, 3, 4) };
        assert_eq!(ok, 1);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let ok2 = unsafe { maze_c_maze_get_finish_cell(ptr, &mut row, &mut col) };
        assert_eq!(ok2, 1);
        assert_eq!(row, 3);
        assert_eq!(col, 4);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn get_finish_cell_error_no_finish() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let mut row: u32 = 0;
        let mut col: u32 = 0;
        let ok = unsafe { maze_c_maze_get_finish_cell(ptr, &mut row, &mut col) };
        assert_eq!(ok, 0);
        assert!(last_error_str().unwrap().contains("no finish cell"));
        unsafe { maze_c_free_maze(ptr) };
    }

    // ── wall / clear cells ─────────────────────────────────────────────────────

    #[test]
    fn can_set_wall_cells() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 5, 5);
        let ok = unsafe { maze_c_maze_set_wall_cells(ptr, 0, 0, 4, 4) };
        assert_eq!(ok, 1);
        for r in 0..5_u32 {
            for c in 0..5_u32 {
                let mut ct: u32 = 0;
                unsafe { maze_c_maze_get_cell_type(ptr, r, c, &mut ct) };
                assert_eq!(ct, 3, "expected Wall at ({r},{c})");
            }
        }
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn can_clear_cells() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        unsafe { maze_c_maze_set_wall_cells(ptr, 0, 0, 2, 2) };
        let ok = unsafe { maze_c_maze_clear_cells(ptr, 0, 0, 2, 2) };
        assert_eq!(ok, 1);
        for r in 0..3_u32 {
            for c in 0..3_u32 {
                let mut ct: u32 = 0;
                unsafe { maze_c_maze_get_cell_type(ptr, r, c, &mut ct) };
                assert_eq!(ct, 0, "expected Empty at ({r},{c})");
            }
        }
        unsafe { maze_c_free_maze(ptr) };
    }

    // ── insert / delete rows ───────────────────────────────────────────────────

    #[test]
    fn can_insert_rows() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let ok = maze_c_maze_insert_rows(ptr, 1, 2);
        assert_eq!(ok, 1);
        assert_eq!(maze_c_maze_get_row_count(ptr), 5);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn can_delete_rows() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 5, 3);
        let ok = maze_c_maze_delete_rows(ptr, 1, 2);
        assert_eq!(ok, 1);
        assert_eq!(maze_c_maze_get_row_count(ptr), 3);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn insert_rows_error_out_of_bounds() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let ok = maze_c_maze_insert_rows(ptr, 99, 1);
        assert_eq!(ok, 0);
        assert!(last_error_str().is_some());
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn delete_rows_error_out_of_bounds() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let ok = maze_c_maze_delete_rows(ptr, 99, 1);
        assert_eq!(ok, 0);
        assert!(last_error_str().is_some());
        unsafe { maze_c_free_maze(ptr) };
    }

    // ── insert / delete cols ───────────────────────────────────────────────────

    #[test]
    fn can_insert_cols() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let ok = maze_c_maze_insert_cols(ptr, 1, 3);
        assert_eq!(ok, 1);
        assert_eq!(maze_c_maze_get_col_count(ptr), 6);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn can_delete_cols() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 5);
        let ok = maze_c_maze_delete_cols(ptr, 1, 2);
        assert_eq!(ok, 1);
        assert_eq!(maze_c_maze_get_col_count(ptr), 3);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn insert_cols_error_out_of_bounds() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let ok = maze_c_maze_insert_cols(ptr, 99, 1);
        assert_eq!(ok, 0);
        assert!(last_error_str().is_some());
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn delete_cols_error_out_of_bounds() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let ok = maze_c_maze_delete_cols(ptr, 99, 1);
        assert_eq!(ok, 0);
        assert!(last_error_str().is_some());
        unsafe { maze_c_free_maze(ptr) };
    }

    // ── JSON round-trip ────────────────────────────────────────────────────────

    #[test]
    fn can_convert_maze_to_json() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 2, 2);
        let json_ptr = maze_c_maze_to_json(ptr);
        assert!(!json_ptr.is_null());
        let json = unsafe { CStr::from_ptr(json_ptr) }.to_string_lossy().into_owned();
        assert!(json.contains("grid"));
        unsafe { maze_c_free_string(json_ptr) };
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn can_load_maze_from_json() {
        let ptr = new_maze();
        let json = CString::new(
            r#"{"id":"","name":"","definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#,
        )
        .unwrap();
        let ok = unsafe { maze_c_maze_from_json(ptr, json.as_ptr()) };
        assert_eq!(ok, 1);
        assert_eq!(maze_c_maze_get_row_count(ptr), 2);
        assert_eq!(maze_c_maze_get_col_count(ptr), 3);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn load_maze_from_json_error_invalid_json() {
        let ptr = new_maze();
        let json = CString::new("{invalid}").unwrap();
        let ok = unsafe { maze_c_maze_from_json(ptr, json.as_ptr()) };
        assert_eq!(ok, 0);
        assert!(last_error_str().is_some());
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn json_round_trip_preserves_maze() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        unsafe {
            maze_c_maze_set_start_cell(ptr, 0, 0);
            maze_c_maze_set_finish_cell(ptr, 2, 2);
            maze_c_maze_set_wall_cells(ptr, 1, 1, 1, 1);
        }
        let json_ptr = maze_c_maze_to_json(ptr);
        assert!(!json_ptr.is_null());

        let ptr2 = new_maze();
        let ok = unsafe { maze_c_maze_from_json(ptr2, json_ptr) };
        assert_eq!(ok, 1);
        unsafe { maze_c_free_string(json_ptr) };

        assert_eq!(maze_c_maze_get_row_count(ptr2), 3);
        assert_eq!(maze_c_maze_get_col_count(ptr2), 3);
        let mut ct: u32 = 0;
        unsafe { maze_c_maze_get_cell_type(ptr2, 1, 1, &mut ct) };
        assert_eq!(ct, 3); // Wall

        unsafe {
            maze_c_free_maze(ptr);
            maze_c_free_maze(ptr2);
        }
    }

    // ── solve ──────────────────────────────────────────────────────────────────

    #[test]
    fn can_solve_solvable_maze() {
        let ptr = solvable_maze();
        let sol = maze_c_maze_solve(ptr);
        assert!(!sol.is_null());
        assert!(last_error_str().is_none());
        unsafe {
            maze_c_free_maze_solution(sol);
            maze_c_free_maze(ptr);
        }
    }

    #[test]
    fn solve_error_no_start_cell() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        unsafe { maze_c_maze_set_finish_cell(ptr, 2, 2) };
        let sol = maze_c_maze_solve(ptr);
        assert!(sol.is_null());
        assert!(last_error_str().unwrap().contains("no start cell"));
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn solve_error_no_finish_cell() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        unsafe { maze_c_maze_set_start_cell(ptr, 0, 0) };
        let sol = maze_c_maze_solve(ptr);
        assert!(sol.is_null());
        assert!(last_error_str().unwrap().contains("no finish cell"));
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn solve_error_no_solution() {
        let ptr = unsolvable_maze();
        let sol = maze_c_maze_solve(ptr);
        assert!(sol.is_null());
        assert!(last_error_str().unwrap().contains("no solution"));
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn free_null_solution_is_safe() {
        maze_c_free_maze_solution(std::ptr::null_mut());
    }

    // ── path points ────────────────────────────────────────────────────────────

    #[test]
    fn can_get_solution_path_points() {
        let ptr = solvable_maze();
        let sol = maze_c_maze_solve(ptr);
        assert!(!sol.is_null());

        let mut count: u32 = 0;
        let pts = unsafe { maze_c_maze_solution_get_path_points(sol, &mut count) };
        assert!(count > 0);
        assert!(!pts.is_null());

        // First point should be start (0,0), last should be finish (2,2)
        let first_row = unsafe { *pts };
        let first_col = unsafe { *pts.add(1) };
        assert_eq!(first_row, 0);
        assert_eq!(first_col, 0);
        let last_row = unsafe { *pts.add(2 * (count as usize - 1)) };
        let last_col = unsafe { *pts.add(2 * (count as usize - 1) + 1) };
        assert_eq!(last_row, 2);
        assert_eq!(last_col, 2);

        unsafe {
            maze_c_free_path_points(pts, count);
            maze_c_free_maze_solution(sol);
            maze_c_free_maze(ptr);
        }
    }

    #[test]
    fn path_points_null_solution_returns_null() {
        let mut count: u32 = 99;
        let pts = unsafe { maze_c_maze_solution_get_path_points(std::ptr::null_mut(), &mut count) };
        assert!(pts.is_null());
        assert_eq!(count, 0);
    }

    #[test]
    fn free_null_path_points_is_safe() {
        unsafe { maze_c_free_path_points(std::ptr::null_mut(), 0) };
    }

    // ── object count tracking ──────────────────────────────────────────────────

    #[test]
    fn object_count_increments_for_maze() {
        let before = maze_c_get_num_objects_allocated();
        let ptr = new_maze();
        assert_eq!(maze_c_get_num_objects_allocated(), before + 1);
        unsafe { maze_c_free_maze(ptr) };
        assert_eq!(maze_c_get_num_objects_allocated(), before);
    }

    #[test]
    fn object_count_increments_for_solution() {
        let before = maze_c_get_num_objects_allocated();
        let ptr = solvable_maze();
        let sol = maze_c_maze_solve(ptr);
        assert_eq!(maze_c_get_num_objects_allocated(), before + 2); // maze + solution
        unsafe {
            maze_c_free_maze_solution(sol);
            maze_c_free_maze(ptr);
        }
        assert_eq!(maze_c_get_num_objects_allocated(), before);
    }

    #[test]
    fn object_count_increments_for_generator_options() {
        let before = maze_c_get_num_objects_allocated();
        let opts = maze_c_new_generator_options(5, 5, 0, 42);
        assert_eq!(maze_c_get_num_objects_allocated(), before + 1);
        maze_c_free_generator_options(opts);
        assert_eq!(maze_c_get_num_objects_allocated(), before);
    }

    #[test]
    fn sized_memory_used_is_always_zero() {
        assert_eq!(maze_c_get_sized_memory_used(), 0);
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 10, 10);
        assert_eq!(maze_c_get_sized_memory_used(), 0);
        unsafe { maze_c_free_maze(ptr) };
    }

    // ── last error ─────────────────────────────────────────────────────────────

    #[test]
    fn last_error_is_null_initially() {
        // Call a successful operation to clear any prior error
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        // After a successful operation, last_error should be cleared
        let ok = unsafe { maze_c_maze_set_start_cell(ptr, 0, 0) };
        assert_eq!(ok, 1);
        assert!(last_error_str().is_none());
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn last_error_set_after_out_of_bounds_cell_type() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let mut ct: u32 = 0;
        unsafe { maze_c_maze_get_cell_type(ptr, 99, 0, &mut ct) };
        assert!(last_error_str().is_some());
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn last_error_cleared_after_successful_call() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        // Trigger an error
        let mut ct: u32 = 0;
        unsafe { maze_c_maze_get_cell_type(ptr, 99, 0, &mut ct) };
        assert!(last_error_str().is_some());
        // Successful call clears it
        unsafe { maze_c_maze_set_start_cell(ptr, 0, 0) };
        assert!(last_error_str().is_none());
        unsafe { maze_c_free_maze(ptr) };
    }

    // ── generator options ──────────────────────────────────────────────────────

    #[test]
    fn can_create_and_free_generator_options() {
        let opts = maze_c_new_generator_options(10, 10, 0, 12345);
        assert!(!opts.is_null());
        maze_c_free_generator_options(opts);
    }

    #[test]
    fn free_null_generator_options_is_safe() {
        maze_c_free_generator_options(std::ptr::null_mut());
    }

    #[test]
    fn generator_options_set_start() {
        let opts = maze_c_new_generator_options(10, 10, 0, 0);
        maze_c_generator_options_set_start(opts, 2, 3);
        let o = unsafe { &*opts };
        assert_eq!(o.start_row, 2);
        assert_eq!(o.start_col, 3);
        maze_c_free_generator_options(opts);
    }

    #[test]
    fn generator_options_set_finish() {
        let opts = maze_c_new_generator_options(10, 10, 0, 0);
        maze_c_generator_options_set_finish(opts, 8, 9);
        let o = unsafe { &*opts };
        assert_eq!(o.finish_row, 8);
        assert_eq!(o.finish_col, 9);
        maze_c_free_generator_options(opts);
    }

    #[test]
    fn generator_options_set_min_spine_length() {
        let opts = maze_c_new_generator_options(10, 10, 0, 0);
        maze_c_generator_options_set_min_spine_length(opts, 7);
        assert_eq!(unsafe { (*opts).min_spine_length }, 7);
        maze_c_free_generator_options(opts);
    }

    #[test]
    fn generator_options_set_max_retries() {
        let opts = maze_c_new_generator_options(10, 10, 0, 0);
        maze_c_generator_options_set_max_retries(opts, 50);
        assert_eq!(unsafe { (*opts).max_retries }, 50);
        maze_c_free_generator_options(opts);
    }

    #[test]
    fn generator_options_set_branch_from_finish() {
        let opts = maze_c_new_generator_options(10, 10, 0, 0);
        maze_c_generator_options_set_branch_from_finish(opts, 1);
        assert_eq!(unsafe { (*opts).branch_from_finish }, 1);
        maze_c_free_generator_options(opts);
    }

    // ── generation ─────────────────────────────────────────────────────────────

    #[test]
    fn can_generate_maze() {
        let ptr = new_maze();
        let opts = maze_c_new_generator_options(7, 7, 0, 99);
        let ok = maze_c_maze_generate(ptr, opts);
        assert_eq!(ok, 1, "generate failed: {:?}", last_error_str());
        assert_eq!(maze_c_maze_get_row_count(ptr), 7);
        assert_eq!(maze_c_maze_get_col_count(ptr), 7);
        maze_c_free_generator_options(opts);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn generate_maze_is_solvable() {
        let ptr = new_maze();
        let opts = maze_c_new_generator_options(9, 9, 0, 42);
        let ok = maze_c_maze_generate(ptr, opts);
        assert_eq!(ok, 1);
        let sol = maze_c_maze_solve(ptr);
        assert!(!sol.is_null(), "generated maze is not solvable");
        unsafe {
            maze_c_free_maze_solution(sol);
        }
        maze_c_free_generator_options(opts);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn generate_maze_error_too_small() {
        let ptr = new_maze();
        let opts = maze_c_new_generator_options(1, 1, 0, 0);
        let ok = maze_c_maze_generate(ptr, opts);
        assert_eq!(ok, 0);
        assert!(last_error_str().is_some());
        maze_c_free_generator_options(opts);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn generate_maze_with_start_cell() {
        let ptr = new_maze();
        let opts = maze_c_new_generator_options(9, 9, 0, 7);
        maze_c_generator_options_set_start(opts, 0, 0);
        let ok = maze_c_maze_generate(ptr, opts);
        assert_eq!(ok, 1, "generate failed: {:?}", last_error_str());
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let ok2 = unsafe { maze_c_maze_get_start_cell(ptr, &mut row, &mut col) };
        assert_eq!(ok2, 1);
        assert_eq!(row, 0);
        assert_eq!(col, 0);
        maze_c_free_generator_options(opts);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn generate_maze_with_finish_cell() {
        let ptr = new_maze();
        let opts = maze_c_new_generator_options(9, 9, 0, 8);
        maze_c_generator_options_set_finish(opts, 8, 8);
        let ok = maze_c_maze_generate(ptr, opts);
        assert_eq!(ok, 1, "generate failed: {:?}", last_error_str());
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let ok2 = unsafe { maze_c_maze_get_finish_cell(ptr, &mut row, &mut col) };
        assert_eq!(ok2, 1);
        assert_eq!(row, 8);
        assert_eq!(col, 8);
        maze_c_free_generator_options(opts);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn generate_maze_is_deterministic_with_same_seed() {
        let ptr1 = new_maze();
        let opts1 = maze_c_new_generator_options(11, 11, 0, 12345);
        maze_c_maze_generate(ptr1, opts1);
        let json1_ptr = maze_c_maze_to_json(ptr1);
        let json1 = unsafe { CStr::from_ptr(json1_ptr) }.to_string_lossy().into_owned();
        unsafe { maze_c_free_string(json1_ptr) };

        let ptr2 = new_maze();
        let opts2 = maze_c_new_generator_options(11, 11, 0, 12345);
        maze_c_maze_generate(ptr2, opts2);
        let json2_ptr = maze_c_maze_to_json(ptr2);
        let json2 = unsafe { CStr::from_ptr(json2_ptr) }.to_string_lossy().into_owned();
        unsafe { maze_c_free_string(json2_ptr) };

        assert_eq!(json1, json2, "same seed should produce identical mazes");

        maze_c_free_generator_options(opts1);
        maze_c_free_generator_options(opts2);
        unsafe {
            maze_c_free_maze(ptr1);
            maze_c_free_maze(ptr2);
        }
    }

    #[test]
    fn generate_maze_differs_with_different_seeds() {
        let ptr1 = new_maze();
        let opts1 = maze_c_new_generator_options(11, 11, 0, 11111);
        maze_c_maze_generate(ptr1, opts1);
        let json1_ptr = maze_c_maze_to_json(ptr1);
        let json1 = unsafe { CStr::from_ptr(json1_ptr) }.to_string_lossy().into_owned();
        unsafe { maze_c_free_string(json1_ptr) };

        let ptr2 = new_maze();
        let opts2 = maze_c_new_generator_options(11, 11, 0, 22222);
        maze_c_maze_generate(ptr2, opts2);
        let json2_ptr = maze_c_maze_to_json(ptr2);
        let json2 = unsafe { CStr::from_ptr(json2_ptr) }.to_string_lossy().into_owned();
        unsafe { maze_c_free_string(json2_ptr) };

        assert_ne!(json1, json2, "different seeds should produce different mazes");

        maze_c_free_generator_options(opts1);
        maze_c_free_generator_options(opts2);
        unsafe {
            maze_c_free_maze(ptr1);
            maze_c_free_maze(ptr2);
        }
    }

    #[test]
    fn multiple_independent_mazes() {
        let ptr1 = new_maze();
        let ptr2 = new_maze();
        maze_c_maze_resize(ptr1, 3, 3);
        maze_c_maze_resize(ptr2, 5, 7);
        assert_eq!(maze_c_maze_get_row_count(ptr1), 3);
        assert_eq!(maze_c_maze_get_col_count(ptr1), 3);
        assert_eq!(maze_c_maze_get_row_count(ptr2), 5);
        assert_eq!(maze_c_maze_get_col_count(ptr2), 7);
        unsafe {
            maze_c_free_maze(ptr1);
            maze_c_free_maze(ptr2);
        }
    }
}
