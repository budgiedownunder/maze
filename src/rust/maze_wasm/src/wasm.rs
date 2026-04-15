use crate::wasm_common::{new_maze, to_cell_type_enum, MazeWasm};
use data_model::MazePoint;
use maze::{MazeSolver, MazeSolution};
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;
#[cfg(feature = "wasm-lite")]
use crate::wasm_common::{to_generation_algorithm, GenerationAlgorithmWasm};
#[cfg(feature = "wasm-lite")]
use maze::{Generator, GeneratorOptions};
/// Creates a new, empty `MazeWasm`
///
/// # Returns
///
/// Pointer to `MazeWasm`
///
#[no_mangle]
pub extern "C" fn new_maze_wasm() -> *mut MazeWasm {
    let maze = Box::new(MazeWasm { maze: new_maze() });
    increment_num_objects_allocated();
    Box::into_raw(maze)
}
/// Frees a `MazeWasm` pointer
///
/// # Returns
///
/// Nothing
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn free_maze_wasm(maze_wasm: *mut MazeWasm) {
    if !maze_wasm.is_null() {
        unsafe {
            let _ = Box::from_raw(maze_wasm); // This automatically frees the memory
            decrement_num_objects_allocated();
        }
    }
}
/// Gets the row count associated with a `MazeWasm`
///
/// # Returns
///
/// Row count
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_get_row_count(maze_wasm: *mut MazeWasm) -> u32 {
    let maze_wasm = unsafe { &*maze_wasm };
    maze_wasm.maze.definition.row_count() as u32
}
/// Gets the column count associated with a `MazeWasm`
///
/// # Returns
///
/// Column count
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_get_col_count(maze_wasm: *mut MazeWasm) -> u32 {
    let maze_wasm = unsafe { &*maze_wasm };
    maze_wasm.maze.definition.col_count() as u32
}
/// Gets the cell type associated with a `MazeWasm` cell
///
/// # Returns
///
/// A `MazeWasmResult` containing either the cell type (as the `value_ptr`) if successful, or an `error_ptr`
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_get_cell_type(maze_wasm: *mut MazeWasm, row: u32, col: u32) -> u32 {
    let maze_wasm = unsafe { &*maze_wasm };
    let row = row as usize;
    let col = col as usize;
    if row >= maze_wasm.maze.definition.row_count() {
        return create_maze_wasm_error_result(Some(&format!("row index ({row}) out of bounds")));
    }
    if col >= maze_wasm.maze.definition.col_count() {
        return create_maze_wasm_error_result(Some(&format!(
            "column index ({col}) out of bounds"
        )));
    }
    create_maze_wasm_enum_result(to_cell_type_enum(maze_wasm.maze.definition.grid[row][col]) as u32)
}
/// Sets the start cell in a `MazeWasm`
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_set_start_cell(
    maze_wasm: *mut MazeWasm,
    start_row: u32,
    start_col: u32,
) -> u32 {
    let maze_wasm = unsafe { &mut *maze_wasm };
    let mut error_ptr: u32 = 0;
    if let Err(error) = maze_wasm.maze.definition.set_start(Some(MazePoint {
        row: start_row as usize,
        col: start_col as usize,
    })) {
        error_ptr = create_maze_wasm_error_ptr(error.to_string().as_str());
    }
    error_ptr
}
/// Gets the start cell  associated with a `MazeWasm`
///
/// # Returns
///
/// A `MazeWasmResult` containing either the start cell point (as the `value_ptr`) if successful, 
/// or an `error_ptr``
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_get_start_cell(maze_wasm: *mut MazeWasm) -> u32 {
    let maze_wasm = unsafe { &*maze_wasm };
    if let Some(start) = maze_wasm.maze.definition.get_start() {
        return create_maze_wasm_point_result(&start);
    }
    create_maze_wasm_error_result(Some("no start cell defined"))
}
/// Sets the finish cell in a `MazeWasm`
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_set_finish_cell(
    maze_wasm: *mut MazeWasm,
    finish_row: u32,
    finish_col: u32,
) -> u32 {
    let maze_wasm = unsafe { &mut *maze_wasm };
    let mut error_ptr: u32 = 0;
    if let Err(error) = maze_wasm.maze.definition.set_finish(Some(MazePoint {
        row: finish_row as usize,
        col: finish_col as usize,
    })) {
        error_ptr = create_maze_wasm_error_ptr(error.to_string().as_str());
    }
    error_ptr
}
/// Gets the finish cell  associated with a `MazeWasm`
///
/// # Returns
///
/// A `MazeWasmResult` containing either the finish cell point (as the `value_ptr`) if successful, 
/// or an `error_ptr``
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_get_finish_cell(maze_wasm: *mut MazeWasm) -> u32 {
    let maze_wasm = unsafe { &*maze_wasm };
    if let Some(finish) = maze_wasm.maze.definition.get_finish() {
        return create_maze_wasm_point_result(&finish);
    }
    create_maze_wasm_error_result(Some("no finish cell defined"))
}
/// Sets cells values in a `MazeWasm`
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
fn set_cell_values(
    maze_wasm: *mut MazeWasm,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
    modify_char: char,
) -> u32 {
    let maze_wasm = unsafe { &mut *maze_wasm };
    let mut error_ptr: u32 = 0;
    if let Err(error) = maze_wasm.maze.definition.set_value(
        MazePoint {
            row: start_row as usize,
            col: start_col as usize,
        },
        MazePoint {
            row: end_row as usize,
            col: end_col as usize,
        },
        modify_char,
    ) {
        error_ptr = create_maze_wasm_error_ptr(error.to_string().as_str());
    }
    error_ptr
}
/// Sets cells to walls in a `MazeWasm`
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[no_mangle]
pub extern "C" fn maze_wasm_set_wall_cells(
    maze_wasm: *mut MazeWasm,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> u32 {
    set_cell_values(maze_wasm, start_row, start_col, end_row, end_col, 'W')
}

/// Sets cells values in a `MazeWasm` to empty
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[no_mangle]
pub extern "C" fn maze_wasm_clear_cells(
    maze_wasm: *mut MazeWasm,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> u32 {
    set_cell_values(maze_wasm, start_row, start_col, end_row, end_col, ' ')
}
/// Resizes a `MazeWasm`
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[allow(clippy::not_unsafe_ptr_arg_deref)] 
#[no_mangle]
pub extern "C" fn maze_wasm_resize(
    maze_wasm: *mut MazeWasm,
    new_row_count: u32,
    new_col_count: u32,
) {
    let maze_wasm = unsafe { &mut *maze_wasm };
    maze_wasm
        .maze
        .definition
        .resize(new_row_count as usize, new_col_count as usize);
}
/// Resets a `MazeWasm` to empty
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_reset(maze_wasm: *mut MazeWasm) {
    let maze_wasm = unsafe { &mut *maze_wasm };
    maze_wasm.maze.definition.reset();
}
/// Represents an error 
#[repr(C)]
pub struct MazeWasmError {
    /// Message pointer (string)
    message_ptr: u32,
}
/// Frees a `MazeWasmError`
///
/// # Returns
///
/// Nothing
///
#[no_mangle]
pub extern "C" fn free_maze_wasm_error(error_ptr: u32) {
    let error_ptr = error_ptr as *mut MazeWasmError;
    if !error_ptr.is_null() {
        unsafe {
            if (*error_ptr).message_ptr != 0 {
                free_sized_memory((*error_ptr).message_ptr as *mut u8);
                (*error_ptr).message_ptr = 0;
            }
            let _ = Box::from_raw(error_ptr);
            decrement_num_objects_allocated();
        }
    }
}
/// Inserts rows into a `MazeWasm`
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_insert_rows(
    maze_wasm: *mut MazeWasm,
    start_row: u32,
    count: u32,
) -> u32 {
    let mut error_ptr: u32 = 0;
    let maze_wasm = unsafe { &mut *maze_wasm };
    if let Err(error) = maze_wasm
        .maze
        .definition
        .insert_rows(start_row as usize, count as usize)
    {
        error_ptr = create_maze_wasm_error_ptr(error.to_string().as_str());
    }
    error_ptr
}
/// Deletes rows from a `MazeWasm`
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_delete_rows(
    maze_wasm: *mut MazeWasm,
    start_row: u32,
    count: u32,
) -> u32 {
    let mut error_ptr: u32 = 0;
    let maze_wasm = unsafe { &mut *maze_wasm };
    if let Err(error) = maze_wasm
        .maze
        .definition
        .delete_rows(start_row as usize, count as usize)
    {
        error_ptr = create_maze_wasm_error_ptr(error.to_string().as_str());
    }
    error_ptr
}
/// Inserts columns into a `MazeWasm`
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub fn maze_wasm_insert_cols(maze_wasm: *mut MazeWasm, start_col: u32, count: u32) -> u32 {
    let mut error_ptr: u32 = 0;
    let maze_wasm = unsafe { &mut *maze_wasm };
    if let Err(error) = maze_wasm
        .maze
        .definition
        .insert_cols(start_col as usize, count as usize)
    {
        error_ptr = create_maze_wasm_error_ptr(error.to_string().as_str());
    }
    error_ptr
}
/// Deletes columns from a `MazeWasm`
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub fn maze_wasm_delete_cols(maze_wasm: *mut MazeWasm, start_col: u32, count: u32) -> u32 {
    let mut error_ptr: u32 = 0;
    let maze_wasm = unsafe { &mut *maze_wasm };
    if let Err(error) = maze_wasm
        .maze
        .definition
        .delete_cols(start_col as usize, count as usize)
    {
        error_ptr = create_maze_wasm_error_ptr(error.to_string().as_str());
    }
    error_ptr
}
/// Tests if a `MazeWasm` is empty
///
/// # Returns
///
/// Boolean
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub fn maze_wasm_is_empty(maze_wasm: *mut MazeWasm) -> bool {
    let maze_wasm = unsafe { &mut *maze_wasm };
    maze_wasm.maze.definition.is_empty()
}
/// Populates a `MazeWasm` from a JSON string
///
/// # Returns
///
/// Zero if successful, else an error pointer
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub fn maze_wasm_from_json(maze_wasm: *mut MazeWasm, json_string_ptr: *mut u8) -> u32 {
    let mut error_ptr: u32 = 0;
    let maze_wasm = unsafe { &mut *maze_wasm };
    let json_str = ptr_to_string(json_string_ptr);

    if let Err(error) = maze_wasm.maze.from_json(&json_str) {
        error_ptr = create_maze_wasm_error_ptr(error.to_string().as_str());
    }
    error_ptr
}

/// Web assembly enum for a result value type
#[cfg_attr(not(feature = "wasm-bindgen"), repr(C))]
pub enum MazeWasmResultValueType {
    None = 0,
    String = 1,
    Enum = 2,
    Point = 3,
    Solution = 4,
}
/// Converts an integer to a `MazeWasmResultValueType` 
///
/// # Returns
///
/// `MazeWasmResultValueType`
///
fn to_maze_result_value_type(value_type: u8) -> MazeWasmResultValueType {
    match value_type {
        1 => MazeWasmResultValueType::String,
        2 => MazeWasmResultValueType::Enum,
        3 => MazeWasmResultValueType::Point,
        4 => MazeWasmResultValueType::Solution,
        _ => MazeWasmResultValueType::None,
    }
}

/// Web assembly enum for a result value type
#[repr(C)]
pub struct MazeWasmResult {
    value_type: u8,
    value_ptr: u32,
    error_ptr: u32,
}
/// Creates a `MazeWasmResult` containing an error 
///
/// # Returns
///
/// `MazeWasmResult` with the `error_ptr` set to the allocated error string pointer
///
fn to_maze_wasm_result_ptr(value_type: u8, value_ptr: u32, error: Option<&str>) -> u32 {
    let mut error_ptr: u32 = 0;
    if let Some(error_str) = error {
        error_ptr = create_maze_wasm_error_ptr(error_str);
    }
    let result = MazeWasmResult {
        value_type,
        value_ptr,
        error_ptr,
    };
    let boxed_result = Box::new(result);
    increment_num_objects_allocated();
    Box::into_raw(boxed_result) as u32
}
/// Creates a `MazeWasmResult` containing a string value 
///
/// # Returns
///
/// `MazeWasmResult` with the `value_ptr` set to the allocated string pointer
///
fn create_maze_wasm_string_result(value: &str) -> u32 {
    to_maze_wasm_result_ptr(
        MazeWasmResultValueType::String as u8,
        to_string_ptr(value),
        None,
    )
}
/// Creates a `MazeWasmResult` containing an enumerated value 
///
/// # Returns
///
/// `MazeWasmResult` with the `value_ptr` set to the enumated value
///
fn create_maze_wasm_enum_result(value: u32) -> u32 {
    to_maze_wasm_result_ptr(MazeWasmResultValueType::Enum as u8, value, None)
}
/// Creates a `MazeWasmResult` containing a point value 
///
/// # Returns
///
/// `MazeWasmResult` with the `value_ptr` set to the allocated point value
///
fn create_maze_wasm_point_result(point: &MazePoint) -> u32 {
    to_maze_wasm_result_ptr(
        MazeWasmResultValueType::Point as u8,
        to_point_ptr(point),
        None,
    )
}
/// Creates a `MazeWasmResult` containing a solution value 
///
/// # Returns
///
/// `MazeWasmResult` with the `value_ptr` set to the solution pointer
///
fn create_maze_wasm_solution_result(solution: MazeSolution) -> u32 {
    to_maze_wasm_result_ptr(
        MazeWasmResultValueType::Solution as u8,
        to_solution_ptr(solution),
        None,
    )
}
/// Creates a `MazeWasmResult` containing an error value 
///
/// # Returns
///
/// `MazeWasmResult` with the `error_ptr` set to the allocated error
///
fn create_maze_wasm_error_result(error: Option<&str>) -> u32 {
    to_maze_wasm_result_ptr(MazeWasmResultValueType::None as u8, 0, error)
}
/// Frees a `MazeWasmResult` 
///
/// # Returns
///
/// Nothing
///
#[no_mangle]
pub extern "C" fn free_maze_wasm_result(result_ptr: u32, free_value_ptr: bool) {
    let result_ptr = result_ptr as *mut MazeWasmResult;
    if !result_ptr.is_null() {
        unsafe {
            if free_value_ptr && (*result_ptr).value_ptr != 0 {
                match to_maze_result_value_type((*result_ptr).value_type) {
                    MazeWasmResultValueType::String | MazeWasmResultValueType::Point => {
                        free_sized_memory((*result_ptr).value_ptr as *mut u8);
                    }
                    MazeWasmResultValueType::Solution => {
                        free_maze_wasm_solution((*result_ptr).value_ptr as *mut MazeSolution);
                    }
                    _ => {}
                }
                (*result_ptr).value_ptr = 0;
            }

            if (*result_ptr).error_ptr != 0 {
                free_maze_wasm_error((*result_ptr).error_ptr);
                (*result_ptr).error_ptr = 0;
            }

            let _ = Box::from_raw(result_ptr);
            decrement_num_objects_allocated();
        }
    }
}
/// Converts a `MazeWasm` to a JSON string  value 
///
/// # Returns
///
/// `MazeWasmResult` with the `value_ptr` set to the JSON string pointer,
/// else the `error_ptr` set to non-zero if the function fails
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_to_json(maze_wasm: *mut MazeWasm) -> u32 {
    let maze_wasm = unsafe { &mut *maze_wasm };

    match maze_wasm.maze.to_json() {
        Err(error) => create_maze_wasm_error_result(Some(error.to_string().as_str())),
        Ok(json_str) => create_maze_wasm_string_result(json_str.as_str()),
    }
}
/// Solves a `MazeWasm` for its solution path
///
/// # Returns
///
/// `MazeWasmResult` with the `value_ptr` set to the solution pointer,
/// else the `error_ptr` set to non-zero if the function fails
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_solve(maze_wasm: *mut MazeWasm) -> u32 {
    let maze_wasm = unsafe { &mut *maze_wasm };

    match maze_wasm.maze.solve() {
        Err(error) => create_maze_wasm_error_result(Some(error.to_string().as_str())),
        Ok(solution) => create_maze_wasm_solution_result(solution),
    }
}
/// Returns the path associated with a solution
///
/// # Returns
///
/// Pointer to an array of points in a contiguous block of memory, laid out as follows:
/// Total Memory (u32) = 4 bytes
/// num_points (u32)  = 4 bytes
/// num_points * 8 bytes containing:
///     row (u32)     = 4 bytes
///     columns (u32) = 4 bytes
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_solution_get_path_points(solution: *mut MazeSolution) -> u32 {
    let solution = unsafe { &mut *solution };
    let num_points = solution.path.points.len();
    let length = 4 + num_points * 8;
    let mem_ptr = allocate_sized_memory(length);
    unsafe {
        let mut points_data_ptr = mem_ptr.add(4);
        ptr::write(points_data_ptr as *mut u32, num_points as u32);
        points_data_ptr = points_data_ptr.add(4);
        for point in &solution.path.points {
            points_data_ptr = write_point(points_data_ptr, point);
        }
        mem_ptr as u32
    }
}
/// Allocates a `MazeWasmError`
///
/// # Returns
///
/// Pointer to `MazeWasmError`
fn create_maze_wasm_error_ptr(error_str: &str) -> u32 {
    let error = MazeWasmError {
        message_ptr: to_string_ptr(error_str),
    };
    let boxed_error = Box::new(error);
    increment_num_objects_allocated();
    Box::into_raw(boxed_error) as u32
}
/// Converts a string to a string pointer in sized memory
///
/// # Returns
///
/// Pointer to string
fn to_string_ptr(str: &str) -> u32 {
    let length = str.len();
    let string_ptr = allocate_sized_memory(length);
    unsafe {
        let string_data_ptr = string_ptr.add(4);
        ptr::copy_nonoverlapping(str.as_ptr(), string_data_ptr, length);
    }
    string_ptr as u32
}
/// Converts a point to a point pointer in sized memory
///
/// # Returns
///
/// Pointer to point
fn to_point_ptr(point: &MazePoint) -> u32 {
    let point_ptr = allocate_sized_memory(8);
    unsafe {
        let point_data_ptr = point_ptr.add(4);
        let _ = write_point(point_data_ptr, point);
    }
    point_ptr as u32
}
/// Writes a point to a memory pointer
///
/// # Returns
///
/// Pointer to memory directly after the written point
fn write_point(ptr: *mut u8, point: &MazePoint) -> *mut u8 {
    unsafe {
        ptr::write(ptr as *mut u32, point.row as u32);
        let ptr = ptr.add(4);
        ptr::write(ptr as *mut u32, point.col as u32);
        ptr.add(4)
    }
}
/// Converts a solution to a boxed pointer
///
/// # Returns
///
/// Pointer to boxed solution pointer
fn to_solution_ptr(solution: MazeSolution) -> u32 {
    let boxed_solution = Box::new(solution);
    increment_num_objects_allocated();
    Box::into_raw(boxed_solution) as u32
}
/// Frees a maze solution pointer
///
/// # Returns
///
/// Nothing
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn free_maze_wasm_solution(solution_ptr: *mut MazeSolution) {
    if !solution_ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(solution_ptr);
            decrement_num_objects_allocated();
        }
    }
}
/// Reads the length contained at the start of a given memory pointer 
///
/// # Returns
///
/// Length
///
fn ptr_length(ptr: *const u8) -> usize {
    let mut length: usize = 0;
    if !ptr.is_null() {
        // Read the length of the string from the first 4 bytes (u32)
        length = unsafe { ptr::read(ptr as *const u32) as usize };
    }
    length
}
/// Reads a string from a string memory pointer 
///
/// # Returns
///
/// String
///
fn ptr_to_string(ptr: *const u8) -> String {
    // Read the length of the string from the first 4 bytes (u32)
    let length = ptr_length(ptr);

    // Convert the pointer to the string data (after the first 4 bytes)
    let string_slice = unsafe {
        let data_ptr = ptr.add(4); // Skip the first 4 bytes (u32 for length)
        let slice = std::slice::from_raw_parts(data_ptr, length);
        std::str::from_utf8(slice).unwrap()
    };
    string_slice.to_string()
}

/// Options controlling maze generation.
///
/// Created via `new_generator_options_wasm`, mutated via setter functions,
/// passed to `maze_wasm_generate`, freed via `free_generator_options_wasm`.
///
/// Sentinel values for optional fields:
/// - start/finish row/col: `u32::MAX` = use default
/// - min_spine_length:     `0`        = use default ((row_count + col_count) / 2)
/// - max_retries:          `0`        = use default (100)
/// - branch_from_finish:   `0` = false (default), `1` = true
#[cfg(feature = "wasm-lite")]
#[repr(C)]
pub struct GeneratorOptionsWasm {
    pub row_count:          u32,
    pub col_count:          u32,
    pub algorithm:          GenerationAlgorithmWasm,
    pub seed:               u64,
    pub start_row:          u32,
    pub start_col:          u32,
    pub finish_row:         u32,
    pub finish_col:         u32,
    pub min_spine_length:   u32,
    pub max_retries:        u32,
    pub branch_from_finish: u8,
}
/// Creates a new `GeneratorOptionsWasm` with the given required fields and default optional fields.
///
/// # Returns
///
/// Pointer to `GeneratorOptionsWasm`
#[cfg(feature = "wasm-lite")]
#[no_mangle]
pub extern "C" fn new_generator_options_wasm(
    row_count: u32,
    col_count: u32,
    algorithm: GenerationAlgorithmWasm,
    seed: u64,
) -> *mut GeneratorOptionsWasm {
    let opts = Box::new(GeneratorOptionsWasm {
        row_count,
        col_count,
        algorithm,
        seed,
        start_row:          u32::MAX,
        start_col:          u32::MAX,
        finish_row:         u32::MAX,
        finish_col:         u32::MAX,
        min_spine_length:   0,
        max_retries:        0,
        branch_from_finish: 0,
    });
    increment_num_objects_allocated();
    Box::into_raw(opts)
}
/// Sets the start cell in a `GeneratorOptionsWasm`
#[cfg(feature = "wasm-lite")]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn generator_options_set_start(ptr: *mut GeneratorOptionsWasm, row: u32, col: u32) {
    let opts = unsafe { &mut *ptr };
    opts.start_row = row;
    opts.start_col = col;
}
/// Sets the finish cell in a `GeneratorOptionsWasm`
#[cfg(feature = "wasm-lite")]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn generator_options_set_finish(ptr: *mut GeneratorOptionsWasm, row: u32, col: u32) {
    let opts = unsafe { &mut *ptr };
    opts.finish_row = row;
    opts.finish_col = col;
}
/// Sets the minimum spine length in a `GeneratorOptionsWasm`
#[cfg(feature = "wasm-lite")]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn generator_options_set_min_spine_length(ptr: *mut GeneratorOptionsWasm, value: u32) {
    let opts = unsafe { &mut *ptr };
    opts.min_spine_length = value;
}
/// Sets the maximum retries in a `GeneratorOptionsWasm`
#[cfg(feature = "wasm-lite")]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn generator_options_set_max_retries(ptr: *mut GeneratorOptionsWasm, value: u32) {
    let opts = unsafe { &mut *ptr };
    opts.max_retries = value;
}
/// Sets the branch_from_finish flag in a `GeneratorOptionsWasm` (`0` = false, `1` = true)
#[cfg(feature = "wasm-lite")]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn generator_options_set_branch_from_finish(ptr: *mut GeneratorOptionsWasm, value: u8) {
    let opts = unsafe { &mut *ptr };
    opts.branch_from_finish = value;
}
/// Generates a maze, populating the given `MazeWasm`.
///
/// # Returns
///
/// Zero if successful, else an error pointer
#[cfg(feature = "wasm-lite")]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn maze_wasm_generate(
    maze_wasm: *mut MazeWasm,
    options: *mut GeneratorOptionsWasm,
) -> u32 {
    let maze_wasm = unsafe { &mut *maze_wasm };
    let opts = unsafe { &*options };

    let start = if opts.start_row == u32::MAX || opts.start_col == u32::MAX {
        None
    } else {
        Some(data_model::MazePoint { row: opts.start_row as usize, col: opts.start_col as usize })
    };
    let finish = if opts.finish_row == u32::MAX || opts.finish_col == u32::MAX {
        None
    } else {
        Some(data_model::MazePoint { row: opts.finish_row as usize, col: opts.finish_col as usize })
    };
    let min_spine_length = if opts.min_spine_length == 0 { None } else { Some(opts.min_spine_length as usize) };
    let max_retries = if opts.max_retries == 0 { None } else { Some(opts.max_retries as usize) };
    let branch_from_finish = if opts.branch_from_finish == 0 { Some(false) } else { Some(true) };

    let generator_options = GeneratorOptions {
        row_count: opts.row_count as usize,
        col_count: opts.col_count as usize,
        algorithm: to_generation_algorithm(opts.algorithm),
        start,
        finish,
        min_spine_length,
        max_retries,
        branch_from_finish,
        seed: Some(opts.seed),
    };

    let generator = Generator { options: generator_options };
    match generator.generate() {
        Ok(maze) => {
            maze_wasm.maze = maze;
            0
        }
        Err(e) => create_maze_wasm_error_ptr(e.to_string().as_str()),
    }
}
/// Frees a `GeneratorOptionsWasm` pointer
#[cfg(feature = "wasm-lite")]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn free_generator_options_wasm(ptr: *mut GeneratorOptionsWasm) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
            decrement_num_objects_allocated();
        }
    }
}

/// Total sized memory currently allocated
static mut TOTAL_SIZED_MEM_USED: i64 = 0;

/// Gets the total amount of sized memory currently allocated 
///
/// # Returns
///
/// Number of bytes
///
#[no_mangle]
pub extern "C" fn get_sized_memory_used() -> i64 {
    unsafe { TOTAL_SIZED_MEM_USED }
}
/// Allocates sized memory of a given size 
///
/// # Returns
///
/// Pointer to allocated memory
///
#[no_mangle]
pub extern "C" fn allocate_sized_memory(size: usize) -> *mut u8 {
    // Allocate enough memory for the length (u32) + string data
    let total_size = size + 4;
    let layout = Layout::from_size_align(total_size, 1).unwrap();
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        TOTAL_SIZED_MEM_USED += total_size as i64;
    }
    unsafe {
        ptr::write(ptr as *mut u32, size as u32);
    }
    ptr
}
/// Frees sized memory associated with a given pointer 
///
/// # Returns
///
/// Nothing
///
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn free_sized_memory(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe {
            let size = ptr::read(ptr as *const u32) as usize;
            let total_size = size + 4;
            let layout = Layout::from_size_align(total_size, 1).unwrap();
            dealloc(ptr, layout);
            TOTAL_SIZED_MEM_USED -= total_size as i64;
        }
    }
}

/// Total number of objects currently allocated
static mut TOTAL_NUM_OBJECTS_ALLOCATED: i64 = 0;

/// Gets the total amount of objects currently allocated 
///
/// # Returns
///
/// Number of objects
///
#[no_mangle]
pub extern "C" fn get_num_objects_allocated() -> i64 {
    unsafe { TOTAL_NUM_OBJECTS_ALLOCATED }
}
/// Increments the total amount of objects currently allocated by 1 
///
/// # Returns
///
/// Nothing
///
fn increment_num_objects_allocated() {
    unsafe {
        TOTAL_NUM_OBJECTS_ALLOCATED += 1;
    }
}
/// Decrements the total amount of objects currently allocated by 1 
///
/// # Returns
///
/// Nothing
///
fn decrement_num_objects_allocated() {
    unsafe {
        TOTAL_NUM_OBJECTS_ALLOCATED -= 1;
    }
}
