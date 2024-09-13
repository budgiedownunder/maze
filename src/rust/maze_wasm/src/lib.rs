use js_sys::{Array, Object, Reflect};
use maze::{Definition, Maze, Point, Solution};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct MazeWasm {
    maze: Maze,
}

#[wasm_bindgen]
pub struct SolutionWasm {
    solution: Solution,
}

fn to_js_point_obj(point: &Point) -> Object {
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

#[wasm_bindgen]
pub enum MazeCellType {
    Empty,
    Start,
    Finish,
    Wall,
}

fn to_cell_type_enum(cell_type: char) -> MazeCellType {
    match cell_type {
        'S' => MazeCellType::Start,
        'F' => MazeCellType::Finish,
        'W' => MazeCellType::Wall,
        _ => MazeCellType::Empty,
    }
}

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
impl SolutionWasm {
    pub fn get_path_points(&self) -> Array {
        let path_points = Array::new();
        for point in &self.solution.path.points {
            path_points.push(&to_js_point_obj(point));
        }
        path_points
    }
}

impl Clone for MazeWasm {
    fn clone(&self) -> Self {
        MazeWasm {
            maze: self.maze.clone(),
        }
    }
}

#[wasm_bindgen]
impl MazeWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<MazeWasm, JsValue> {
        let def = Definition::new(0, 0);
        Ok(MazeWasm {
            maze: Maze::new(def),
        })
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) -> MazeWasm {
        self.maze.reset();
        self.clone()
    }

    #[wasm_bindgen]
    pub fn resize(
        &mut self,
        new_row_count: JsValue,
        new_col_count: JsValue,
    ) -> Result<MazeWasm, JsValue> {
        let new_row_count = Self::arg_to_usize("new_row_count", new_row_count)?;
        let new_col_count = Self::arg_to_usize("new_col_count", new_col_count)?;
        self.maze.definition.resize(new_row_count, new_col_count);
        Ok(self.clone())
    }

    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.maze.definition.is_empty()
    }

    #[wasm_bindgen]
    pub fn get_row_count(&self) -> usize {
        self.maze.definition.row_count()
    }

    #[wasm_bindgen]
    pub fn get_col_count(&self) -> usize {
        self.maze.definition.col_count()
    }

    #[wasm_bindgen]
    pub fn get_cell(&self, row: JsValue, col: JsValue) -> Result<Object, JsValue> {
        let row = Self::arg_to_usize("row", row)?;
        if row >= self.maze.definition.row_count() {
            return Err(JsValue::from_str("Row out of bounds"));
        }
        let col = Self::arg_to_usize("col", col)?;
        if col >= self.maze.definition.col_count() {
            return Err(JsValue::from_str("Column out of bounds"));
        }
        Ok(to_js_cell_info_obj(self.maze.definition.grid[row][col]))
    }

    #[wasm_bindgen]
    pub fn set_start(&mut self, start_row: JsValue, start_col: JsValue) -> Result<(), JsValue> {
        let row = Self::arg_to_usize("start_row", start_row)?;
        let col = Self::arg_to_usize("start_col", start_col)?;
        self.maze
            .definition
            .set_start(Some(Point { row, col }))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_start(&mut self) -> Result<Object, JsValue> {
        if let Some(start) = self.maze.definition.get_start() {
            return Ok(to_js_point_obj(&start));
        }
        Err(JsValue::from_str("No start cell defined"))
    }

    #[wasm_bindgen]
    pub fn set_finish(&mut self, finish_row: JsValue, finish_col: JsValue) -> Result<(), JsValue> {
        let row = Self::arg_to_usize("finish_row", finish_row)?;
        let col = Self::arg_to_usize("finish_col", finish_col)?;
        self.maze
            .definition
            .set_finish(Some(Point { row, col }))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_finish(&mut self) -> Result<Object, JsValue> {
        if let Some(finish) = self.maze.definition.get_finish() {
            return Ok(to_js_point_obj(&finish));
        }
        Err(JsValue::from_str("No finish cell defined"))
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        self.maze
            .to_json()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn from_json(&mut self, json_string: JsValue) -> Result<MazeWasm, JsValue> {
        let json_str = Self::arg_to_string("json_string", json_string)?;
        self.maze
            .from_json(&json_str)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(self.clone())
    }

    pub fn solve(&self) -> Result<SolutionWasm, JsValue> {
        let solution = self
            .maze
            .solve()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(SolutionWasm { solution })
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
            "Invalid '{}' argument provided - expected '{}' but '{}{}' provided",
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
}
