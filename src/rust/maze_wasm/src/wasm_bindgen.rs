#[allow(unused_imports)] // MazeCellTypeWasm is referenced in doc comments as an intra-doc link
use crate::wasm_common::{
    new_maze, new_maze_game, to_cell_type_enum, to_generation_algorithm, DirectionWasm,
    GenerationAlgorithmWasm, MazeCellTypeWasm, MazeGameWasm, MoveResultWasm, MazeWasm,
};
use data_model::{CellEntity, MazePoint};
use js_sys::{Array, Object, Reflect, JSON};
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
    ///     let maze = null;
    ///     let solution = null;
    ///     try {
    ///         maze = new MazeWasm();
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
    ///         solution = maze.solve();
    ///         let solutionPoints = solution.get_path_points();
    ///         console.log("Successfully solved maze. Solution points are: ", solutionPoints);
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (solution) solution.free();
    ///         if (maze) maze.free();
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

// ── MazeGameWasm helpers ──────────────────────────────────────────────────────────

fn direction_from_wasm(dir: DirectionWasm) -> maze::Direction {
    match dir {
        DirectionWasm::None  => maze::Direction::None,
        DirectionWasm::Up    => maze::Direction::Up,
        DirectionWasm::Down  => maze::Direction::Down,
        DirectionWasm::Left  => maze::Direction::Left,
        DirectionWasm::Right => maze::Direction::Right,
    }
}

fn direction_to_wasm(dir: maze::Direction) -> DirectionWasm {
    match dir {
        maze::Direction::None  => DirectionWasm::None,
        maze::Direction::Up    => DirectionWasm::Up,
        maze::Direction::Down  => DirectionWasm::Down,
        maze::Direction::Left  => DirectionWasm::Left,
        maze::Direction::Right => DirectionWasm::Right,
    }
}

// The string values below ("key", "locked"/"opening"/"open", "doorOpened") are the
// JavaScript API contract and must match the MazeBagItemType / MazeDoorState /
// MazeGameEventType constants in src/react/maze_web_server/src/wasm/mazeWasm.ts.

/// Converts a bag item to a JavaScript object (e.g. `{ type: "key", id }`).
fn to_js_bag_item_obj(item: &maze::BagItem) -> Object {
    let obj = Object::new();
    match item {
        maze::BagItem::Key { id } => {
            Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("key")).unwrap();
            Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(*id as f64)).unwrap();
        }
    }
    obj
}

/// Converts a door cell and its state to a JavaScript object (`{ row, col, state }`).
fn to_js_door_obj(row: usize, col: usize, state: maze::DoorState) -> Object {
    let obj = Object::new();
    Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(row as f64)).unwrap();
    Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(col as f64)).unwrap();
    let state_str = match state {
        maze::DoorState::Locked => "locked",
        maze::DoorState::Opening { .. } => "opening",
        maze::DoorState::Open => "open",
    };
    Reflect::set(&obj, &JsValue::from_str("state"), &JsValue::from_str(state_str)).unwrap();
    obj
}

/// Converts an uncollected key cell and its id to a JavaScript object (`{ row, col, id }`).
fn to_js_key_obj(row: usize, col: usize, id: u32) -> Object {
    let obj = Object::new();
    Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(row as f64)).unwrap();
    Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(col as f64)).unwrap();
    Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(id as f64)).unwrap();
    obj
}

/// Converts an enemy's current state to a JavaScript object
/// (`{ row, col, id, damage, movePeriodMs, enemyType? }`). `damage` and
/// `movePeriodMs` are the resolved per-enemy values (per-cell override else the
/// per-game default); `enemyType` is present only when the spawn cell carried a
/// rig override, so renderers fall back to their own default when it is absent.
fn to_js_enemy_obj(enemy: &maze::Enemy) -> Object {
    let obj = Object::new();
    Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(enemy.row as f64)).unwrap();
    Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(enemy.col as f64)).unwrap();
    Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(enemy.id as f64)).unwrap();
    Reflect::set(&obj, &JsValue::from_str("damage"), &JsValue::from_f64(enemy.damage as f64)).unwrap();
    Reflect::set(
        &obj,
        &JsValue::from_str("movePeriodMs"),
        &JsValue::from_f64(enemy.move_period_ms as f64),
    )
    .unwrap();
    if let Some(enemy_type) = enemy.enemy_type {
        Reflect::set(
            &obj,
            &JsValue::from_str("enemyType"),
            &JsValue::from_str(enemy_type.as_wire_str()),
        )
        .unwrap();
    }
    obj
}

/// Converts an uncollected health-pickup cell to a JavaScript object
/// (`{ row, col, id }`). The id is a row-major scan-order ordinal assigned at
/// query time — it is unique within the returned snapshot but shifts once an
/// `'H'` cell is consumed (the cell becomes `' '` and disappears from the
/// scan). Renderers should key on `(row, col)` for stable React reconciliation;
/// the id field is supplied for shape parity with [`to_js_key_obj`] and
/// [`to_js_enemy_obj`].
fn to_js_health_pickup_obj(row: usize, col: usize, id: u32) -> Object {
    let obj = Object::new();
    Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(row as f64)).unwrap();
    Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(col as f64)).unwrap();
    Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(id as f64)).unwrap();
    obj
}

/// Converts a tick event to a JavaScript object — one arm per
/// [`maze::GameEvent`] variant. Each arm emits the JS object shape documented
/// in the corresponding `MazeGameEventType` entry of `mazeWasm.ts`.
fn to_js_game_event_obj(event: &maze::GameEvent) -> Object {
    let obj = Object::new();
    match event {
        maze::GameEvent::DoorOpened { cell: (row, col) } => {
            Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("doorOpened")).unwrap();
            Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(*row as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(*col as f64)).unwrap();
        }
        maze::GameEvent::EnemyMoved { id, row, col } => {
            Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("enemyMoved")).unwrap();
            Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(*id as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(*row as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(*col as f64)).unwrap();
        }
        maze::GameEvent::PlayerDamaged { hp_after } => {
            Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("playerDamaged")).unwrap();
            Reflect::set(&obj, &JsValue::from_str("hpAfter"), &JsValue::from_f64(*hp_after as f64)).unwrap();
        }
        maze::GameEvent::PlayerHealed { hp_after, cell: (row, col) } => {
            Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("playerHealed")).unwrap();
            Reflect::set(&obj, &JsValue::from_str("hpAfter"), &JsValue::from_f64(*hp_after as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(*row as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(*col as f64)).unwrap();
        }
        maze::GameEvent::PlayerNotHealed { cell: (row, col), reason, message } => {
            Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("playerNotHealed")).unwrap();
            Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(*row as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(*col as f64)).unwrap();
            let reason_str = match reason {
                maze::PlayerNotHealedReason::AlreadyAtMaxHp => "already_at_max_hp",
            };
            Reflect::set(&obj, &JsValue::from_str("reason"), &JsValue::from_str(reason_str)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("message"), &JsValue::from_str(message)).unwrap();
        }
        maze::GameEvent::KeyCollected { cell: (row, col), id } => {
            Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str("keyCollected")).unwrap();
            Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from_f64(*id as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(*row as f64)).unwrap();
            Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(*col as f64)).unwrap();
        }
    }
    obj
}

fn move_result_to_wasm(result: maze::MoveResult) -> MoveResultWasm {
    match result {
        maze::MoveResult::None                => MoveResultWasm::None,
        maze::MoveResult::Moved               => MoveResultWasm::Moved,
        maze::MoveResult::Blocked             => MoveResultWasm::Blocked,
        maze::MoveResult::Complete            => MoveResultWasm::Complete,
        maze::MoveResult::BlockedByLockedDoor => MoveResultWasm::BlockedByLockedDoor,
        maze::MoveResult::StartedUnlocking    => MoveResultWasm::StartedUnlocking,
        maze::MoveResult::Stranded            => MoveResultWasm::Stranded,
        maze::MoveResult::Killed              => MoveResultWasm::Killed,
    }
}

/// A running maze game session exposed to JavaScript.
///
/// Create with [`MazeGameWasm::from_json`]. The player starts at the `S` cell with
/// direction [`DirectionWasm::None`]. Call [`MazeGameWasm::move_player`] with a
/// [`DirectionWasm`] value to advance the game.
#[wasm_bindgen]
impl MazeGameWasm {
    /// Creates a game session from a maze definition JSON string.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         console.log("player_row() = ", game.player_row());
    ///         console.log("player_col() = ", game.player_col());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a `JsValue` error if the JSON is invalid or the maze has no start cell.
    pub fn from_json(json: &str) -> Result<MazeGameWasm, JsValue> {
        new_maze_game(json).map_err(|e| JsValue::from_str(&e))
    }

    /// Attempts to move the player one cell in `dir`.
    ///
    /// Returns a [`MoveResultWasm`] indicating the outcome.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm, DirectionWasm, MoveResultWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         console.log("move_player(Right) = ", game.move_player(DirectionWasm.Right));
    ///         console.log("player_col() = ", game.player_col());
    ///         console.log("move_player(Right) = ", game.move_player(DirectionWasm.Right));
    ///         console.log("player_col() = ", game.player_col());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn move_player(&mut self, dir: DirectionWasm) -> MoveResultWasm {
        move_result_to_wasm(self.game.move_player(direction_from_wasm(dir)))
    }

    /// Returns the current player row (0-based).
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         console.log("player_row() = ", game.player_row());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn player_row(&self) -> usize {
        self.game.player_row()
    }

    /// Returns the current player column (0-based).
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         console.log("player_col() = ", game.player_col());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn player_col(&self) -> usize {
        self.game.player_col()
    }

    /// Returns the current player facing direction.
    ///
    /// The initial value is [`DirectionWasm::None`] until the first move.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm, DirectionWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         console.log("player_direction() = ", game.player_direction());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn player_direction(&self) -> DirectionWasm {
        direction_to_wasm(self.game.player_direction())
    }

    /// Returns `true` if the player has reached the finish cell.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm, DirectionWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","F"]]}');
    ///         console.log("is_complete() before move = ", game.is_complete());
    ///         game.move_player(DirectionWasm.Right);
    ///         console.log("is_complete() after move = ", game.is_complete());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn is_complete(&self) -> bool {
        self.game.is_complete()
    }

    /// Returns `true` if the game has ended in a loss — triggered when the
    /// player walks through an open door without enough keys remaining to
    /// open every real door on the solution path, or when same-cell enemy
    /// collisions drain HP to zero. Pair with [`MazeGameWasm::lose_reason`]
    /// for the cause.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","F"]]}');
    ///         console.log("is_lost() = ", game.is_lost());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn is_lost(&self) -> bool {
        self.game.is_lost()
    }

    /// Returns the lose reason — `"stranded"` if the player walked through a
    /// door no longer holding enough keys to reach the finish, `"killed"` if
    /// same-cell enemy collisions drained HP to zero, or `null` while the
    /// game is still in progress or already won.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","F"]]}');
    ///         console.log("lose_reason() = ", game.lose_reason());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn lose_reason(&self) -> JsValue {
        match self.game.lose_reason() {
            Some(maze::LoseReason::Stranded) => JsValue::from_str("stranded"),
            Some(maze::LoseReason::Killed) => JsValue::from_str("killed"),
            None => JsValue::NULL,
        }
    }

    /// Returns all cells visited by the player (including start) in visit order,
    /// as a JavaScript `Array` of `{row, col}` objects.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm, DirectionWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         game.move_player(DirectionWasm.Right);
    ///         console.log("visited_cells() = ", game.visited_cells());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn visited_cells(&self) -> Array {
        let result = Array::new();
        for &(row, col) in self.game.visited_cells() {
            result.push(&to_js_point_obj(&MazePoint { row, col }));
        }
        result
    }

    /// Picks up the collectible item (currently a key) at the player's current cell,
    /// returning it as a `{ type, id }` object, or `null` if the cell holds none.
    ///
    /// Keys are auto-collected when the player walks onto a `'K'` cell, so this
    /// normally returns `null` — the cell was cleared as the player stepped onto it.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm, DirectionWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","K","F"]]}');
    ///         game.move_player(DirectionWasm.Right); // onto the key — auto-collected
    ///         console.log("pickup() = ", game.pickup()); // null: already collected
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn pickup(&mut self) -> JsValue {
        match self.game.pickup() {
            Some(item) => to_js_bag_item_obj(&item).into(),
            None => JsValue::NULL,
        }
    }

    /// Advances time-based game state by `dt_ms` milliseconds, returning a JavaScript
    /// `Array` of event objects (e.g. `{ type: "doorOpened", row, col }`).
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm, DirectionWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","K","D","F"]]}');
    ///         game.move_player(DirectionWasm.Right); // onto the key — auto-collected
    ///         game.tick(0);                          // flush the keyCollected event
    ///         game.move_player(DirectionWasm.Right); // start unlocking the door
    ///         console.log("tick(1000) = ", game.tick(1000));
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn tick(&mut self, dt_ms: f32) -> Array {
        let result = Array::new();
        for event in self.game.tick(dt_ms) {
            result.push(&to_js_game_event_obj(&event));
        }
        result
    }

    /// Returns the door cells and their current state as a JavaScript `Array` of
    /// `{ row, col, state }` objects (`state` is `"locked"`, `"opening"`, or `"open"`).
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","D","F"]]}');
    ///         console.log("doors() = ", game.doors());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn doors(&self) -> Array {
        let result = Array::new();
        for ((row, col), state) in self.game.doors() {
            result.push(&to_js_door_obj(row, col, state));
        }
        result
    }

    /// Returns the uncollected key cells as a JavaScript `Array` of `{ row, col, id }`
    /// objects.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","K","F"]]}');
    ///         console.log("keys() = ", game.keys());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn keys(&self) -> Array {
        let result = Array::new();
        for ((row, col), id) in self.game.keys() {
            result.push(&to_js_key_obj(row, col, id));
        }
        result
    }

    /// Returns the player's bag as a JavaScript `Array` of item objects
    /// (e.g. `{ type: "key", id }`), in pickup order.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm, DirectionWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","K","F"]]}');
    ///         game.move_player(DirectionWasm.Right); // onto the key — auto-collected
    ///         console.log("bag() = ", game.bag());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn bag(&self) -> Array {
        let result = Array::new();
        for item in self.game.bag() {
            result.push(&to_js_bag_item_obj(item));
        }
        result
    }

    /// Returns the player's current HP.
    ///
    /// HP decreases on same-cell collisions with an enemy (the player moved
    /// into an enemy or an enemy moved onto the player) and increases on
    /// auto-pickup of an `'H'` cell, capped at [`MazeGameWasm::max_hp`]. When
    /// HP reaches zero the next `move_player` returns
    /// [`MoveResultWasm::Killed`] and [`MazeGameWasm::is_lost`] flips to
    /// `true`.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         console.log("hp() = ", game.hp());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn hp(&self) -> u32 {
        self.game.hp()
    }

    /// Returns the player's maximum HP — the upper bound for
    /// [`MazeGameWasm::hp`]. Set at construction (default 3) and constant
    /// for the lifetime of the game session.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         console.log("maxHp() = ", game.max_hp());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn max_hp(&self) -> u32 {
        self.game.max_hp()
    }

    /// Returns the live enemies as a JavaScript `Array` of `{ row, col, id }`
    /// objects, in stable enemy-id order. `id` is the row-major scan-order
    /// ordinal assigned at construction and is preserved across moves so
    /// renderers can correlate `enemyMoved` tick events with the same entry.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","E","F"]]}');
    ///         console.log("enemies() = ", game.enemies());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn enemies(&self) -> Array {
        let result = Array::new();
        for enemy in self.game.enemies() {
            result.push(&to_js_enemy_obj(&enemy));
        }
        result
    }

    /// Returns the uncollected health-pickup cells as a JavaScript `Array` of
    /// `{ row, col, id }` objects, in row-major scan order. `id` is the
    /// ordinal within the returned snapshot — unique among the entries in
    /// that call, but shifts after a pickup is consumed; renderers should
    /// key on `(row, col)` for stable reconciliation.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S","H","F"]]}');
    ///         console.log("healthPickups() = ", game.health_pickups());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn health_pickups(&self) -> Array {
        let result = Array::new();
        let mut id: u32 = 0;
        for (r, row) in self.game.grid().iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                if ch == 'H' {
                    result.push(&to_js_health_pickup_obj(r, c, id));
                    id += 1;
                }
            }
        }
        result
    }
    #[wasm_bindgen]
    /// Returns the static maze grid as a 2D array of single-character strings
    /// (`"S"`/`"F"`/`"W"`/`"K"`/`"D"`/`"E"`/`"H"`/`" "`).
    ///
    /// The host renders walls, flags and feature-cell sprites from this. Unlike the
    /// stored definition (where an overridden cell is an array), this is always the
    /// plain-character form, so callers never parse the char-or-array wire shape.
    /// Dynamic state (player, enemies, door open/closed, collected keys / consumed
    /// health) comes from the other getters; per-cell visual overrides come from
    /// [`MazeGameWasm::cell_overrides`].
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         console.log("grid() = ", game.grid());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn grid(&self) -> Array {
        let rows = Array::new();
        for row in self.game.grid() {
            let js_row = Array::new();
            for &ch in row {
                js_row.push(&JsValue::from_str(&ch.to_string()));
            }
            rows.push(&js_row);
        }
        rows
    }
    #[wasm_bindgen]
    /// Returns the per-cell overrides as an array of `{ row, col, entity }`, where
    /// `entity` is the wire-shape override object (a `type` discriminator plus the set
    /// fields, e.g. `{ type: "H", healthStyle: "potion" }`).
    ///
    /// The host reads static visual rigs (health / key / door) from these; the moving
    /// enemy's rig rides the live enemy object ([`MazeGameWasm::enemies`]) instead,
    /// since it walks away from its spawn cell. Returns an empty array for a maze with
    /// no overrides.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S",[{"type":"H","healthStyle":"potion"}],"F"]]}');
    ///         console.log("cell_overrides() = ", game.cell_overrides());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn cell_overrides(&self) -> Result<Array, JsValue> {
        let result = Array::new();
        for ((row, col), entities) in self.game.cell_entities() {
            if let Some(entity) = entities.first() {
                let json = serde_json::to_string(entity).map_err(|e| {
                    JsValue::from_str(&format!("failed to serialise cell entity: {e}"))
                })?;
                let obj = Object::new();
                Reflect::set(&obj, &JsValue::from_str("row"), &JsValue::from_f64(*row as f64))?;
                Reflect::set(&obj, &JsValue::from_str("col"), &JsValue::from_f64(*col as f64))?;
                Reflect::set(&obj, &JsValue::from_str("entity"), &JSON::parse(&json)?)?;
                result.push(&obj);
            }
        }
        Ok(result)
    }

    /// Returns the time in milliseconds until the next [`MazeGameWasm::tick`]
    /// will produce an event, or `null` when the game is idle.
    ///
    /// Lets a host loop sleep with `setTimeout` instead of polling at frame
    /// rate. The returned time corresponds to the next *committed* event
    /// (an enemy arrives at its new cell, a door finishes opening) —
    /// intra-cell enemy motion is never an event.
    ///
    /// - Returns `0` when events queued by prior [`MazeGameWasm::move_player`]
    ///   calls (PlayerDamaged / PlayerHealed / PlayerNotHealed) are waiting
    ///   to flush.
    /// - Otherwise returns the soonest of each enemy's `move_period_ms -
    ///   accum_ms` (only enemies with a planned step contribute) and each
    ///   opening door's remaining progress in milliseconds.
    /// - Returns `null` when no enemy is planning a step, no door is
    ///   opening, and no events are pending — the host loop can sleep until
    ///   external input (e.g. the player's next move) wakes it.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeGameWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let game = null;
    ///     try {
    ///         game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
    ///         console.log("timeUntilNextEventMs() = ", game.time_until_next_event_ms());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (game) game.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn time_until_next_event_ms(&self) -> JsValue {
        match self.game.time_until_next_event_ms() {
            Some(ms) => JsValue::from_f64(ms as f64),
            None => JsValue::NULL,
        }
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         console.log("Successfully created maze. Dimensions: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.reset();
    ///         console.log("After reset(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn reset(&mut self) {
        self.maze.reset();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm ();
    ///         console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s   )");
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.insert_rows(0, 5);
    ///         console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.insert_rows(0, 5);
    ///         console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.delete_rows(1, 3);
    ///         console.log("After delete_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.insert_rows(0, 1);
    ///         console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.insert_cols(0, 10);
    ///         console.log("After insert_cols(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///         maze.delete_cols(1, 3);
    ///         console.log("After delete_cols(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         console.log("After creation, is_empty() = ", maze.is_empty());
    ///         maze.resize(1,2);
    ///         console.log("After resize(), is_empty() = ", maze.is_empty());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         console.log("After creation, get_row_count() = ", maze.get_row_count());
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), get_row_count() = ", maze.get_row_count());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         console.log("After creation, get_col_count() = ", maze.get_col_count());
    ///         maze.resize(10, 5);
    ///         console.log("After resize(), get_col_count() = ", maze.get_col_count());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("get_cell(1, 2) = ", maze.get_cell(1, 2));
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    /// Returns the per-cell entity override at the given location, or `null`
    /// when the cell carries none.
    ///
    /// The returned object is the entity in its wire shape — a `type`
    /// discriminator (`"E"` / `"H"` / `"K"` / `"D"`) plus only the override
    /// fields that are set, e.g. `{ type: "E", enemyType: "ghost", damage: 2 }` —
    /// so JavaScript never parses the char-or-array grid form itself.
    ///
    /// # Arguments
    ///
    /// * `row` - Row index (zero-based)
    /// * `col` - Column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error if the target location is out of range.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(1, 3);
    ///         maze.set_enemy_cells(0, 1, 0, 1);
    ///         maze.set_cell_entity(0, 1, { type: "E", enemyType: "ghost", damage: 2 });
    ///         console.log("get_cell_entity(0, 1) = ", maze.get_cell_entity(0, 1));
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn get_cell_entity(&self, row: JsValue, col: JsValue) -> Result<JsValue, JsValue> {
        let row = Self::arg_to_usize("row", row)?;
        if row >= self.maze.definition.row_count() {
            return Err(JsValue::from_str("row out of bounds"));
        }
        let col = Self::arg_to_usize("col", col)?;
        if col >= self.maze.definition.col_count() {
            return Err(JsValue::from_str("column out of bounds"));
        }
        match self
            .maze
            .definition
            .cell_entities
            .get(&(row, col))
            .and_then(|entities| entities.first())
        {
            Some(entity) => {
                let json = serde_json::to_string(entity).map_err(|e| {
                    JsValue::from_str(&format!("failed to serialise cell entity: {e}"))
                })?;
                JSON::parse(&json)
            }
            None => Ok(JsValue::NULL),
        }
    }
    #[wasm_bindgen]
    /// Sets the per-cell entity override at the given location, replacing any
    /// existing one.
    ///
    /// `entity` is the wire-shape object — a `type` discriminator plus the
    /// override fields to set, e.g. `{ type: "E", enemyType: "ghost", damage: 2 }`.
    /// The `type` must match the cell's current character (set the cell to the
    /// matching kind first, e.g. via `set_enemy_cells`). An entity that sets no
    /// field is accepted but normalises away on the next serialise.
    ///
    /// # Arguments
    ///
    /// * `row` - Row index (zero-based)
    /// * `col` - Column index (zero-based)
    /// * `entity` - The entity override object
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    /// - If `entity` is not a valid entity object
    /// - If the entity `type` does not match the cell's character
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(1, 3);
    ///         maze.set_health_cells(0, 1, 0, 1);
    ///         maze.set_cell_entity(0, 1, { type: "H", healthStyle: "potion", healAmount: 2 });
    ///         console.log("get_cell_entity(0, 1) = ", maze.get_cell_entity(0, 1));
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn set_cell_entity(
        &mut self,
        row: JsValue,
        col: JsValue,
        entity: JsValue,
    ) -> Result<(), JsValue> {
        let row = Self::arg_to_usize("row", row)?;
        if row >= self.maze.definition.row_count() {
            return Err(JsValue::from_str("row out of bounds"));
        }
        let col = Self::arg_to_usize("col", col)?;
        if col >= self.maze.definition.col_count() {
            return Err(JsValue::from_str("column out of bounds"));
        }
        let json = JSON::stringify(&entity)
            .map_err(|_| JsValue::from_str("entity is not a serialisable object"))?
            .as_string()
            .ok_or_else(|| JsValue::from_str("entity is not a serialisable object"))?;
        let parsed: CellEntity = serde_json::from_str(&json)
            .map_err(|e| JsValue::from_str(&format!("invalid cell entity: {e}")))?;
        let cell_char = self.maze.definition.grid[row][col];
        if parsed.cell_char() != cell_char {
            return Err(JsValue::from_str(&format!(
                "cell entity type '{}' does not match cell character '{}'",
                parsed.cell_char(),
                cell_char
            )));
        }
        self.maze
            .definition
            .cell_entities
            .insert((row, col), vec![parsed]);
        Ok(())
    }
    #[wasm_bindgen]
    /// Clears any per-cell entity override at the given location. A cell with no
    /// override is unaffected.
    ///
    /// # Arguments
    ///
    /// * `row` - Row index (zero-based)
    /// * `col` - Column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error if the target location is out of range.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(1, 3);
    ///         maze.set_enemy_cells(0, 1, 0, 1);
    ///         maze.set_cell_entity(0, 1, { type: "E", damage: 2 });
    ///         maze.clear_cell_entity(0, 1);
    ///         console.log("get_cell_entity(0, 1) = ", maze.get_cell_entity(0, 1)); // null
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn clear_cell_entity(&mut self, row: JsValue, col: JsValue) -> Result<(), JsValue> {
        let row = Self::arg_to_usize("row", row)?;
        if row >= self.maze.definition.row_count() {
            return Err(JsValue::from_str("row out of bounds"));
        }
        let col = Self::arg_to_usize("col", col)?;
        if col >= self.maze.definition.col_count() {
            return Err(JsValue::from_str("column out of bounds"));
        }
        self.maze.definition.cell_entities.remove(&(row, col));
        Ok(())
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("Before set_start_cell(), get_cell(1, 2) = ", maze.get_cell(1, 2));
    ///         maze.set_start_cell(1, 2);
    ///         console.log("After set_start_cell(), get_cell(1, 2) = ", maze.get_cell(1, 2));
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_start_cell(1, 2);
    ///         console.log("get_start_cell() = ", maze.get_start_cell());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         console.log("Before set_finish_cell(), get_cell(3, 4) = ", maze.get_cell(3, 4));
    ///         maze.set_finish_cell(3, 4);
    ///         console.log("After set_finish_cell(), get_cell(3, 4) = ", maze.get_cell(3, 4));
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_finish_cell(9, 4);
    ///         console.log("get_finish_cell() = ", maze.get_finish_cell());
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_wall_cells(0, 1, 0, 3);
    ///         for (let col  = 0; col < 5; col ++) {
    ///             console.log(`After set_walls_cell(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
    ///         }
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    /// Sets a range of cells within the maze instance to be keys (`cell_type` = [`MazeCellTypeWasm::Key`])
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `start_col` - Start column index (zero-based)
    /// * `end_row` - End row index (zero-based)
    /// * `end_col` - End column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and then set
    /// cell (0, 2) to be a key. Then print the `cell_type` for the top row,
    /// where cell (0, 2) will be [`MazeCellTypeWasm::Key`] and the others
    /// [`MazeCellTypeWasm::Empty`].
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_key_cells(0, 2, 0, 2);
    ///         for (let col  = 0; col < 5; col ++) {
    ///             console.log(`After set_key_cells(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
    ///         }
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn set_key_cells(
        &mut self,
        start_row: JsValue,
        start_col: JsValue,
        end_row: JsValue,
        end_col: JsValue,
    ) -> Result<(), JsValue> {
        self.set_cell_values(start_row, start_col, end_row, end_col, 'K')?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Sets a range of cells within the maze instance to be doors (`cell_type` = [`MazeCellTypeWasm::Door`])
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `start_col` - Start column index (zero-based)
    /// * `end_row` - End row index (zero-based)
    /// * `end_col` - End column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and then set
    /// cell (0, 2) to be a door. Then print the `cell_type` for the top row,
    /// where cell (0, 2) will be [`MazeCellTypeWasm::Door`] and the others
    /// [`MazeCellTypeWasm::Empty`].
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_door_cells(0, 2, 0, 2);
    ///         for (let col  = 0; col < 5; col ++) {
    ///             console.log(`After set_door_cells(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
    ///         }
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn set_door_cells(
        &mut self,
        start_row: JsValue,
        start_col: JsValue,
        end_row: JsValue,
        end_col: JsValue,
    ) -> Result<(), JsValue> {
        self.set_cell_values(start_row, start_col, end_row, end_col, 'D')?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Sets a range of cells within the maze instance to be enemy spawns (`cell_type` = [`MazeCellTypeWasm::Enemy`])
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `start_col` - Start column index (zero-based)
    /// * `end_row` - End row index (zero-based)
    /// * `end_col` - End column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and then set
    /// cell (0, 2) to be an enemy spawn. Then print the `cell_type` for the
    /// top row, where cell (0, 2) will be [`MazeCellTypeWasm::Enemy`] and the
    /// others [`MazeCellTypeWasm::Empty`].
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_enemy_cells(0, 2, 0, 2);
    ///         for (let col  = 0; col < 5; col ++) {
    ///             console.log(`After set_enemy_cells(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
    ///         }
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn set_enemy_cells(
        &mut self,
        start_row: JsValue,
        start_col: JsValue,
        end_row: JsValue,
        end_col: JsValue,
    ) -> Result<(), JsValue> {
        self.set_cell_values(start_row, start_col, end_row, end_col, 'E')?;
        Ok(())
    }
    #[wasm_bindgen]
    /// Sets a range of cells within the maze instance to be health pickups (`cell_type` = [`MazeCellTypeWasm::Health`])
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `start_col` - Start column index (zero-based)
    /// * `end_row` - End row index (zero-based)
    /// * `end_col` - End column index (zero-based)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target location is out of range
    ///
    /// # Examples
    ///
    /// Create a new maze, resize it to 10 rows x 5 columns and then set
    /// cell (0, 2) to be a health pickup. Then print the `cell_type` for the
    /// top row, where cell (0, 2) will be [`MazeCellTypeWasm::Health`] and the
    /// others [`MazeCellTypeWasm::Empty`].
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(10, 5);
    ///         maze.set_health_cells(0, 2, 0, 2);
    ///         for (let col  = 0; col < 5; col ++) {
    ///             console.log(`After set_health_cells(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
    ///         }
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
    ///     }
    /// }
    /// run();
    /// ```
    pub fn set_health_cells(
        &mut self,
        start_row: JsValue,
        start_col: JsValue,
        end_row: JsValue,
        end_col: JsValue,
    ) -> Result<(), JsValue> {
        self.set_cell_values(start_row, start_col, end_row, end_col, 'H')?;
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
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
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.resize(6, 5);
    ///         maze.set_wall_cells(0, 1, 2, 4);
    ///         let json = maze.to_json();
    ///         console.log("to_json() returned: ", json);
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
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
    ///     } finally {
    ///         if (maze) maze.free();
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
    ///     let maze = null;
    ///     let solution = null;
    ///     try {
    ///         maze = new MazeWasm();
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
    ///         solution = maze.solve();
    ///         let solutionPoints = solution.get_path_points();
    ///         console.log("Maze solve() succeeded. Solution points are: ", solutionPoints);
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (solution) solution.free();
    ///         if (maze) maze.free();
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
    /// * `seed` - Seed (undefined = default random)
    /// * `door_count` - Number of doors (each with one key) to auto-place (undefined = default 0)
    /// * `spare_doors` - Number of decoy doors planted on off-spine branches after solvability check (undefined = default 0)
    /// * `spare_keys` - Number of spare keys planted on off-spine branches after solvability check (undefined = default 0)
    /// * `enemy_count` - Number of enemy cells to auto-place at random passable cells (undefined = default 0)
    /// * `health_count` - Number of health-pickup cells to auto-place at random passable cells (undefined = default 0)
    ///
    /// # Returns
    ///
    /// This function will return an error if the maze could not be generated
    ///
    /// # Examples
    ///
    /// Generate a 7×5 maze using the recursive backtracking algorithm with default options,
    /// then print the resulting maze as a JSON string
    ///
    /// ```javascript
    /// // Javascript <script> content:
    ///
    /// import init, { MazeWasm, GenerationAlgorithmWasm } from 'maze_wasm.js';
    ///
    /// async function run() {
    ///     await init();
    ///
    ///     let maze = null;
    ///     try {
    ///         maze = new MazeWasm();
    ///         maze.generate(
    ///             7,
    ///             5,
    ///             GenerationAlgorithmWasm.RecursiveBacktracking,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined,
    ///             undefined
    ///         );
    ///         let json = maze.to_json();
    ///         console.log("Maze generate() succeeded. Maze JSON is: ", json);
    ///     } catch (e) {
    ///         console.error("Operation failed: ", e);
    ///     } finally {
    ///         if (maze) maze.free();
    ///     }
    /// }
    /// run();
    /// ```
    #[allow(clippy::too_many_arguments)]
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
        seed: JsValue,
        door_count: JsValue,
        spare_doors: JsValue,
        spare_keys: JsValue,
        enemy_count: JsValue,
        health_count: JsValue,
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

        let seed: Option<u64> = if seed.is_null() || seed.is_undefined() {
            // On wasm32, thread_rng() is unavailable; derive entropy from JS Math.random()
            let hi = js_sys::Math::random().to_bits();
            let lo = js_sys::Math::random().to_bits();
            Some((hi << 32) | (lo & 0xFFFF_FFFF))
        } else {
            seed.as_f64().map(|v| v as u64)
        };

        let door_count = Self::opt_arg_to_usize("door_count", door_count)?;
        let spare_doors = Self::opt_arg_to_usize("spare_doors", spare_doors)?;
        let spare_keys = Self::opt_arg_to_usize("spare_keys", spare_keys)?;
        let enemy_count = Self::opt_arg_to_usize("enemy_count", enemy_count)?;
        let health_count = Self::opt_arg_to_usize("health_count", health_count)?;

        let options = GeneratorOptions {
            row_count,
            col_count,
            algorithm: to_generation_algorithm(algorithm),
            start,
            finish,
            min_spine_length,
            max_retries,
            branch_from_finish,
            seed,
            door_count,
            spare_doors,
            spare_keys,
            enemy_count,
            health_count,
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
