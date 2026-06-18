use data_model::{CellEntity, Maze, MazeDefinition, MazePoint};
use maze::{Generator, GenerationAlgorithm, GeneratorOptions, MazeSolution, MazeSolver};
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

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
/// - `door_count` / `spare_doors` / `spare_keys`: `0` = none (default)
/// - `enemy_count` / `health_count`: `0` = none (default)
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
    pub door_count: u32,
    pub spare_doors: u32,
    pub spare_keys: u32,
    pub enemy_count: u32,
    pub health_count: u32,
    pub treasure_count: u32,
}

/// Opaque game session handle, exposed to C# via P/Invoke.
///
/// Created via [`maze_c_new_maze_game`] and freed via [`maze_c_free_maze_game`].
/// All operations on a `MazeGameC` are performed through the `maze_c_maze_game_*`
/// family of functions.
///
/// # Examples
///
/// Create a game session from a simple 1×3 maze and move the player to the finish.
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert!(!ptr.is_null());
/// assert_eq!(maze_c_maze_game_player_row(ptr), 0);
/// assert_eq!(maze_c_maze_game_player_col(ptr), 0);
/// maze_c_free_maze_game(ptr);
/// ```
pub struct MazeGameC {
    game: maze::MazeGame,
    /// Tick events buffered between consecutive `maze_c_maze_game_tick` calls.
    /// `tick` overwrites this; `get_tick_event` reads from it by index.
    tick_events: Vec<maze::GameEvent>,
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
        'K' => 4,
        'D' => 5,
        'E' => 6,
        'H' => 7,
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

/// Sets a rectangular range of cells to keys (`'K'`). Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Resize a maze to 3 × 3, set cell (1, 1) as a key, and assert its cell
/// type is Key (4).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
///
/// let ok = maze_c_maze_set_key_cells(ptr, 1, 1, 1, 1);
/// assert_eq!(ok, 1);
///
/// let mut ct: u32 = 0;
/// unsafe { maze_c_maze_get_cell_type(ptr, 1, 1, &mut ct) };
/// assert_eq!(ct, 4, "expected Key at (1, 1)");
///
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_maze_set_key_cells(
    ptr: *mut MazeC,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> u8 {
    clear_last_error();
    set_cell_range(ptr, start_row, start_col, end_row, end_col, 'K')
}

/// Sets a rectangular range of cells to doors (`'D'`). Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Resize a maze to 3 × 3, set cell (1, 2) as a door, and assert its cell
/// type is Door (5).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
///
/// let ok = maze_c_maze_set_door_cells(ptr, 1, 2, 1, 2);
/// assert_eq!(ok, 1);
///
/// let mut ct: u32 = 0;
/// unsafe { maze_c_maze_get_cell_type(ptr, 1, 2, &mut ct) };
/// assert_eq!(ct, 5, "expected Door at (1, 2)");
///
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_maze_set_door_cells(
    ptr: *mut MazeC,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> u8 {
    clear_last_error();
    set_cell_range(ptr, start_row, start_col, end_row, end_col, 'D')
}

/// Sets a rectangular range of cells to enemy spawns (`'E'`). Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Resize a maze to 3 × 3, set cell (1, 1) as an enemy spawn, and assert its
/// cell type is Enemy (6).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
///
/// let ok = maze_c_maze_set_enemy_cells(ptr, 1, 1, 1, 1);
/// assert_eq!(ok, 1);
///
/// let mut ct: u32 = 0;
/// unsafe { maze_c_maze_get_cell_type(ptr, 1, 1, &mut ct) };
/// assert_eq!(ct, 6, "expected Enemy at (1, 1)");
///
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_maze_set_enemy_cells(
    ptr: *mut MazeC,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> u8 {
    clear_last_error();
    set_cell_range(ptr, start_row, start_col, end_row, end_col, 'E')
}

/// Sets a rectangular range of cells to health pickups (`'H'`). Returns `1` on success, `0` on error.
///
/// # Examples
///
/// Resize a maze to 3 × 3, set cell (1, 2) as a health pickup, and assert its
/// cell type is Health (7).
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 3, 3);
///
/// let ok = maze_c_maze_set_health_cells(ptr, 1, 2, 1, 2);
/// assert_eq!(ok, 1);
///
/// let mut ct: u32 = 0;
/// unsafe { maze_c_maze_get_cell_type(ptr, 1, 2, &mut ct) };
/// assert_eq!(ct, 7, "expected Health at (1, 2)");
///
/// maze_c_free_maze(ptr);
/// ```
#[no_mangle]
pub extern "C" fn maze_c_maze_set_health_cells(
    ptr: *mut MazeC,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> u8 {
    clear_last_error();
    set_cell_range(ptr, start_row, start_col, end_row, end_col, 'H')
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

/// Returns the per-cell entity override at `(row, col)` as its wire JSON
/// (e.g. `{"type":"E","enemyType":"ghost","damage":2}`), or `null` when the
/// cell carries no override (or is out of range). The caller must free a
/// non-null result with [`maze_c_free_string`].
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 1, 3);
/// assert!(maze_c_maze_get_cell_entity(ptr, 0, 1).is_null()); // no override yet
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_get_cell_entity(ptr: *mut MazeC, row: u32, col: u32) -> *mut c_char {
    clear_last_error();
    let mw = unsafe { &*ptr };
    let entity = mw
        .maze
        .definition
        .cell_entities
        .get(&(row as usize, col as usize))
        .and_then(|entities| entities.first());
    match entity {
        Some(entity) => match serde_json::to_string(entity) {
            Ok(s) => match CString::new(s) {
                Ok(cs) => cs.into_raw(),
                Err(e) => {
                    set_last_error(&e.to_string());
                    ptr::null_mut()
                }
            },
            Err(e) => {
                set_last_error(&e.to_string());
                ptr::null_mut()
            }
        },
        None => ptr::null_mut(),
    }
}

/// Sets the per-cell entity override at `(row, col)` from its wire JSON,
/// replacing any existing one. The entity `type` must match the cell's current
/// character (set the cell to the matching kind first). Returns `1` on success,
/// `0` on a null/invalid JSON pointer, a parse error, an out-of-range cell, or
/// a `type`/cell-character mismatch (see [`maze_c_get_last_error`]).
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze`]; `json`
/// must be a valid null-terminated UTF-8 string or null.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 1, 3);
/// maze_c_maze_set_enemy_cells(ptr, 0, 1, 0, 1); // cell (0,1) becomes 'E'
/// let entity = CString::new(r#"{"type":"E","enemyType":"ghost"}"#).unwrap();
/// let rc = unsafe { maze_c_maze_set_cell_entity(ptr, 0, 1, entity.as_ptr()) };
/// assert_eq!(rc, 1);
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_set_cell_entity(
    ptr: *mut MazeC,
    row: u32,
    col: u32,
    json: *const c_char,
) -> u8 {
    clear_last_error();
    if json.is_null() {
        set_last_error("json pointer is null");
        return 0;
    }
    let json_str = match unsafe { CStr::from_ptr(json) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&e.to_string());
            return 0;
        }
    };
    let entity: CellEntity = match serde_json::from_str(json_str) {
        Ok(entity) => entity,
        Err(e) => {
            set_last_error(&e.to_string());
            return 0;
        }
    };
    let mw = unsafe { &mut *ptr };
    let (r, c) = (row as usize, col as usize);
    if r >= mw.maze.definition.row_count() || c >= mw.maze.definition.col_count() {
        set_last_error("cell out of range");
        return 0;
    }
    let cell_char = mw.maze.definition.grid[r][c];
    if entity.cell_char() != cell_char {
        set_last_error(&format!(
            "cell entity type '{}' does not match cell character '{}'",
            entity.cell_char(),
            cell_char
        ));
        return 0;
    }
    mw.maze.definition.cell_entities.insert((r, c), vec![entity]);
    1
}

/// Clears any per-cell entity override at `(row, col)`. Returns `1` (a cell
/// with no override is unaffected).
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let ptr = maze_c_new_maze();
/// maze_c_maze_resize(ptr, 1, 3);
/// maze_c_maze_set_enemy_cells(ptr, 0, 1, 0, 1);
/// let entity = CString::new(r#"{"type":"E","damage":2}"#).unwrap();
/// unsafe { maze_c_maze_set_cell_entity(ptr, 0, 1, entity.as_ptr()) };
/// assert_eq!(maze_c_maze_clear_cell_entity(ptr, 0, 1), 1);
/// assert!(maze_c_maze_get_cell_entity(ptr, 0, 1).is_null()); // cleared
/// maze_c_free_maze(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_clear_cell_entity(ptr: *mut MazeC, row: u32, col: u32) -> u8 {
    let mw = unsafe { &mut *ptr };
    mw.maze
        .definition
        .cell_entities
        .remove(&(row as usize, col as usize));
    1
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
        door_count: 0,
        spare_doors: 0,
        spare_keys: 0,
        enemy_count: 0,
        health_count: 0,
        treasure_count: 0,
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

/// Sets the number of real path doors auto-placed on the spine.
///
/// `0` (the default) places none; the value is clamped further by the Rust
/// generator (see `maze::MAX_AUTO_DOORS`) and rejected outright if
/// `2 * door_count + spare_doors + spare_keys` exceeds the key-aware
/// solver's `maze::MAX_TOTAL_FEATURES` cap (see [`maze_c_maze_generate`]).
///
/// # Examples
///
/// Create generator options and set the door count to 3.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_door_count(opts, 3);
/// assert_eq!(unsafe { (*opts).door_count }, 3);
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_door_count(
    ptr: *mut MazeCGeneratorOptions,
    value: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.door_count = value;
}

/// Sets the number of decoy doors planted on off-spine branches.
///
/// `0` (the default) places none. See [`maze_c_generator_options_set_door_count`]
/// for the joint cap that applies across all three K/D fields.
///
/// # Examples
///
/// Create generator options and set the spare-door count to 2.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_spare_doors(opts, 2);
/// assert_eq!(unsafe { (*opts).spare_doors }, 2);
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_spare_doors(
    ptr: *mut MazeCGeneratorOptions,
    value: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.spare_doors = value;
}

/// Sets the number of spare keys planted on off-spine branches.
///
/// `0` (the default) places none. See [`maze_c_generator_options_set_door_count`]
/// for the joint cap that applies across all three K/D fields.
///
/// # Examples
///
/// Create generator options and set the spare-key count to 2.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_spare_keys(opts, 2);
/// assert_eq!(unsafe { (*opts).spare_keys }, 2);
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_spare_keys(
    ptr: *mut MazeCGeneratorOptions,
    value: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.spare_keys = value;
}

/// Sets the number of enemies to auto-place at random passable cells.
///
/// `0` (the default) places none. The generator clamps the request to
/// `maze::MAX_ENEMY_COUNT` and to the number of eligible cells. Enemies
/// never spawn on the start, finish, the cells immediately around the
/// start, or any key / door / enemy / health cell.
///
/// # Examples
///
/// Create generator options and set the enemy count to 3.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_enemy_count(opts, 3);
/// assert_eq!(unsafe { (*opts).enemy_count }, 3);
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_enemy_count(
    ptr: *mut MazeCGeneratorOptions,
    value: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.enemy_count = value;
}

/// Sets the number of health pickups to auto-place at random passable cells.
///
/// `0` (the default) places none. The generator clamps the request to
/// `maze::MAX_HEALTH_COUNT` and to the number of eligible cells, using the
/// same eligibility rules as enemy placement.
///
/// # Examples
///
/// Create generator options and set the health-pickup count to 2.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_health_count(opts, 2);
/// assert_eq!(unsafe { (*opts).health_count }, 2);
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_health_count(
    ptr: *mut MazeCGeneratorOptions,
    value: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.health_count = value;
}

/// Sets the number of treasure cells to auto-place.
///
/// `0` (the default) places none. The generator places treasure dead-end-first
/// (corridor ends before other walkable cells), rarity-weighted, clamping the
/// request to `maze::MAX_TREASURE_COUNT` and to the eligible-cell count.
///
/// # Examples
///
/// Create generator options and set the treasure count to 3.
///
/// ```rust
/// use maze_c::*;
///
/// let opts = maze_c_new_generator_options(10, 10, 0, 42);
/// maze_c_generator_options_set_treasure_count(opts, 3);
/// assert_eq!(unsafe { (*opts).treasure_count }, 3);
/// maze_c_free_generator_options(opts);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_generator_options_set_treasure_count(
    ptr: *mut MazeCGeneratorOptions,
    value: u32,
) {
    let opts = unsafe { &mut *ptr };
    opts.treasure_count = value;
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
        door_count: Some(opts.door_count as usize),
        spare_doors: Some(opts.spare_doors as usize),
        spare_keys: Some(opts.spare_keys as usize),
        enemy_count: Some(opts.enemy_count as usize),
        health_count: Some(opts.health_count as usize),
        treasure_count: Some(opts.treasure_count as usize),
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

// ──────────────────────────────────────────────────────────────────────────────
// MazeGameC — direction / move-result helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Converts an `i32` to a [`maze::Direction`].
///
/// Encoding: `0`=None, `1`=Up, `2`=Down, `3`=Left, `4`=Right.
/// Returns `None` for any other value.
fn game_direction_from_i32(dir: i32) -> Option<maze::Direction> {
    match dir {
        0 => Some(maze::Direction::None),
        1 => Some(maze::Direction::Up),
        2 => Some(maze::Direction::Down),
        3 => Some(maze::Direction::Left),
        4 => Some(maze::Direction::Right),
        _ => None,
    }
}

/// Converts a [`maze::Direction`] to its `i32` encoding.
///
/// Encoding: `0`=None, `1`=Up, `2`=Down, `3`=Left, `4`=Right.
fn game_direction_to_i32(dir: maze::Direction) -> i32 {
    match dir {
        maze::Direction::None => 0,
        maze::Direction::Up => 1,
        maze::Direction::Down => 2,
        maze::Direction::Left => 3,
        maze::Direction::Right => 4,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeGameC — lifecycle
// ──────────────────────────────────────────────────────────────────────────────

/// Creates a maze game session from a JSON maze definition.
///
/// `json` must be a non-null, null-terminated UTF-8 string containing a
/// `MazeDefinition` JSON object.
///
/// Returns a non-null pointer on success. The caller must free it with
/// [`maze_c_free_maze_game`] when done.
///
/// On error (invalid JSON or no start cell), returns `null` and stores the
/// error message for retrieval via [`maze_c_get_last_error`].
///
/// # Safety
///
/// `json` must be a non-null pointer to a valid null-terminated UTF-8 string.
/// The pointer must remain valid for the duration of this call.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert!(!ptr.is_null());
/// maze_c_free_maze_game(ptr);
/// ```
#[no_mangle]
pub unsafe extern "C" fn maze_c_new_maze_game(json: *const c_char) -> *mut MazeGameC {
    clear_last_error();
    if json.is_null() {
        set_last_error("json pointer is null");
        return ptr::null_mut();
    }
    let json_str = match CStr::from_ptr(json).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&e.to_string());
            return ptr::null_mut();
        }
    };    
    match maze::MazeGame::from_json(json_str) {
        Ok(game) => {
            let boxed = Box::new(MazeGameC { game, tick_events: Vec::new() });
            increment_num_objects_allocated();
            Box::into_raw(boxed)
        }
        Err(e) => {
            set_last_error(&e);
            ptr::null_mut()
        }
    }
}

/// Frees a [`MazeGameC`] pointer returned by [`maze_c_new_maze_game`].
///
/// Passing `null` is safe and has no effect.
///
/// # Safety
///
/// `ptr` must be either null or a non-null pointer previously returned by
/// [`maze_c_new_maze_game`]. Calling this function twice on the same pointer
/// is undefined behaviour.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// maze_c_free_maze_game(ptr);
///
/// // Freeing null is a no-op.
/// maze_c_free_maze_game(std::ptr::null_mut());
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_free_maze_game(ptr: *mut MazeGameC) {
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
        decrement_num_objects_allocated();
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeGameC — movement
// ──────────────────────────────────────────────────────────────────────────────

/// Moves the player one cell in the given direction.
///
/// `dir` encoding: `0`=None, `1`=Up, `2`=Down, `3`=Left, `4`=Right.
///
/// Returns the move result: `0`=None, `1`=Moved, `2`=Blocked, `3`=Complete,
/// `4`=BlockedByLockedDoor, `5`=StartedUnlocking, `6`=Stranded, or `-1` for an
/// unknown `dir` value.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_move_player(ptr, 4), 1); // Right → Moved
/// assert_eq!(maze_c_maze_game_move_player(ptr, 4), 3); // Right → Complete
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_move_player(ptr: *mut MazeGameC, dir: i32) -> i32 {
    let game = unsafe { &mut (*ptr).game };
    let direction = match game_direction_from_i32(dir) {
        Some(d) => d,
        None => return -1,
    };
    match game.move_player(direction) {
        maze::MoveResult::None => 0,
        maze::MoveResult::Moved => 1,
        maze::MoveResult::Blocked => 2,
        maze::MoveResult::Complete => 3,
        maze::MoveResult::BlockedByLockedDoor => 4,
        maze::MoveResult::StartedUnlocking => 5,
        maze::MoveResult::Stranded => 6,
        maze::MoveResult::Killed => 7,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeGameC — state getters
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the player's current row (0-based).
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_player_row(ptr), 0);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_player_row(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    game.player_row() as i32
}

/// Returns the player's current column (0-based).
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_player_col(ptr), 0);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_player_col(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    game.player_col() as i32
}

/// Returns the player's current facing direction.
///
/// Return encoding: `0`=None, `1`=Up, `2`=Down, `3`=Left, `4`=Right.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// maze_c_maze_game_move_player(ptr, 4); // move Right
/// assert_eq!(maze_c_maze_game_player_direction(ptr), 4); // Right
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_player_direction(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    game_direction_to_i32(game.player_direction())
}

/// Returns `1` if the player has reached the finish cell, `0` otherwise.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_is_complete(ptr), 0);
/// maze_c_maze_game_move_player(ptr, 4); // Right
/// maze_c_maze_game_move_player(ptr, 4); // Right → finish
/// assert_eq!(maze_c_maze_game_is_complete(ptr), 1);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_is_complete(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    if game.is_complete() { 1 } else { 0 }
}

/// Returns `1` if the game is in a lost state, `0` otherwise.
///
/// The companion [`maze_c_maze_game_lose_reason`] returns the reason code when
/// this returns `1`.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_is_lost(ptr), 0);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_is_lost(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    if game.is_lost() { 1 } else { 0 }
}

/// Returns the lose-reason code for the game session.
///
/// Encoding: `0` = None (the game is not lost), `1` = Stranded (the player
/// can no longer hold enough keys to open every closed door remaining on a
/// route to the finish). Mirrors the [`maze::LoseReason`] enum; new variants
/// extend the integer space.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_lose_reason(ptr), 0); // None
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_lose_reason(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    match game.lose_reason() {
        None => 0,
        Some(maze::LoseReason::Stranded) => 1,
        Some(maze::LoseReason::Killed) => 2,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeGameC — bag / pickup
// ──────────────────────────────────────────────────────────────────────────────

/// Attempts to pick up a collectible at the player's current cell.
///
/// On success returns `1` and writes the kind / id of the picked item into
/// `*out_kind` / `*out_id`. On failure (no collectible at the player's cell)
/// returns `0` and the out-parameters are not written.
///
/// `kind` encoding: `0` = Key (the only variant in the current bag model).
/// New variants extend the kind space.
///
/// Keys are auto-collected when the player walks onto a `'K'` cell, so an
/// explicit call normally finds nothing left to collect and returns `0` — the
/// cell was cleared as the player stepped onto it.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// `out_kind` and `out_id` may be null; non-null pointers must be valid
/// writable locations.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// // Player at (0,0); key at (0,1); finish at (0,2).
/// let json = CString::new(r#"{"grid":[["S","K","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
///
/// // Standing on S → no pickup.
/// let mut k: u32 = 99;
/// let mut id: u32 = 99;
/// let ok = unsafe { maze_c_maze_game_pickup(ptr, &mut k, &mut id) };
/// assert_eq!(ok, 0);
///
/// // Step onto the K cell — the key is auto-collected on walk-over, so an
/// // explicit pickup at the now-cleared cell returns 0.
/// maze_c_maze_game_move_player(ptr, 4); // Right
/// let ok = unsafe { maze_c_maze_game_pickup(ptr, &mut k, &mut id) };
/// assert_eq!(ok, 0);
///
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_pickup(
    ptr: *mut MazeGameC,
    out_kind: *mut u32,
    out_id: *mut u32,
) -> u8 {
    let game = unsafe { &mut (*ptr).game };
    match game.pickup() {
        Some(maze::BagItem::Key { id }) => {
            unsafe {
                if !out_kind.is_null() {
                    *out_kind = 0; // Key
                }
                if !out_id.is_null() {
                    *out_id = id;
                }
            }
            1
        }
        None => 0,
    }
}

/// Returns the number of items currently in the player's bag.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","K","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_bag_count(ptr), 0);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_bag_count(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    game.bag().len() as i32
}

/// Retrieves a single bag item by index.
///
/// Writes the item's kind code and id into `*out_kind` / `*out_id`. Returns
/// `1` on success, `0` if `index` is out of range.
///
/// `kind` encoding: `0` = Key.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// `out_kind` and `out_id` may be null; non-null pointers must be valid
/// writable locations.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","K","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// maze_c_maze_game_move_player(ptr, 4); // Right onto K — auto-collected
///
/// let mut k2: u32 = 99;
/// let mut id2: u32 = 99;
/// let ok = unsafe { maze_c_maze_game_get_bag_item(ptr, 0, &mut k2, &mut id2) };
/// assert_eq!(ok, 1);
/// assert_eq!(k2, 0); // Key
/// assert_eq!(id2, 0); // first key id
///
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_bag_item(
    ptr: *mut MazeGameC,
    index: i32,
    out_kind: *mut u32,
    out_id: *mut u32,
) -> u8 {
    let game = unsafe { &(*ptr).game };
    let bag = game.bag();
    if index < 0 || index as usize >= bag.len() {
        return 0;
    }
    let maze::BagItem::Key { id } = bag[index as usize];
    unsafe {
        if !out_kind.is_null() {
            *out_kind = 0; // Key
        }
        if !out_id.is_null() {
            *out_id = id;
        }
    }
    1
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeGameC — doors / tick / events
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the number of door cells (`'D'`) in the maze, regardless of state.
///
/// The count is fixed for the lifetime of the game session (opening a door
/// changes its [`maze::DoorState`] but does not remove it from the list).
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","K","D","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_door_count(ptr), 1);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_door_count(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    game.doors().len() as i32
}

/// Retrieves a single door cell by index.
///
/// Writes the door's row, column, and current state code into the out
/// parameters. State encoding mirrors [`maze::DoorState`]:
/// `0` = Locked, `1` = Opening, `2` = Open.
///
/// Returns `1` on success, `0` if `index` is out of range.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// Out parameters may be null; non-null pointers must be valid writable
/// locations.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","K","D","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// let mut row: u32 = 99;
/// let mut col: u32 = 99;
/// let mut state: u32 = 99;
/// let ok = unsafe { maze_c_maze_game_get_door(ptr, 0, &mut row, &mut col, &mut state) };
/// assert_eq!(ok, 1);
/// assert_eq!(row, 0);
/// assert_eq!(col, 2);
/// assert_eq!(state, 0); // Locked
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_door(
    ptr: *mut MazeGameC,
    index: i32,
    out_row: *mut u32,
    out_col: *mut u32,
    out_state: *mut u32,
) -> u8 {
    let game = unsafe { &(*ptr).game };
    let doors = game.doors();
    if index < 0 || index as usize >= doors.len() {
        return 0;
    }
    let ((r, c), state) = doors[index as usize];
    let state_code: u32 = match state {
        maze::DoorState::Locked => 0,
        maze::DoorState::Opening { .. } => 1,
        maze::DoorState::Open => 2,
    };
    unsafe {
        if !out_row.is_null() {
            *out_row = r as u32;
        }
        if !out_col.is_null() {
            *out_col = c as u32;
        }
        if !out_state.is_null() {
            *out_state = state_code;
        }
    }
    1
}

/// Advances time-based game state by `dt_ms` milliseconds and buffers the
/// resulting events on the game session. Returns the number of events
/// produced. Subsequent calls to [`maze_c_maze_game_tick_event_count`] /
/// [`maze_c_maze_game_get_tick_event`] read from the same buffer until the
/// next call to `tick` overwrites it.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","K","D","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// maze_c_maze_game_move_player(ptr, 4); // Right → key, auto-collected
/// maze_c_maze_game_tick(ptr, 0.0);      // flush the KeyCollected event
/// maze_c_maze_game_move_player(ptr, 4); // Right into door → StartedUnlocking
/// let count = maze_c_maze_game_tick(ptr, 1000.0);
/// assert_eq!(count, 1); // one DoorOpened event
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_tick(ptr: *mut MazeGameC, dt_ms: f32) -> i32 {
    let g = unsafe { &mut *ptr };
    g.tick_events = g.game.tick(dt_ms);
    g.tick_events.len() as i32
}

/// Returns the number of events currently buffered from the most recent
/// [`maze_c_maze_game_tick`] call. Zero before the first tick.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_tick_event_count(ptr), 0);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_tick_event_count(ptr: *mut MazeGameC) -> i32 {
    let g = unsafe { &(*ptr) };
    g.tick_events.len() as i32
}

/// Retrieves a single tick event's kind + cell coordinates from the buffer by index.
///
/// Writes the event's kind code and a `(row, col)` pair into the out parameters.
/// Kind encoding (mirrors [`maze::GameEvent`]):
/// - `0` = `DoorOpened` — `(row, col)` is the door cell.
/// - `1` = `EnemyMoved` — `(row, col)` is the enemy's new cell; the enemy id is
///   carried by [`maze_c_maze_game_get_tick_event_payload`].
/// - `2` = `PlayerDamaged` — `(row, col)` is `(0, 0)`; `hp_after` is carried by
///   [`maze_c_maze_game_get_tick_event_payload`].
/// - `3` = `PlayerHealed` — `(row, col)` is the consumed pickup cell; `hp_after`
///   is carried by [`maze_c_maze_game_get_tick_event_payload`].
/// - `4` = `PlayerNotHealed` — `(row, col)` is the spared pickup cell; the reason
///   code is carried by [`maze_c_maze_game_get_tick_event_payload`] and the
///   default message by [`maze_c_maze_game_get_tick_event_string_payload`].
/// - `5` = `KeyCollected` — `(row, col)` is the consumed key cell; the key id is
///   carried by [`maze_c_maze_game_get_tick_event_payload`].
///
/// Returns `1` on success, `0` if `index` is out of range.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// Out parameters may be null; non-null pointers must be valid writable
/// locations.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","K","D","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// maze_c_maze_game_move_player(ptr, 4); // onto key — auto-collected
/// maze_c_maze_game_tick(ptr, 0.0);      // flush the KeyCollected event
/// maze_c_maze_game_move_player(ptr, 4); // into door → StartedUnlocking
/// maze_c_maze_game_tick(ptr, 1000.0);
/// let mut kind: u32 = 99;
/// let mut row: u32 = 99;
/// let mut col: u32 = 99;
/// let ok = unsafe { maze_c_maze_game_get_tick_event(ptr, 0, &mut kind, &mut row, &mut col) };
/// assert_eq!(ok, 1);
/// assert_eq!(kind, 0); // DoorOpened
/// assert_eq!((row, col), (0, 2));
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_tick_event(
    ptr: *mut MazeGameC,
    index: i32,
    out_kind: *mut u32,
    out_row: *mut u32,
    out_col: *mut u32,
) -> u8 {
    let g = unsafe { &(*ptr) };
    if index < 0 || index as usize >= g.tick_events.len() {
        return 0;
    }
    let (kind, r, c) = match &g.tick_events[index as usize] {
        maze::GameEvent::DoorOpened { cell: (r, c) } => (0u32, *r, *c),
        maze::GameEvent::EnemyMoved { row, col, .. } => (1u32, *row, *col),
        maze::GameEvent::PlayerDamaged { .. } => (2u32, 0, 0),
        maze::GameEvent::PlayerHealed { cell: (r, c), .. } => (3u32, *r, *c),
        maze::GameEvent::PlayerNotHealed { cell: (r, c), .. } => (4u32, *r, *c),
        maze::GameEvent::KeyCollected { cell: (r, c), .. } => (5u32, *r, *c),
        maze::GameEvent::TreasureCollected { cell: (r, c), .. } => (6u32, *r, *c),
    };
    unsafe {
        if !out_kind.is_null() {
            *out_kind = kind;
        }
        if !out_row.is_null() {
            *out_row = r as u32;
        }
        if !out_col.is_null() {
            *out_col = c as u32;
        }
    }
    1
}

/// Retrieves a tick event's `u32` payload by index — the extra scalar that
/// doesn't fit the `(kind, row, col)` shape of
/// [`maze_c_maze_game_get_tick_event`]:
/// - `EnemyMoved` → the enemy id.
/// - `PlayerDamaged` / `PlayerHealed` → `hp_after`.
/// - `PlayerNotHealed` → the reason code (`0` = already at max HP).
/// - `KeyCollected` → the collected key id.
/// - `DoorOpened` → `0` (no extra payload).
///
/// Returns `1` on success, `0` if `index` is out of range.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// `out_payload` may be null; a non-null pointer must be a valid writable
/// location.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","E","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// maze_c_maze_game_tick(ptr, 1500.0); // enemy steps onto the player → EnemyMoved + PlayerDamaged
/// let mut kind: u32 = 99;
/// unsafe { maze_c_maze_game_get_tick_event(ptr, 0, &mut kind, std::ptr::null_mut(), std::ptr::null_mut()) };
/// assert_eq!(kind, 1); // EnemyMoved
/// let mut id: u32 = 99;
/// let ok = unsafe { maze_c_maze_game_get_tick_event_payload(ptr, 0, &mut id) };
/// assert_eq!(ok, 1);
/// assert_eq!(id, 0); // enemy id
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_tick_event_payload(
    ptr: *mut MazeGameC,
    index: i32,
    out_payload: *mut u32,
) -> u8 {
    let g = unsafe { &(*ptr) };
    if index < 0 || index as usize >= g.tick_events.len() {
        return 0;
    }
    let payload = match &g.tick_events[index as usize] {
        maze::GameEvent::DoorOpened { .. } => 0,
        maze::GameEvent::EnemyMoved { id, .. } => *id,
        maze::GameEvent::PlayerDamaged { hp_after } => *hp_after,
        maze::GameEvent::PlayerHealed { hp_after, .. } => *hp_after,
        maze::GameEvent::PlayerNotHealed { reason, .. } => match reason {
            maze::PlayerNotHealedReason::AlreadyAtMaxHp => 0,
        },
        maze::GameEvent::KeyCollected { id, .. } => *id,
        maze::GameEvent::TreasureCollected { value, .. } => *value,
    };
    unsafe {
        if !out_payload.is_null() {
            *out_payload = payload;
        }
    }
    1
}

/// Retrieves the UTF-8 string payload of a tick event by index — currently only
/// `PlayerNotHealed` carries one (its default human-readable message); every
/// other variant reports a zero-length string.
///
/// Two-call protocol: call once with `out_buf` null to read the byte length into
/// `out_len`, allocate a buffer of that size, then call again with `out_buf`
/// pointing at it to copy the bytes. The buffer is stable between calls until the
/// next [`maze_c_maze_game_tick`]. When `out_buf` is non-null the caller must have
/// allocated at least `*out_len` bytes.
///
/// Returns `1` on success, `0` if `index` is out of range.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// `out_len` may be null. When `out_buf` is non-null it must point to a writable
/// region of at least the byte length previously reported via `out_len`.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// // Player at full HP walks onto 'H' → the pickup is spared → PlayerNotHealed.
/// let json = CString::new(r#"{"grid":[["S","H","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// maze_c_maze_game_move_player(ptr, 4); // Right onto 'H'
/// maze_c_maze_game_tick(ptr, 0.0);      // flush the queued PlayerNotHealed
/// let mut len: u32 = 0;
/// let ok = unsafe { maze_c_maze_game_get_tick_event_string_payload(ptr, 0, std::ptr::null_mut(), &mut len) };
/// assert_eq!(ok, 1);
/// let mut buf = vec![0u8; len as usize];
/// unsafe { maze_c_maze_game_get_tick_event_string_payload(ptr, 0, buf.as_mut_ptr(), &mut len) };
/// assert_eq!(String::from_utf8(buf).unwrap(), "Already at maximum health");
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_tick_event_string_payload(
    ptr: *mut MazeGameC,
    index: i32,
    out_buf: *mut u8,
    out_len: *mut u32,
) -> u8 {
    let g = unsafe { &(*ptr) };
    if index < 0 || index as usize >= g.tick_events.len() {
        return 0;
    }
    let message: &str = match &g.tick_events[index as usize] {
        maze::GameEvent::PlayerNotHealed { message, .. } => message,
        _ => "",
    };
    let bytes = message.as_bytes();
    unsafe {
        if !out_len.is_null() {
            *out_len = bytes.len() as u32;
        }
        if !out_buf.is_null() {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf, bytes.len());
        }
    }
    1
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeGameC — keys (uncollected, sorted by (row, col))
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the number of uncollected key cells in the maze.
///
/// The count shrinks as the player picks keys up (collected keys disappear
/// from this list — they move into the bag).
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","K","K","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_key_count(ptr), 2);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_key_count(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    game.keys().len() as i32
}

/// Retrieves a single uncollected key cell by index.
///
/// Writes the key's row, column, and stable id into the out parameters.
///
/// Returns `1` on success, `0` if `index` is out of range.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// Out parameters may be null; non-null pointers must be valid writable
/// locations.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","K","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// let mut row: u32 = 99;
/// let mut col: u32 = 99;
/// let mut id: u32 = 99;
/// let ok = unsafe { maze_c_maze_game_get_key(ptr, 0, &mut row, &mut col, &mut id) };
/// assert_eq!(ok, 1);
/// assert_eq!(row, 0);
/// assert_eq!(col, 1);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_key(
    ptr: *mut MazeGameC,
    index: i32,
    out_row: *mut u32,
    out_col: *mut u32,
    out_id: *mut u32,
) -> u8 {
    let game = unsafe { &(*ptr).game };
    let keys = game.keys();
    if index < 0 || index as usize >= keys.len() {
        return 0;
    }
    let ((r, c), id) = keys[index as usize];
    unsafe {
        if !out_row.is_null() {
            *out_row = r as u32;
        }
        if !out_col.is_null() {
            *out_col = c as u32;
        }
        if !out_id.is_null() {
            *out_id = id;
        }
    }
    1
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeGameC — HP, enemies, health pickups
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the player's current HP.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_hp(ptr), 3); // default starting HP
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_hp(ptr: *mut MazeGameC) -> u32 {
    let game = unsafe { &(*ptr).game };
    game.hp()
}

/// Returns the player's maximum HP.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_max_hp(ptr), 3); // default max HP
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_max_hp(ptr: *mut MazeGameC) -> u32 {
    let game = unsafe { &(*ptr).game };
    game.max_hp()
}

/// Returns the number of active enemies.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","E","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_enemy_count(ptr), 1);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_enemy_count(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    game.enemies().len() as i32
}

/// Retrieves a single enemy's current cell, stable id, and resolved per-enemy
/// tunables by index. `out_damage` / `out_move_period_ms` carry the resolved
/// values (per-cell override else the per-game default). `out_enemy_type`
/// carries the visual-rig override: `-1` when the spawn cell set none (the
/// renderer falls back to its default), else the [`maze::EnemyType`] ordinal
/// (`0` = goblin, `1` = ghost).
///
/// Returns `1` on success, `0` if `index` is out of range.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// Out parameters may be null; non-null pointers must be valid writable
/// locations.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","E","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// let mut row = 0u32;
/// let mut col = 0u32;
/// let mut id = 0u32;
/// let mut damage = 0u32;
/// let mut move_period_ms = 0f32;
/// let mut enemy_type = -2i32;
/// let ok = unsafe {
///     maze_c_maze_game_get_enemy(
///         ptr, 0, &mut row, &mut col, &mut id, &mut damage, &mut move_period_ms, &mut enemy_type,
///     )
/// };
/// assert_eq!(ok, 1);
/// assert_eq!((row, col), (0, 1));
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_enemy(
    ptr: *mut MazeGameC,
    index: i32,
    out_row: *mut u32,
    out_col: *mut u32,
    out_id: *mut u32,
    out_damage: *mut u32,
    out_move_period_ms: *mut f32,
    out_enemy_type: *mut i32,
) -> u8 {
    let game = unsafe { &(*ptr).game };
    let enemies = game.enemies();
    if index < 0 || index as usize >= enemies.len() {
        return 0;
    }
    let enemy = &enemies[index as usize];
    unsafe {
        if !out_row.is_null() {
            *out_row = enemy.row as u32;
        }
        if !out_col.is_null() {
            *out_col = enemy.col as u32;
        }
        if !out_id.is_null() {
            *out_id = enemy.id;
        }
        if !out_damage.is_null() {
            *out_damage = enemy.damage;
        }
        if !out_move_period_ms.is_null() {
            *out_move_period_ms = enemy.move_period_ms;
        }
        if !out_enemy_type.is_null() {
            *out_enemy_type = enemy_type_to_ffi(enemy.enemy_type);
        }
    }
    1
}

/// Encodes an optional enemy rig override for the FFI boundary: `-1` for
/// "no per-cell override", else the [`maze::EnemyType`] ordinal (`0` = goblin,
/// `1` = ghost). C# maps the ordinal back to its `EnemyType`.
fn enemy_type_to_ffi(enemy_type: Option<maze::EnemyType>) -> i32 {
    match enemy_type {
        None => -1,
        Some(maze::EnemyType::Goblin) => 0,
        Some(maze::EnemyType::Ghost) => 1,
    }
}

/// Returns the number of uncollected treasure cells (live `'T'`).
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","T","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_treasure_count(ptr), 1);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_treasure_count(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    game.treasures().len() as i32
}

/// Retrieves a single uncollected treasure cell by index: its cell, visual
/// style, and resolved reward value. `out_style` carries the
/// [`maze::TreasureStyle`] ordinal (`0` = silver, `1` = gold, `2` = diamonds,
/// `3` = jewels); `out_value` carries the score the treasure awards (per-cell
/// override else the rarity-derived default).
///
/// Returns `1` on success, `0` if `index` is out of range.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// Out parameters may be null; non-null pointers must be valid writable
/// locations.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","T","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// let mut row = 0u32;
/// let mut col = 0u32;
/// let mut style = -1i32;
/// let mut value = 0u32;
/// let ok = unsafe {
///     maze_c_maze_game_get_treasure(ptr, 0, &mut row, &mut col, &mut style, &mut value)
/// };
/// assert_eq!(ok, 1);
/// assert_eq!((row, col), (0, 1));
/// assert_eq!(style, 0); // silver (a bare 'T' default)
/// assert_eq!(value, 10); // Common-tier reward
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_treasure(
    ptr: *mut MazeGameC,
    index: i32,
    out_row: *mut u32,
    out_col: *mut u32,
    out_style: *mut i32,
    out_value: *mut u32,
) -> u8 {
    let game = unsafe { &(*ptr).game };
    let treasures = game.treasures();
    if index < 0 || index as usize >= treasures.len() {
        return 0;
    }
    let ((row, col), style, value) = treasures[index as usize];
    unsafe {
        if !out_row.is_null() {
            *out_row = row as u32;
        }
        if !out_col.is_null() {
            *out_col = col as u32;
        }
        if !out_style.is_null() {
            *out_style = treasure_style_to_ffi(style);
        }
        if !out_value.is_null() {
            *out_value = value;
        }
    }
    1
}

/// Encodes a treasure's visual style for the FFI boundary: the
/// [`maze::TreasureStyle`] ordinal (`0` = silver, `1` = gold, `2` = diamonds,
/// `3` = jewels). A treasure always has a style (a bare `'T'` defaults to
/// silver), so unlike the enemy rig there is no `-1` "none" case. C# maps the
/// ordinal back to its `TreasureStyle`.
fn treasure_style_to_ffi(style: maze::TreasureStyle) -> i32 {
    match style {
        maze::TreasureStyle::Silver => 0,
        maze::TreasureStyle::Gold => 1,
        maze::TreasureStyle::Diamonds => 2,
        maze::TreasureStyle::Jewels => 3,
    }
}

/// Returns the number of uncollected health-pickup cells (live `'H'`).
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","H","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_health_pickup_count(ptr), 1);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_health_pickup_count(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    health_pickup_cells(game).len() as i32
}

/// Retrieves a single uncollected health-pickup cell by index. `out_id`
/// is always written as `0` — pickups have no stable id, the cell
/// coordinate is the natural key. The field is kept on the signature for
/// shape parity with [`maze_c_maze_game_get_enemy`] /
/// [`maze_c_maze_game_get_key`] so callers can use a single
/// `(row, col, id)` row-getter pattern.
///
/// Returns `1` on success, `0` if `index` is out of range.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// Out parameters may be null; non-null pointers must be valid writable
/// locations.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S","H","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// let mut row = 0u32;
/// let mut col = 0u32;
/// let mut id = 99u32;
/// let ok = unsafe { maze_c_maze_game_get_health_pickup(ptr, 0, &mut row, &mut col, &mut id) };
/// assert_eq!(ok, 1);
/// assert_eq!((row, col), (0, 1));
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_health_pickup(
    ptr: *mut MazeGameC,
    index: i32,
    out_row: *mut u32,
    out_col: *mut u32,
    out_id: *mut u32,
) -> u8 {
    let game = unsafe { &(*ptr).game };
    let pickups = health_pickup_cells(game);
    if index < 0 || index as usize >= pickups.len() {
        return 0;
    }
    let (r, c) = pickups[index as usize];
    unsafe {
        if !out_row.is_null() {
            *out_row = r as u32;
        }
        if !out_col.is_null() {
            *out_col = c as u32;
        }
        if !out_id.is_null() {
            *out_id = 0;
        }
    }
    1
}

fn health_pickup_cells(game: &maze::MazeGame) -> Vec<(usize, usize)> {
    game.grid()
        .iter()
        .enumerate()
        .flat_map(|(r, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, &ch)| ch == 'H')
                .map(move |(c, _)| (r, c))
        })
        .collect()
}

// ──────────────────────────────────────────────────────────────────────────────
// MazeGameC — visited cells
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the number of cells the player has visited (including the start cell).
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// assert_eq!(maze_c_maze_game_visited_cell_count(ptr), 1); // start cell
/// maze_c_maze_game_move_player(ptr, 4);
/// assert_eq!(maze_c_maze_game_visited_cell_count(ptr), 2);
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_c_maze_game_visited_cell_count(ptr: *mut MazeGameC) -> i32 {
    let game = unsafe { &(*ptr).game };
    game.visited_cells().len() as i32
}

/// Retrieves a single visited cell by index.
///
/// Writes the cell's row and column into `row_out` and `col_out` respectively.
/// Either output pointer may be null if that value is not needed.
///
/// Returns `1` on success, `0` if `index` is out of range.
///
/// # Safety
///
/// `ptr` must be a non-null pointer returned by [`maze_c_new_maze_game`].
/// `row_out` and `col_out` must each be either null or a valid writable `i32`.
///
/// # Examples
///
/// ```rust
/// use maze_c::*;
/// use std::ffi::CString;
///
/// let json = CString::new(r#"{"grid":[["S"," ","F"]]}"#).unwrap();
/// let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
/// maze_c_maze_game_move_player(ptr, 4); // Right
/// let mut row: i32 = -1;
/// let mut col: i32 = -1;
/// let ok = unsafe { maze_c_maze_game_get_visited_cell(ptr, 0, &mut row, &mut col) };
/// assert_eq!(ok, 1);
/// assert_eq!(row, 0);
/// assert_eq!(col, 0); // start cell
/// maze_c_free_maze_game(ptr);
/// ```
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub unsafe extern "C" fn maze_c_maze_game_get_visited_cell(
    ptr: *mut MazeGameC,
    index: i32,
    row_out: *mut i32,
    col_out: *mut i32,
) -> u8 {
    let game = unsafe { &(*ptr).game };
    let cells = game.visited_cells();
    if index < 0 || index as usize >= cells.len() {
        return 0;
    }
    let (row, col) = cells[index as usize];
    unsafe {
        if !row_out.is_null() {
            *row_out = row as i32;
        }
        if !col_out.is_null() {
            *col_out = col as i32;
        }
    }
    1
}

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
    fn can_set_key_cells() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 5, 5);
        let ok = maze_c_maze_set_key_cells(ptr, 1, 1, 3, 3);
        assert_eq!(ok, 1);
        for r in 0..5_u32 {
            for c in 0..5_u32 {
                let mut ct: u32 = 99;
                unsafe { maze_c_maze_get_cell_type(ptr, r, c, &mut ct) };
                let inside = (1..=3).contains(&r) && (1..=3).contains(&c);
                assert_eq!(
                    ct,
                    if inside { 4 } else { 0 },
                    "expected {} at ({r},{c})",
                    if inside { "Key" } else { "Empty" },
                );
            }
        }
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn can_set_door_cells() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 5, 5);
        let ok = maze_c_maze_set_door_cells(ptr, 0, 4, 2, 4);
        assert_eq!(ok, 1);
        for r in 0..5_u32 {
            for c in 0..5_u32 {
                let mut ct: u32 = 99;
                unsafe { maze_c_maze_get_cell_type(ptr, r, c, &mut ct) };
                let inside = (0..=2).contains(&r) && c == 4;
                assert_eq!(
                    ct,
                    if inside { 5 } else { 0 },
                    "expected {} at ({r},{c})",
                    if inside { "Door" } else { "Empty" },
                );
            }
        }
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn can_set_enemy_cells() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 5, 5);
        let ok = maze_c_maze_set_enemy_cells(ptr, 1, 1, 2, 2);
        assert_eq!(ok, 1);
        for r in 0..5_u32 {
            for c in 0..5_u32 {
                let mut ct: u32 = 99;
                unsafe { maze_c_maze_get_cell_type(ptr, r, c, &mut ct) };
                let inside = (1..=2).contains(&r) && (1..=2).contains(&c);
                assert_eq!(
                    ct,
                    if inside { 6 } else { 0 },
                    "expected {} at ({r},{c})",
                    if inside { "Enemy" } else { "Empty" },
                );
            }
        }
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn can_set_health_cells() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 5, 5);
        let ok = maze_c_maze_set_health_cells(ptr, 0, 4, 2, 4);
        assert_eq!(ok, 1);
        for r in 0..5_u32 {
            for c in 0..5_u32 {
                let mut ct: u32 = 99;
                unsafe { maze_c_maze_get_cell_type(ptr, r, c, &mut ct) };
                let inside = (0..=2).contains(&r) && c == 4;
                assert_eq!(
                    ct,
                    if inside { 7 } else { 0 },
                    "expected {} at ({r},{c})",
                    if inside { "Health" } else { "Empty" },
                );
            }
        }
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn set_key_cells_fails_for_empty_maze() {
        let ptr = new_maze();
        let ok = maze_c_maze_set_key_cells(ptr, 0, 0, 0, 0);
        assert_eq!(ok, 0);
        assert!(last_error_str().is_some());
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn set_door_cells_fails_for_invalid_end_location() {
        let ptr = new_maze();
        maze_c_maze_resize(ptr, 3, 3);
        let ok = maze_c_maze_set_door_cells(ptr, 0, 0, 5, 5);
        assert_eq!(ok, 0);
        assert!(last_error_str().is_some());
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
    fn generate_maze_places_enemies_and_health() {
        let ptr = new_maze();
        let opts = maze_c_new_generator_options(15, 15, 0, 123);
        maze_c_generator_options_set_enemy_count(opts, 3);
        maze_c_generator_options_set_health_count(opts, 2);
        let ok = maze_c_maze_generate(ptr, opts);
        assert_eq!(ok, 1, "generate failed: {:?}", last_error_str());
        let mw = unsafe { &*ptr };
        let mut enemies = 0;
        let mut health = 0;
        for row in &mw.maze.definition.grid {
            for &ch in row {
                match ch {
                    'E' => enemies += 1,
                    'H' => health += 1,
                    _ => {}
                }
            }
        }
        assert_eq!(enemies, 3, "expected 3 enemy cells");
        assert_eq!(health, 2, "expected 2 health cells");
        maze_c_free_generator_options(opts);
        unsafe { maze_c_free_maze(ptr) };
    }

    #[test]
    fn generate_maze_places_treasure() {
        let ptr = new_maze();
        let opts = maze_c_new_generator_options(15, 15, 0, 123);
        maze_c_generator_options_set_treasure_count(opts, 4);
        let ok = maze_c_maze_generate(ptr, opts);
        assert_eq!(ok, 1, "generate failed: {:?}", last_error_str());
        let mw = unsafe { &*ptr };
        let treasure = mw
            .maze
            .definition
            .grid
            .iter()
            .flatten()
            .filter(|&&ch| ch == 'T')
            .count();
        assert_eq!(treasure, 4, "expected 4 treasure cells");
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

    // ── MazeGameC helpers ─────────────────────────────────────────────────────

    /// 1×3 maze: S _ F  (single row, player starts at col 0, finish at col 2)
    fn simple_game_json() -> CString {
        CString::new(
            r#"{"grid":[["S"," ","F"]]}"#,
        )
        .unwrap()
    }

    /// 3×3 maze with walls:
    ///   S  _  _
    ///   W  W  _
    ///   _  _  F
    fn walled_game_json() -> CString {
        CString::new(
            r#"{"grid":[["S"," "," "],["W","W"," "],[" "," ","F"]]}"#,
        )
        .unwrap()
    }

    fn new_game(json: &CString) -> *mut MazeGameC {
        unsafe { maze_c_new_maze_game(json.as_ptr()) }
    }

    // ── MazeGameC — lifecycle ─────────────────────────────────────────────────

    #[test]
    fn game_create_returns_non_null() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        assert!(!ptr.is_null());
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_create_invalid_json_returns_null_and_sets_error() {
        let json = CString::new("{not valid json}").unwrap();
        let ptr = unsafe { maze_c_new_maze_game(json.as_ptr()) };
        assert!(ptr.is_null());
        assert!(last_error_str().is_some());
    }

    #[test]
    fn game_free_null_is_safe() {
        maze_c_free_maze_game(std::ptr::null_mut());
    }

    #[test]
    fn game_create_increments_object_count() {
        let before = maze_c_get_num_objects_allocated();
        let json = simple_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_get_num_objects_allocated(), before + 1);
        maze_c_free_maze_game(ptr);
        assert_eq!(maze_c_get_num_objects_allocated(), before);
    }

    // ── MazeGameC — initial state ─────────────────────────────────────────────

    #[test]
    fn game_initial_player_position_is_start_cell() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_player_row(ptr), 0);
        assert_eq!(maze_c_maze_game_player_col(ptr), 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_initial_is_not_complete() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_is_complete(ptr), 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_initial_is_not_lost() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_is_lost(ptr), 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_initial_lose_reason_is_none() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_lose_reason(ptr), 0); // None
        maze_c_free_maze_game(ptr);
    }

    // ── MazeGameC — bag / pickup ──────────────────────────────────────────────

    fn key_game_json() -> CString {
        // 1 row, 3 cols: S at (0,0), K at (0,1), F at (0,2)
        CString::new(r#"{"grid":[["S","K","F"]]}"#).unwrap()
    }

    #[test]
    fn game_initial_bag_is_empty() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_bag_count(ptr), 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_pickup_on_non_key_cell_returns_zero() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        let mut k: u32 = 99;
        let mut id: u32 = 99;
        let ok = unsafe { maze_c_maze_game_pickup(ptr, &mut k, &mut id) };
        assert_eq!(ok, 0);
        assert_eq!(maze_c_maze_game_bag_count(ptr), 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_move_onto_key_auto_collects_and_grows_bag() {
        let json = key_game_json();
        let ptr = new_game(&json);
        maze_c_maze_game_move_player(ptr, 4); // Right → key cell, auto-collected
        assert_eq!(maze_c_maze_game_bag_count(ptr), 1);
        // The cell is already cleared, so an explicit pickup finds nothing.
        let mut k: u32 = 99;
        let mut id: u32 = 99;
        let ok = unsafe { maze_c_maze_game_pickup(ptr, &mut k, &mut id) };
        assert_eq!(ok, 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_bag_item_returns_auto_collected_key() {
        let json = key_game_json();
        let ptr = new_game(&json);
        maze_c_maze_game_move_player(ptr, 4); // onto the key — auto-collected
        let mut k: u32 = 99;
        let mut id: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_bag_item(ptr, 0, &mut k, &mut id) };
        assert_eq!(ok, 1);
        assert_eq!(k, 0); // Key
        assert_eq!(id, 0); // first key's id
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_bag_item_out_of_range_returns_zero() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        let mut k: u32 = 99;
        let mut id: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_bag_item(ptr, 0, &mut k, &mut id) };
        assert_eq!(ok, 0); // empty bag → 0 out of range
        let ok = unsafe { maze_c_maze_game_get_bag_item(ptr, -1, &mut k, &mut id) };
        assert_eq!(ok, 0); // negative index → 0
        maze_c_free_maze_game(ptr);
    }

    // ── MazeGameC — doors / tick / events ─────────────────────────────────────

    fn door_game_json() -> CString {
        // 1 row, 4 cols: S at (0,0), K at (0,1), D at (0,2), F at (0,3)
        CString::new(r#"{"grid":[["S","K","D","F"]]}"#).unwrap()
    }

    #[test]
    fn game_initial_door_count_for_door_grid() {
        let json = door_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_door_count(ptr), 1);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_door_returns_locked_initially() {
        let json = door_game_json();
        let ptr = new_game(&json);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let mut state: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_door(ptr, 0, &mut row, &mut col, &mut state) };
        assert_eq!(ok, 1);
        assert_eq!(row, 0);
        assert_eq!(col, 2);
        assert_eq!(state, 0); // Locked
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_tick_emits_door_opened_after_unlocking() {
        let json = door_game_json();
        let ptr = new_game(&json);
        // Walk onto K → auto-collected; flush the resulting KeyCollected event.
        maze_c_maze_game_move_player(ptr, 4); // Right → K
        maze_c_maze_game_tick(ptr, 0.0);
        // Step into D → StartedUnlocking (5)
        let result = maze_c_maze_game_move_player(ptr, 4);
        assert_eq!(result, 5);
        // Tick a full second → DoorOpened event, count = 1.
        let count = maze_c_maze_game_tick(ptr, 1000.0);
        assert_eq!(count, 1);
        assert_eq!(maze_c_maze_game_tick_event_count(ptr), 1);
        // Read the event back.
        let mut kind: u32 = 99;
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_tick_event(ptr, 0, &mut kind, &mut row, &mut col) };
        assert_eq!(ok, 1);
        assert_eq!(kind, 0); // DoorOpened
        assert_eq!((row, col), (0, 2));
        // Door is now Open (state code 2).
        let mut drow: u32 = 0;
        let mut dcol: u32 = 0;
        let mut state: u32 = 99;
        unsafe { maze_c_maze_game_get_door(ptr, 0, &mut drow, &mut dcol, &mut state) };
        assert_eq!(state, 2);
        // StartedUnlocking did not move the player — still on K cell (0,1).
        // Step right onto the now-open door (0,2), then right again to F (0,3).
        let result = maze_c_maze_game_move_player(ptr, 4);
        assert_eq!(result, 1); // Moved through open door
        let result = maze_c_maze_game_move_player(ptr, 4);
        assert_eq!(result, 3); // Complete
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_tick_event_count_is_zero_before_first_tick() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_tick_event_count(ptr), 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_tick_event_out_of_range_returns_zero() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        let mut k: u32 = 99;
        let mut r: u32 = 99;
        let mut c: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_tick_event(ptr, 0, &mut k, &mut r, &mut c) };
        assert_eq!(ok, 0);
        maze_c_free_maze_game(ptr);
    }

    // ── MazeGameC — HP / enemies / health pickups / extended tick events ─────

    #[test]
    fn game_hp_and_max_hp_default_to_three() {
        let json = CString::new(r#"{"grid":[["S","E","F"]]}"#).unwrap();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_hp(ptr), 3);
        assert_eq!(maze_c_maze_game_max_hp(ptr), 3);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_enemy_count_and_get_enemy() {
        let json = CString::new(r#"{"grid":[["S","E","F"]]}"#).unwrap();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_enemy_count(ptr), 1);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let mut id: u32 = 99;
        let mut damage: u32 = 99;
        let mut move_period_ms: f32 = -1.0;
        let mut enemy_type: i32 = 99;
        let ok = unsafe {
            maze_c_maze_game_get_enemy(
                ptr,
                0,
                &mut row,
                &mut col,
                &mut id,
                &mut damage,
                &mut move_period_ms,
                &mut enemy_type,
            )
        };
        assert_eq!(ok, 1);
        assert_eq!((row, col), (0, 1));
        assert_eq!(id, 0);
        // Defaults for an enemy with no per-cell override.
        assert_eq!(damage, 1);
        assert_eq!(move_period_ms, 1500.0);
        assert_eq!(enemy_type, -1); // no rig override
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_enemy_surfaces_per_cell_override() {
        let json = CString::new(
            r#"{"grid":[["S",[{"type":"E","enemyType":"ghost","damage":3,"movePeriodMs":600.0}],"F"]]}"#,
        )
        .unwrap();
        let ptr = new_game(&json);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let mut id: u32 = 99;
        let mut damage: u32 = 99;
        let mut move_period_ms: f32 = -1.0;
        let mut enemy_type: i32 = 99;
        let ok = unsafe {
            maze_c_maze_game_get_enemy(
                ptr,
                0,
                &mut row,
                &mut col,
                &mut id,
                &mut damage,
                &mut move_period_ms,
                &mut enemy_type,
            )
        };
        assert_eq!(ok, 1);
        assert_eq!(damage, 3);
        assert_eq!(move_period_ms, 600.0);
        assert_eq!(enemy_type, 1); // ghost
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_treasure_count_and_get_surfaces_style_and_value() {
        // A gold treasure with an explicit value override.
        let json =
            CString::new(r#"{"grid":[["S",[{"type":"T","style":"gold","value":250}],"F"]]}"#)
                .unwrap();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_treasure_count(ptr), 1);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let mut style: i32 = 99;
        let mut value: u32 = 99;
        let ok = unsafe {
            maze_c_maze_game_get_treasure(ptr, 0, &mut row, &mut col, &mut style, &mut value)
        };
        assert_eq!(ok, 1);
        assert_eq!((row, col), (0, 1));
        assert_eq!(style, 1); // gold
        assert_eq!(value, 250);
        // Out-of-range index returns 0.
        let oob = unsafe {
            maze_c_maze_game_get_treasure(ptr, 9, &mut row, &mut col, &mut style, &mut value)
        };
        assert_eq!(oob, 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_treasure_defaults_a_bare_cell_to_silver_and_ten() {
        let json = CString::new(r#"{"grid":[["S","T","F"]]}"#).unwrap();
        let ptr = new_game(&json);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let mut style: i32 = 99;
        let mut value: u32 = 99;
        let ok = unsafe {
            maze_c_maze_game_get_treasure(ptr, 0, &mut row, &mut col, &mut style, &mut value)
        };
        assert_eq!(ok, 1);
        assert_eq!(style, 0); // silver (default)
        assert_eq!(value, 10); // Common default
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn maze_with_enemy_override_round_trips_through_json() {
        // The char-or-array cell form survives a from_json -> to_json round-trip
        // at the FFI boundary (serde in data_model does the work).
        let src = r#"{"id":"m","name":"n","definition":{"grid":[["S",[{"type":"E","damage":2}],"F"]]}}"#;
        let json = CString::new(src).unwrap();
        let maze_ptr = maze_c_new_maze();
        let rc = unsafe { maze_c_maze_from_json(maze_ptr, json.as_ptr()) };
        assert_eq!(rc, 1); // 1 = success
        let out = maze_c_maze_to_json(maze_ptr);
        let round_tripped = unsafe { CStr::from_ptr(out) }.to_str().unwrap().to_string();
        assert!(
            round_tripped.contains(r#"[{"type":"E","damage":2}]"#),
            "override array form missing from round-trip: {round_tripped}"
        );
        unsafe { maze_c_free_string(out) };
        maze_c_free_maze(maze_ptr);
    }

    #[test]
    fn maze_get_set_clear_cell_entity_round_trip() {
        let maze_ptr = maze_c_new_maze();
        maze_c_maze_resize(maze_ptr, 1, 3);
        maze_c_maze_set_enemy_cells(maze_ptr, 0, 1, 0, 1);
        // No override yet → null.
        assert!(maze_c_maze_get_cell_entity(maze_ptr, 0, 1).is_null());

        let entity = CString::new(r#"{"type":"E","enemyType":"ghost","damage":2}"#).unwrap();
        let rc = unsafe { maze_c_maze_set_cell_entity(maze_ptr, 0, 1, entity.as_ptr()) };
        assert_eq!(rc, 1);

        let got = maze_c_maze_get_cell_entity(maze_ptr, 0, 1);
        assert!(!got.is_null());
        let got_str = unsafe { CStr::from_ptr(got) }.to_str().unwrap().to_string();
        assert!(got_str.contains(r#""type":"E""#), "got: {got_str}");
        assert!(got_str.contains(r#""enemyType":"ghost""#), "got: {got_str}");
        assert!(got_str.contains(r#""damage":2"#), "got: {got_str}");
        unsafe { maze_c_free_string(got) };

        // Type mismatch (cell is 'E', entity claims 'H') is rejected.
        let mismatch = CString::new(r#"{"type":"H","healAmount":2}"#).unwrap();
        let rc2 = unsafe { maze_c_maze_set_cell_entity(maze_ptr, 0, 1, mismatch.as_ptr()) };
        assert_eq!(rc2, 0);

        assert_eq!(maze_c_maze_clear_cell_entity(maze_ptr, 0, 1), 1);
        assert!(maze_c_maze_get_cell_entity(maze_ptr, 0, 1).is_null());
        maze_c_free_maze(maze_ptr);
    }

    #[test]
    fn maze_set_get_wall_cell_entity_round_trip() {
        // Wall cells are overridable too (the `wall_type` per-cell override),
        // and the type-vs-char check accepts a `W` entity on a `W` cell.
        let maze_ptr = maze_c_new_maze();
        maze_c_maze_resize(maze_ptr, 1, 3);
        unsafe { maze_c_maze_set_wall_cells(maze_ptr, 0, 1, 0, 1) };

        let entity = CString::new(r#"{"type":"W","wallType":"lava"}"#).unwrap();
        let rc = unsafe { maze_c_maze_set_cell_entity(maze_ptr, 0, 1, entity.as_ptr()) };
        assert_eq!(rc, 1);

        let got = maze_c_maze_get_cell_entity(maze_ptr, 0, 1);
        assert!(!got.is_null());
        let got_str = unsafe { CStr::from_ptr(got) }.to_str().unwrap().to_string();
        assert!(got_str.contains(r#""type":"W""#), "got: {got_str}");
        assert!(got_str.contains(r#""wallType":"lava""#), "got: {got_str}");
        unsafe { maze_c_free_string(got) };
        maze_c_free_maze(maze_ptr);
    }

    #[test]
    fn game_health_pickup_count_and_get() {
        let json = CString::new(r#"{"grid":[["S","H","F"]]}"#).unwrap();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_health_pickup_count(ptr), 1);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let mut id: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_health_pickup(ptr, 0, &mut row, &mut col, &mut id) };
        assert_eq!(ok, 1);
        assert_eq!((row, col), (0, 1));
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_tick_emits_enemy_moved_and_player_damaged_with_payloads() {
        // Enemy at (0,1) chasing the player at (0,0). One full move period:
        // the enemy steps onto the player → EnemyMoved (kind 1) then
        // PlayerDamaged (kind 2).
        let json = CString::new(r#"{"grid":[["S","E","F"]]}"#).unwrap();
        let ptr = new_game(&json);
        let count = maze_c_maze_game_tick(ptr, 1500.0);
        assert_eq!(count, 2);

        let mut kind: u32 = 99;
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        unsafe { maze_c_maze_game_get_tick_event(ptr, 0, &mut kind, &mut row, &mut col) };
        assert_eq!(kind, 1); // EnemyMoved
        assert_eq!((row, col), (0, 0));
        let mut id: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_tick_event_payload(ptr, 0, &mut id) };
        assert_eq!(ok, 1);
        assert_eq!(id, 0); // enemy id

        unsafe { maze_c_maze_game_get_tick_event(ptr, 1, &mut kind, &mut row, &mut col) };
        assert_eq!(kind, 2); // PlayerDamaged
        assert_eq!((row, col), (0, 0)); // no cell for damage
        let mut hp_after: u32 = 99;
        unsafe { maze_c_maze_game_get_tick_event_payload(ptr, 1, &mut hp_after) };
        assert_eq!(hp_after, 2);
        assert_eq!(maze_c_maze_game_hp(ptr), 2);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_tick_emits_player_healed_after_damage() {
        // Step onto the enemy (damage 3 → 2), flush, then step onto the health
        // pickup below max HP → PlayerHealed (kind 3) at the pickup cell.
        let json = CString::new(r#"{"grid":[["S","E","H","F"]]}"#).unwrap();
        let ptr = new_game(&json);
        maze_c_maze_game_move_player(ptr, 4); // Right onto 'E' → damage queued
        maze_c_maze_game_tick(ptr, 0.0); // flush; hp 3 → 2
        assert_eq!(maze_c_maze_game_hp(ptr), 2);
        maze_c_maze_game_move_player(ptr, 4); // Right onto 'H' (hp 2 < 3) → heal queued
        let count = maze_c_maze_game_tick(ptr, 0.0);
        assert_eq!(count, 1);
        let mut kind: u32 = 99;
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        unsafe { maze_c_maze_game_get_tick_event(ptr, 0, &mut kind, &mut row, &mut col) };
        assert_eq!(kind, 3); // PlayerHealed
        assert_eq!((row, col), (0, 2)); // consumed pickup cell
        let mut hp_after: u32 = 99;
        unsafe { maze_c_maze_game_get_tick_event_payload(ptr, 0, &mut hp_after) };
        assert_eq!(hp_after, 3);
        assert_eq!(maze_c_maze_game_hp(ptr), 3);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_tick_emits_player_not_healed_with_string_payload() {
        // Player at full HP walks onto 'H' → the pickup is spared →
        // PlayerNotHealed (kind 4), reason 0, default message via the
        // two-call string-payload protocol.
        let json = CString::new(r#"{"grid":[["S","H","F"]]}"#).unwrap();
        let ptr = new_game(&json);
        maze_c_maze_game_move_player(ptr, 4); // Right onto 'H'
        let count = maze_c_maze_game_tick(ptr, 0.0);
        assert_eq!(count, 1);

        let mut kind: u32 = 99;
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        unsafe { maze_c_maze_game_get_tick_event(ptr, 0, &mut kind, &mut row, &mut col) };
        assert_eq!(kind, 4); // PlayerNotHealed
        assert_eq!((row, col), (0, 1)); // spared pickup cell
        let mut reason: u32 = 99;
        unsafe { maze_c_maze_game_get_tick_event_payload(ptr, 0, &mut reason) };
        assert_eq!(reason, 0); // AlreadyAtMaxHp

        let mut len: u32 = 0;
        let ok = unsafe {
            maze_c_maze_game_get_tick_event_string_payload(ptr, 0, std::ptr::null_mut(), &mut len)
        };
        assert_eq!(ok, 1);
        let mut buf = vec![0u8; len as usize];
        unsafe {
            maze_c_maze_game_get_tick_event_string_payload(ptr, 0, buf.as_mut_ptr(), &mut len)
        };
        assert_eq!(String::from_utf8(buf).unwrap(), "Already at maximum health");
        // The pickup was spared — still present.
        assert_eq!(maze_c_maze_game_health_pickup_count(ptr), 1);
        maze_c_free_maze_game(ptr);
    }

    // ── MazeGameC — keys ──────────────────────────────────────────────────────

    fn two_key_game_json() -> CString {
        // 1 row, 4 cols: S K K F
        CString::new(r#"{"grid":[["S","K","K","F"]]}"#).unwrap()
    }

    #[test]
    fn game_key_count_reports_uncollected_keys() {
        let json = two_key_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_key_count(ptr), 2);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_key_returns_first_key_cell() {
        let json = two_key_game_json();
        let ptr = new_game(&json);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let mut id: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_key(ptr, 0, &mut row, &mut col, &mut id) };
        assert_eq!(ok, 1);
        assert_eq!(row, 0);
        assert_eq!(col, 1);
        // Verify we can read the second key as well, with a distinct id.
        let mut row2: u32 = 99;
        let mut col2: u32 = 99;
        let mut id2: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_key(ptr, 1, &mut row2, &mut col2, &mut id2) };
        assert_eq!(ok, 1);
        assert_eq!(row2, 0);
        assert_eq!(col2, 2);
        assert_ne!(id, id2);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_key_collection_removes_key_from_list() {
        let json = two_key_game_json();
        let ptr = new_game(&json);
        // Step right onto the first key and pick it up.
        maze_c_maze_game_move_player(ptr, 4);
        let mut k: u32 = 0;
        let mut id: u32 = 0;
        unsafe { maze_c_maze_game_pickup(ptr, &mut k, &mut id) };
        // Only one key remains.
        assert_eq!(maze_c_maze_game_key_count(ptr), 1);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let mut rem_id: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_key(ptr, 0, &mut row, &mut col, &mut rem_id) };
        assert_eq!(ok, 1);
        // The remaining key is the second cell (0,2) — and its id is preserved.
        assert_eq!((row, col), (0, 2));
        assert_ne!(rem_id, id);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_key_out_of_range_returns_zero() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        let mut row: u32 = 99;
        let mut col: u32 = 99;
        let mut id: u32 = 99;
        let ok = unsafe { maze_c_maze_game_get_key(ptr, 0, &mut row, &mut col, &mut id) };
        assert_eq!(ok, 0); // no keys → out of range
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_decoy_door_walk_through_sets_stranded_lose_state() {
        // Mirrors the maze-crate test
        // `decoy_door_with_only_one_key_strands_on_walk_through`. One key,
        // one real door, one decoy door — detouring through the decoy
        // burns the only key, and walking through it strands the player.
        #[rustfmt::skip]
        let json = CString::new(
            r#"{"grid":[["S","K","D","F"],["W","D","W","W"],["W"," ","W","W"]]}"#
        ).unwrap();
        let ptr = new_game(&json);
        // Grab the key at (0,1).
        maze_c_maze_game_move_player(ptr, 4); // Right
        let mut k: u32 = 0;
        let mut id: u32 = 0;
        unsafe { maze_c_maze_game_pickup(ptr, &mut k, &mut id) };
        // Step into the decoy door at (1,1).
        maze_c_maze_game_move_player(ptr, 2); // Down → StartedUnlocking
        // Tick long enough to open it.
        maze_c_maze_game_tick(ptr, 1000.0);
        // Walk through the decoy — this is the strand trigger.
        let result = maze_c_maze_game_move_player(ptr, 2);
        assert_eq!(result, 6); // Stranded
        assert_eq!(maze_c_maze_game_is_lost(ptr), 1);
        assert_eq!(maze_c_maze_game_lose_reason(ptr), 1); // Stranded
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_initial_visited_cell_count_is_one() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        assert_eq!(maze_c_maze_game_visited_cell_count(ptr), 1);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_initial_visited_cell_is_start() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        let mut row: i32 = -1;
        let mut col: i32 = -1;
        let ok = unsafe { maze_c_maze_game_get_visited_cell(ptr, 0, &mut row, &mut col) };
        assert_eq!(ok, 1);
        assert_eq!(row, 0);
        assert_eq!(col, 0);
        maze_c_free_maze_game(ptr);
    }

    // ── MazeGameC — movement ──────────────────────────────────────────────────

    #[test]
    fn game_move_right_returns_moved() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        let result = maze_c_maze_game_move_player(ptr, 4); // Right
        assert_eq!(result, 1, "expected Moved (1)");
        assert_eq!(maze_c_maze_game_player_col(ptr), 1);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_move_into_wall_returns_blocked() {
        let json = walled_game_json();
        let ptr = new_game(&json);
        // Player is at (0,0). Down leads to (1,0) which is 'W'.
        let result = maze_c_maze_game_move_player(ptr, 2); // Down
        assert_eq!(result, 2, "expected Blocked (2)");
        assert_eq!(maze_c_maze_game_player_row(ptr), 0); // unchanged
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_move_out_of_bounds_returns_blocked() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        // Player is at (0,0). Moving Up is out of bounds.
        let result = maze_c_maze_game_move_player(ptr, 1); // Up
        assert_eq!(result, 2, "expected Blocked (2)");
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_move_to_finish_returns_complete() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        maze_c_maze_game_move_player(ptr, 4); // Right → (0,1)
        let result = maze_c_maze_game_move_player(ptr, 4); // Right → (0,2) = F
        assert_eq!(result, 3, "expected Complete (3)");
        assert_eq!(maze_c_maze_game_is_complete(ptr), 1);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_move_unknown_direction_returns_minus_one() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        let result = maze_c_maze_game_move_player(ptr, 99);
        assert_eq!(result, -1);
        maze_c_free_maze_game(ptr);
    }

    // ── MazeGameC — direction tracking ───────────────────────────────────────

    #[test]
    fn game_player_direction_updates_after_move() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        maze_c_maze_game_move_player(ptr, 4); // Right
        assert_eq!(maze_c_maze_game_player_direction(ptr), 4); // Right
        maze_c_free_maze_game(ptr);
    }

    // ── MazeGameC — visited cells ─────────────────────────────────────────────

    #[test]
    fn game_visited_cells_accumulate() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        maze_c_maze_game_move_player(ptr, 4); // Right → (0,1)
        assert_eq!(maze_c_maze_game_visited_cell_count(ptr), 2);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_visited_cell_out_of_range_returns_zero() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        let mut row: i32 = -1;
        let mut col: i32 = -1;
        let ok = unsafe { maze_c_maze_game_get_visited_cell(ptr, 99, &mut row, &mut col) };
        assert_eq!(ok, 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_get_visited_cell_negative_index_returns_zero() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        let mut row: i32 = -1;
        let mut col: i32 = -1;
        let ok = unsafe { maze_c_maze_game_get_visited_cell(ptr, -1, &mut row, &mut col) };
        assert_eq!(ok, 0);
        maze_c_free_maze_game(ptr);
    }

    #[test]
    fn game_visited_cell_order_matches_movement() {
        let json = simple_game_json();
        let ptr = new_game(&json);
        maze_c_maze_game_move_player(ptr, 4); // Right → (0,1)
        maze_c_maze_game_move_player(ptr, 4); // Right → (0,2) = F

        let mut row: i32 = -1;
        let mut col: i32 = -1;

        unsafe { maze_c_maze_game_get_visited_cell(ptr, 0, &mut row, &mut col) };
        assert_eq!((row, col), (0, 0));

        unsafe { maze_c_maze_game_get_visited_cell(ptr, 1, &mut row, &mut col) };
        assert_eq!((row, col), (0, 1));

        unsafe { maze_c_maze_game_get_visited_cell(ptr, 2, &mut row, &mut col) };
        assert_eq!((row, col), (0, 2));

        maze_c_free_maze_game(ptr);
    }

    // ── MazeGameC — multiple independent sessions ─────────────────────────────

    #[test]
    fn multiple_independent_game_sessions() {
        let json = simple_game_json();
        let ptr1 = new_game(&json);
        let ptr2 = new_game(&json);
        maze_c_maze_game_move_player(ptr1, 4); // move ptr1 right
        // ptr2 should be unaffected
        assert_eq!(maze_c_maze_game_player_col(ptr1), 1);
        assert_eq!(maze_c_maze_game_player_col(ptr2), 0);
        maze_c_free_maze_game(ptr1);
        maze_c_free_maze_game(ptr2);
    }
}
