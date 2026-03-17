#[allow(unused_imports)] // MazeCellTypeWasm is referenced in doc comments as an intra-doc link
use crate::wasm_common::{
    new_maze, to_cell_type_enum, to_generation_algorithm, GenerationAlgorithmWasm, MazeCellTypeWasm,
    MazeWasm,
};
use data_model::MazePoint;
use js_sys::{Array, Object, Reflect};
use maze::{Generator, GeneratorOptions, MazeSolution, MazeSolver};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

/// Converts a Rust Point to a JavaScript object
fn to_js_point_obj(point: &MazePoint) -> Object {
    let obj = Object::new();
    Reflect::set(
        &obj,
        &JsValue::from_str("row"),
        &JsValue::from_f64(point.row as f64),
    )
    .unwrap();
    Reflect::set(
        &obj,
        &JsValue::from_str("col"),
        &JsValue::from_f64(point.col as f64),
    )
    .unwrap();
    obj
}

/// Converts a cell type to a JavaScript object
fn to_js_cell_info_obj(cell_type: char) -> Object {
    let obj = Object::new();
    Reflect::set(
        &obj,
        &JsValue::from_str("cell_type"),
        &JsValue::from(to_cell_type_enum(cell_type) as u32),
    )
    .unwrap();
    obj
}

#[wasm_bindgen]
/// Web assembly representation of a maze solution
pub struct MazeSolutionWasm {
    //#[cfg_attr(feature = "wasm-bindgen", wasm_bindgen(skip))] - does not work
    #[wasm_bindgen(skip)]
    pub solution: MazeSolution,
}


#[wasm_bindgen]
impl MazeSolutionWasm {
    /// Returns the array of points (if any) associated with the maze solution
    ///
    /// # Returns
    ///
    /// This function will return an array of Javascript objects defining each point in
    /// the solution. Each solution point object has the folllowing properties:
    ///
    /// - `row` - zero-based row index for the solution point
    /// - `col` - zero-based column index for the solution point
    ///
    /// # Examples
    ///
    /// Initialize a maze from a JSON string, then attempt to solve it and, if successful,
    /// print the maze solution path's points
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.from_json(`{
    ///             \"id\":\"maze_id\",
    ///             \"name\":\"test\",
    ///             \"definition\": {
    ///                 \"grid\":[
    ///                     [\"S\", \"W\", \" \", \" \", \"W\"],
    ///                     [\" \", \"W\", \" \", \"W\", \" \"],
    ///                     [\" \", \" \", \" \", \"W\", \"F\"],
    ///                     [\"W\", \" \", \"W\", \" \", \" \"],
    ///                     [\" \", \" \", \" \", \"W\", \" \"],
    ///                     [\"W\", \"W\", \" \", \" \", \" \"],
    ///                     [\"W\", \"W\", \" \", \"W\", \" \"]
    ///                 ]
    ///         }}`);
    ///         let solution = maze.solve();
    ///         let solutionPoints = solution.get_path_points();
    ///         console.log("Successfully solved maze. Solution points are: ", solutionPoints);
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn get_path_points(&self) -> Array {
        let path_points = Array::new();
        for point in &self.solution.path.points {
            path_points.push(&to_js_point_obj(point));
        }
        path_points
    }
}

#[cfg_attr(feature = "wasm-bindgen", wasm_bindgen)]
impl MazeWasm {
    #[wasm_bindgen(constructor)]
    /// Creates a new maze instance
    ///
    /// # Returns
    ///
    /// A new maze instance
    ///
    /// # Examples
    ///
    /// Create a new maze and print its dimensions (which will be 0 rows x 0 columns)
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     import init, { MazeWasm } from 'maze_wasm.js';
    ///         ///     try {
    ///         let maze = new MazeWasm();
    ///         console.log("Successfully created maze. Dimensions: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn new() -> Result<MazeWasm, JsValue> {
        Ok(MazeWasm { maze: new_maze() })
    }
    #[wasm_bindgen]
    /// Resets the maze instance to empty
    ///
    /// # Returns
    ///
    /// The empty maze instance
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and print out its dimensions.
    /// Then, reset it and print out its dimensions again (which will now be 0 rows x 0 columns).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.reset();
    ///         console.log("After reset(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn reset(&mut self) -> MazeWasm {
        self.maze.reset();
        self.clone()
    }
    #[wasm_bindgen]
    /// Resizes the maze instance
    ///
    /// # Arguments
    /// * `new_row_count` - New number of rows
    /// * `new_col_count` - New number of columns
    ///
    /// # Returns
    ///
    /// This function will return an error if the maze could not be resized
    ///
    /// # Examples
    ///
    /// Create a new maze, print its dimensions, resize it to 10 rows x 5 columns and
    /// then print out its dimensions again.
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm ();
    ///         console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s   )");
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    pub fn resize(
        &mut self,
        new_row_count: JsValue,
        new_col_count: JsValue,
    ) -> Result<(), JsValue> {
        let new_row_count = Self::arg_to_usize("new_row_count", new_row_count)?;
        let new_col_count = Self::arg_to_usize("new_col_count", new_col_count)?;
        self.maze.definition.resize(new_row_count, new_col_count);
        Ok(())
    }
    #[wasm_bindgen]
    /// Inserts one or more empty rows into the maze instance
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `count` - Number of rows to insert
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target rows are out of range
    ///
    ///  # Examples
    ///
    /// Create a new maze, print its dimensions, insert 5 rows and
    /// then print out its dimensions again (which will now be 5 rows x 0 columns).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.insert_rows(0, 5);
    ///         console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn insert_rows(&mut self, start_row: JsValue, count: JsValue) -> Result<(), JsValue> {
        let start_row = Self::arg_to_usize("start_row", start_row)?;
        let count = Self::arg_to_usize("count", count)?;
        self.maze
            .definition
            .insert_rows(start_row, count)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Deletes one or more consecutive rows from the maze instance
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `count` - Number of rows to delete
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the definition is empty
    /// - If the target rows are out of range
    ///
    ///  # Examples
    ///
    /// Create a new maze, insert 5 rows and print out its dimensions.
    /// Then, delete rows 2 to 4 and print out the dimensions again (which will now be 2 rows x 0 columns).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.insert_rows(0, 5);
    ///         console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.delete_rows(1, 3);
    ///         console.log("After delete_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn delete_rows(&mut self, start_row: JsValue, count: JsValue) -> Result<(), JsValue> {
        let start_row = Self::arg_to_usize("start_row", start_row)?;
        let count = Self::arg_to_usize("count", count)?;
        self.maze
            .definition
            .delete_rows(start_row, count)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Inserts one or more empty columns into the maze instance
    ///
    /// # Arguments
    ///
    /// * `start_col` - Start column index (zero-based)
    /// * `count` - Number of columns to insert
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the definition is empty
    /// - If the target columns are out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, insert 1 row and print out its dimensions. Then, insert 10 colums
    /// and print out the dimensions again (which will now be 1 row x 10 columns).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.insert_rows(0, 1);
    ///         console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.insert_cols(0, 10);
    ///         console.log("After insert_cols(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn insert_cols(&mut self, start_col: JsValue, count: JsValue) -> Result<(), JsValue> {
        let start_col = Self::arg_to_usize("start_col", start_col)?;
        let count = Self::arg_to_usize("count", count)?;
        self.maze
            .definition
            .insert_cols(start_col, count)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Deletes one or more consecutive columns from the maze instance
    ///
    /// # Arguments
    ///
    /// * `start_col` - Start column index (zero-based)
    /// * `count` - Number of columns to delete
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the definition is empty
    /// - If the target columns are out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 column and print out its dimensions. Then, delete
    /// columns 2 to 4 and print out the dimensions again (which will now be 10 rows x 2 columns).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.delete_cols(1, 3);
    ///         console.log("After delete_cols(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn delete_cols(&mut self, start_col: JsValue, count: JsValue) -> Result<(), JsValue> {
        let start_col = Self::arg_to_usize("start_col", start_col)?;
        let count = Self::arg_to_usize("count", count)?;
        self.maze
            .definition
            .delete_cols(start_col, count)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Checks whether the maze instance is empty
    ///
    /// # Returns
    ///
    /// Boolean
    ///
    /// # Examples
    ///
    /// Create a new maze and print out whether it is empty (`true`). Then, resize it to
    /// 1 row x 2 columns and again print out whether it is empty (`false`).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         console.log("After creation, is_empty() = ", maze.is_empty());
    ///         maze.resize(1,2);
    ///         console.log("After resize(), is_empty() = ", maze.is_empty());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn is_empty(&self) -> bool {
        self.maze.definition.is_empty()
    }
    #[wasm_bindgen]
    /// Returns the number of rows associated with the maze instance
    ///
    /// # Returns
    ///
    /// Number of rows
    ///
    /// # Examples
    ///
    /// Create a new maze and print out the number rows (0). Then, resize it to
    /// 10 rows x 5 columns and then print out the number of rows again (10).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         console.log("After creation, get_row_count() = ", maze.get_row_count());
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), get_row_count() = ", maze.get_row_count());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn get_row_count(&self) -> usize {
        self.maze.definition.row_count()
    }
    #[wasm_bindgen]
    /// Returns the number of columns associated with the maze instance
    ///
    /// # Returns
    ///
    /// Number of columns
    ///
    /// # Examples
    ///
    /// Create a new maze and print out the number columns (0). Then, resize it to
    /// 10 rows x 5 columns and then print out the number of columns again (5).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         console.log("After creation, get_col_count() = ", maze.get_col_count());
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), get_col_count() = ", maze.get_col_count());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn get_col_count(&self) -> usize {
        self.maze.definition.col_count()
    }
    #[wasm_bindgen]
    /// Returns cell information for the given location within the maze instance
    ///
    /// # Arguments
    ///
    /// * `row` - Row index (zero-based)
    /// * `col` - Column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    ///
    /// If sucessful, a cell information object will be returned with the following properties:
    ///
    /// * `cell_type` - The type ([`MazeCellTypeWasm`]) associated with the cell
    ///
    /// # Examples
    ///
    /// Create a new maze and resize it to 10 rows x 5 columns. Then, print out the cell
    /// information for the cell at row = 1, column = 2.
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("get_cell(1, 2) = ", maze.get_cell(1, 2));
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn get_cell(&self, row: JsValue, col: JsValue) -> Result<Object, JsValue> {
        let row = Self::arg_to_usize("row", row)?;
        if row >= self.maze.definition.row_count() {
            return Err(JsValue::from_str("row out of bounds"));
        }
        let col = Self::arg_to_usize("col", col)?;
        if col >= self.maze.definition.col_count() {
            return Err(JsValue::from_str("column out of bounds"));
        }
        Ok(to_js_cell_info_obj(self.maze.definition.grid[row][col]))
    }
    #[wasm_bindgen]
    /// Sets the start cell location within the maze instance
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start cell row index (zero-based)
    /// * `start_col` - Start cell column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and print out the cell
    /// information for the cell at row = 1, column = 2 (`cell_type` will be [`MazeCellTypeWasm::Empty`]).
    /// Then, set the start cell to that same location and print out the cell information again
    /// (`cell_type` will now be [`MazeCellTypeWasm::Start`]).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("Before set_start_cell(), get_cell(1, 2) = ", maze.get_cell(1, 2));
    ///         maze.set_start_cell(1, 2);
    ///         console.log("After set_start_cell(), get_cell(1, 2) = ", maze.get_cell(1, 2));
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn set_start_cell(
        &mut self,
        start_row: JsValue,
        start_col: JsValue,
    ) -> Result<(), JsValue> {
        let row = Self::arg_to_usize("start_row", start_row)?;
        let col = Self::arg_to_usize("start_col", start_col)?;
        self.maze
            .definition
            .set_start(Some(MazePoint { row, col }))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Returns the start cell associated with the maze instance (if any)
    ///
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the start cell does not exist
    ///
    /// If sucessful, an object will be returned with the following properties:
    ///
    /// * `row` - Start cell row index (zero-based)
    /// * `col` - Start cell column index (zero-based)
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and set the
    /// start cell to be at row = 1, column = 2. Then, retreive and print
    /// out of the details for the start cell.
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_start_cell(1, 2);
    ///         console.log("get_start_cell() = ", maze.get_start_cell());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn get_start_cell(&mut self) -> Result<Object, JsValue> {
        if let Some(start) = self.maze.definition.get_start() {
            return Ok(to_js_point_obj(&start));
        }
        Err(JsValue::from_str("no start cell defined"))
    }
    #[wasm_bindgen]
    /// Sets the finish cell location within the maze instance
    ///
    /// # Arguments
    ///
    /// * `finish_row` - Finish cell row index (zero-based)
    /// * `finish_col` - Finish cell column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and print out the cell
    /// information for the cell at row = 3, column = 4 (`cell_type` will be [`MazeCellTypeWasm::Empty`]).
    /// Then, set the finish cell to that same location and print out the cell information again
    /// (`cell_type` will now be [`MazeCellTypeWasm::Finish`]).
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("Before set_finish_cell(), get_cell(3, 4) = ", maze.get_cell(3, 4));
    ///         maze.set_finish_cell(3, 4);
    ///         console.log("After set_finish_cell(), get_cell(3, 4) = ", maze.get_cell(3, 4));
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn set_finish_cell(
        &mut self,
        finish_row: JsValue,
        finish_col: JsValue,
    ) -> Result<(), JsValue> {
        let row = Self::arg_to_usize("finish_row", finish_row)?;
        let col = Self::arg_to_usize("finish_col", finish_col)?;
        self.maze
            .definition
            .set_finish(Some(MazePoint { row, col }))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Returns the finish cell associated with the maze instance (if any)
    ///
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the finish cell does not exist
    ///
    /// If sucessful, an object will be returned with the following properties:
    ///
    /// * `row` - Finish cell row index (zero-based)
    /// * `col` - Finish cell column index (zero-based)
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and set the
    /// finish cell to be at row = 9, column = 4. Then, retreive and print
    /// out of the details for the finish cell.
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_finish_cell(9, 4);
    ///         console.log("get_finish_cell() = ", maze.get_finish_cell());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn get_finish_cell(&mut self) -> Result<Object, JsValue> {
        if let Some(finish) = self.maze.definition.get_finish() {
            return Ok(to_js_point_obj(&finish));
        }
        Err(JsValue::from_str("no finish cell defined"))
    }
    #[wasm_bindgen]
    /// Sets a range of cells within the maze instance to be walls (`cell_type` = [`MazeCellTypeWasm::Wall`])
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `start_col` - Start column index (zero-based)
    /// * `finish_row` - Finish row index (zero-based)
    /// * `finish_col` - Finish column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and then set
    /// cells 2 to 4 to be walls in the first row. Then print the cell
    /// information for the top row, which will have cells (0, 0) and (0, 4)
    /// as  [`MazeCellTypeWasm::Empty`] and cells (0, 1) to (0, 3) as
    /// [`MazeCellTypeWasm::Wall`].
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_wall_cells(0, 1, 0, 3);
    ///         for (let col  = 0; col < 5; col ++) {
    ///             console.log(`After set_walls_cell(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
    ///         }
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn set_wall_cells(
        &mut self,
        start_row: JsValue,
        start_col: JsValue,
        end_row: JsValue,
        end_col: JsValue,
    ) -> Result<(), JsValue> {
        self.set_cell_values(start_row, start_col, end_row, end_col, 'W')?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Clears a range of cells within the maze instance, setting their `cell_type` = [`MazeCellTypeWasm::Empty`]
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `start_col` - Start column index (zero-based)
    /// * `finish_row` - Finish row index (zero-based)
    /// * `finish_col` - Finish column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and then set
    /// cells 2 to 4 to be walls in the first row. Then print the `cell_type`
    /// for the top row, which will have cells (0, 0) and (0, 4)
    /// as [`MazeCellTypeWasm::Empty`] and cells (0, 1) to (0, 3) as
    /// [`MazeCellTypeWasm::Wall`]. Finally, clear cells (0, 2) to (3, 4) and
    /// reprint the `cell_type` for the top row, which will now have
    /// just once cell (0, 1) as [`MazeCellTypeWasm::Wall`].
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_wall_cells(0, 1, 0, 3);
    ///         for (let col  = 0; col < 5; col ++) {
    ///             console.log(`After set_walls_cell(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
    ///         }
    ///         maze.clear_cells(0, 2, 3, 4);
    ///         for (let col  = 0; col < 5; col ++) {
    ///             console.log(`After clear_walls(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
    ///         }
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn clear_cells(
        &mut self,
        start_row: JsValue,
        start_col: JsValue,
        end_row: JsValue,
        end_col: JsValue,
    ) -> Result<(), JsValue> {
        self.set_cell_values(start_row, start_col, end_row, end_col, ' ')?;
        Ok(())
    }
    #[wasm_bindgen]
    /// This function will return the JSON string representation for the maze instance
    ///
    /// # Returns
    ///
    /// JSON string representing the maze, or an error if the JSON could not be generated
    ///
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 6 rows x 5 columns and then set
    /// cells 2 to 4 to be walls in the first 3 rows. The conver to JSON
    /// and print the result.
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.resize(6, 5);
    ///         maze.set_wall_cells(0, 1, 2, 4);
    ///         let json = maze.to_json();
    ///         console.log("to_json() returned: ", json);
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn to_json(&self) -> Result<String, JsValue> {
        self.maze
            .to_json()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    #[wasm_bindgen]
    /// Initializes the maze instance by reading the JSON string content provided
    ///
    /// # Returns
    ///
    /// This function will return an error if the JSON could not be read
    ///
    /// # Examples
    ///
    /// Create a new maze and initialise it from a JSON string. Then, print
    /// the `cell_type` for each cell.
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.from_json(`{
    ///             \"id\":\"maze_id\",
    ///             \"name\":\"test\",
    ///             \"definition\": {
    ///                 \"grid\":[
    ///                     [\"S\", \"W\", \" \", \" \", \"W\"],
    ///                     [\" \", \"W\", \" \", \"W\", \" \"],
    ///                     [\" \", \" \", \" \", \"W\", \"F\"],
    ///                     [\"W\", \" \", \"W\", \" \", \" \"],
    ///                     [\" \", \" \", \" \", \"W\", \" \"],
    ///                     [\"W\", \"W\", \" \", \" \", \" \"],
    ///                     [\"W\", \"W\", \" \", \"W\", \" \"]
    ///                 ]
    ///         }}`);
    ///         for (let row  = 0; row < maze.get_row_count(); row ++) {
    ///             for (let col  = 0; col < maze.get_col_count(); col ++) {
    ///                 console.log(`After from_json(), cell_type at (${row}, ${col}) = `, maze.get_cell(row, col).cell_type);
    ///             }
    ///         }
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn from_json(&mut self, json_string: JsValue) -> Result<(), JsValue> {
        let json_str = Self::arg_to_string("json_string", json_string)?;
        self.maze
            .from_json(&json_str)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Attempts to solve the path between the start and end points defined within the maze instance
    ///
    /// # Returns
    ///
    /// A maze solution ([`MazeSolutionWasm`]) if successful, else an error if the maze could not be solved
    ///
    ///
    /// # Examples
    ///
    /// Initialize a maze from a JSON string, then attempt to solve it and, if successful,
    /// print the maze solution path's points
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     try {
    ///         let maze = new MazeWasm();
    ///         maze.from_json(`{
    ///             \"id\":\"maze_id\",
    ///             \"name\":\"test\",
    ///             \"definition\": {
    ///                 \"grid\":[
    ///                     [\"S\", \"W\", \" \", \" \", \"W\"],
    ///                     [\" \", \"W\", \" \", \"W\", \" \"],
    ///                     [\" \", \" \", \" \", \"W\", \"F\"],
    ///                     [\"W\", \" \", \"W\", \" \", \" \"],
    ///                     [\" \", \" \", \" \", \"W\", \" \"],
    ///                     [\"W\", \"W\", \" \", \" \", \" \"],
    ///                     [\"W\", \"W\", \" \", \"W\", \" \"]
    ///                 ]
    ///         }}`);
    ///         let solution = maze.solve();
    ///         let solutionPoints = solution.get_path_points();
    ///         console.log("Maze solve() succeeded. Solution points are: ", solutionPoints);
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     }
    /// }
    /// run();
    /// ```
    pub fn solve(&self) -> Result<MazeSolutionWasm, JsValue> {
        let solution = self
            .maze
            .solve()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(MazeSolutionWasm { solution })
    }

    #[wasm_bindgen]
    /// Generates a new maze and replaces the current maze instance with it
    ///
    /// # Arguments
    ///
    /// * `row_count` - Number of rows (must be >= 3)
    /// * `col_count` - Number of columns (must be >= 3)
    /// * `algorithm` - Generation algorithm to use ([`GenerationAlgorithmWasm`])
    /// * `start_row` - Start cell row index (undefined = default 0)
    /// * `start_col` - Start cell column index (undefined = default 0)
    /// * `finish_row` - Finish cell row index (undefined = default row_count-1)
    /// * `finish_col` - Finish cell column index (undefined = default col_count-1)
    /// * `min_spine_length` - Minimum solution path length (undefined = default (row_count+col_count)/2)
    /// * `max_retries` - Maximum generation attempts (undefined = default 100)
    /// * `branch_from_finish` - Whether to branch from the finish cell (undefined = default false)
    ///
    /// # Returns
    ///
    /// This function will return an error if the maze could not be generated
    pub fn generate(
        &mut self,
        row_count: JsValue,
        col_count: JsValue,
        algorithm: GenerationAlgorithmWasm,
        start_row: JsValue,
        start_col: JsValue,
        finish_row: JsValue,
        finish_col: JsValue,
        min_spine_length: JsValue,
        max_retries: JsValue,
        branch_from_finish: JsValue,
    ) -> Result<(), JsValue> {
        let row_count = Self::arg_to_usize("row_count", row_count)?;
        let col_count = Self::arg_to_usize("col_count", col_count)?;

        let start_row = Self::opt_arg_to_usize("start_row", start_row)?;
        let start_col = Self::opt_arg_to_usize("start_col", start_col)?;
        let finish_row = Self::opt_arg_to_usize("finish_row", finish_row)?;
        let finish_col = Self::opt_arg_to_usize("finish_col", finish_col)?;

        let start = match (start_row, start_col) {
            (Some(r), Some(c)) => Some(MazePoint { row: r, col: c }),
            _ => None,
        };
        let finish = match (finish_row, finish_col) {
            (Some(r), Some(c)) => Some(MazePoint { row: r, col: c }),
            _ => None,
        };

        let min_spine_length = Self::opt_arg_to_usize("min_spine_length", min_spine_length)?;
        let max_retries = Self::opt_arg_to_usize("max_retries", max_retries)?;

        let branch_from_finish = if branch_from_finish.is_null() || branch_from_finish.is_undefined() {
            None
        } else {
            branch_from_finish.as_bool()
        };

        let options = GeneratorOptions {
            row_count,
            col_count,
            algorithm: to_generation_algorithm(algorithm),
            start,
            finish,
            min_spine_length,
            max_retries,
            branch_from_finish,
        };

        let maze = Generator { options }
            .generate()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.maze = maze;
        Ok(())
    }

    // Private helper functions and methods
    fn js_value_type_str(val: JsValue) -> String {
        if val.is_string() {
            "string".to_string()
        } else if val.as_f64().is_some() {
            "number".to_string()
        } else if val.as_bool().is_some() {
            "boolean".to_string()
        } else if val.is_object() {
            if val.is_null() {
                "null".to_string()
            } else {
                "object".to_string()
            }
        } else if val.is_undefined() {
            "undefined".to_string()
        } else if val.is_symbol() {
            "symbol".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn js_arg_error_str(
        name: &str,
        expected_type: &str,
        value: JsValue,
        value_type_prefix: &str,
    ) -> JsValue {
        JsValue::from_str(&format!(
            "invalid '{}' argument provided - expected '{}' but '{}{}' provided",
            name,
            expected_type,
            value_type_prefix,
            Self::js_value_type_str(value)
        ))
    }

    fn arg_to_string(name: &str, value: JsValue) -> Result<String, JsValue> {
        if value.is_null() || value.is_undefined() {
            return Err(Self::js_arg_error_str(name, "string", value, ""));
        }
        match value.as_string() {
            Some(s) => Ok(s),
            None => Err(Self::js_arg_error_str(name, "string", value, "")),
        }
    }

    fn opt_arg_to_usize(name: &str, value: JsValue) -> Result<Option<usize>, JsValue> {
        if value.is_null() || value.is_undefined() {
            return Ok(None);
        }
        Self::arg_to_usize(name, value).map(Some)
    }

    fn arg_to_usize(name: &str, value: JsValue) -> Result<usize, JsValue> {
        if value.is_null() || value.is_undefined() {
            return Err(Self::js_arg_error_str(name, "unsigned integer", value, ""));
        }
        if let Some(number) = value.as_f64() {
            if number >= 0.0 && number.fract() == 0.0 {
                Ok(number as usize)
            } else {
                Err(Self::js_arg_error_str(
                    name,
                    "unsigned integer",
                    value,
                    "negative ",
                ))
            }
        } else {
            Err(Self::js_arg_error_str(name, "unsigned integer", value, ""))
        }
    }

    fn set_cell_values(
        &mut self,
        start_row: JsValue,
        start_col: JsValue,
        end_row: JsValue,
        end_col: JsValue,
        modify_char: char,
    ) -> Result<(), JsValue> {
        let start_row = Self::arg_to_usize("start_row", start_row)?;
        let start_col = Self::arg_to_usize("start_col", start_col)?;
        let end_row = Self::arg_to_usize("end_row", end_row)?;
        let end_col = Self::arg_to_usize("end_col", end_col)?;

        self.maze
            .definition
            .set_value(
                MazePoint {
                    row: start_row,
                    col: start_col,
                },
                MazePoint {
                    row: end_row,
                    col: end_col,
                },
                modify_char,
            )
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }
}
