// This file exports a single function run_tests() which runs the tests for each
// JavaScript example confirming that they work
import { readFile } from 'fs/promises';
import init, { DirectionWasm, MazeGameWasm, MazeWasm, MazeCellTypeWasm, MoveResultWasm, GenerationAlgorithmWasm } from '../../pkg/maze_wasm.js';
import util from 'util';

// Custom function to handle loading WASM in Node.js
async function loadWasm() {
    const wasmBuffer = await readFile('../../pkg/maze_wasm_bg.wasm');
    await init({ module_or_path: wasmBuffer });
}

// Test MazeWasm::new() example
function testMazeNew() {
    let maze = null;
    try {
        maze = new MazeWasm();
        console.log("Successfully created maze. Dimensions: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeNewExpectedOutput() {
    return [
        "Successfully created maze. Dimensions:  0 row(s) x  0  column(s)"
    ];
}

// Test MazeWasm::reset() example
function testMazeReset() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.reset();
        console.log("After reset(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeResetExpectedOutput() {
    return [
        "After resize(), dimensions are:  10 row(s) x  5  column(s)",
        "After reset(), dimensions are:  0 row(s) x  0  column(s)"
    ];
}

// Test MazeWasm::resize() example
function testMazeResize() {
    let maze = null;
    try {
        maze = new MazeWasm();
        console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.resize(10, 5);
        console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeResizeExpectedOutput() {
    return [
        "After creation, dimensions are:  0 row(s) x  0  column(s)",
        "After resize(), dimensions are:  10 row(s) x  5  column(s)"
    ];
}

// Test MazeWasm::insert_rows() example
function testMazeInsertRows() {
    let maze = null;
    try {
        maze = new MazeWasm();
        console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.insert_rows(0, 5);
        console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeInsertRowsExpectedOutput() {
    return [
        "After creation, dimensions are:  0 row(s) x  0  column(s)",
        "After insert_rows(), dimensions are:  5 row(s) x  0  column(s)"
    ];
}

// Test MazeWasm::delete_rows() example
function testMazeDeleteRows() {
    let maze = null;
    try {
        maze = new MazeWasm();
        console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.insert_rows(0, 5);
        console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.delete_rows(2, 3);
        console.log("After delete_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeDeleteRowsExpectedOutput() {
    return [
        "After creation, dimensions are:  0 row(s) x  0  column(s)",
        "After insert_rows(), dimensions are:  5 row(s) x  0  column(s)",
        "After delete_rows(), dimensions are:  2 row(s) x  0  column(s)"
    ];
}

// Test MazeWasm::insert_cols() example
function testMazeInsertCols() {
    let maze = null;
    try {
        maze = new MazeWasm();
        console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.insert_rows(0, 1);
        console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.insert_cols(0, 10);
        console.log("After insert_cols(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeInsertColsExpectedOutput() {
    return [
        "After creation, dimensions are:  0 row(s) x  0  column(s)",
        "After insert_rows(), dimensions are:  1 row(s) x  0  column(s)",
        "After insert_cols(), dimensions are:  1 row(s) x  10  column(s)"
    ];
}

// Test MazeWasm::delete_cols() example
function testMazeDeleteCols() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.delete_cols(1, 3);
        console.log("After delete_cols(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeDeleteColsExpectedOutput() {
    return [
        "After resize(), dimensions are:  10 row(s) x  5  column(s)",
        "After delete_cols(), dimensions are:  10 row(s) x  2  column(s)"
    ];
}

// Test MazeWasm::is_empty() example
function testMazeIsEmpty() {
    let maze = null;
    try {
        maze = new MazeWasm();
        console.log("After creation, is_empty() = ", maze.is_empty());
        maze.resize(1, 2);
        console.log("After resize(), is_empty() = ", maze.is_empty());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeIsEmptyExpectedOutput() {
    return [
        "After creation, is_empty() =  true",
        "After resize(), is_empty() =  false"
    ];
}

// Test MazeWasm::get_row_count() example
function testMazeGetRowCount() {
    let maze = null;
    try {
        maze = new MazeWasm();
        console.log("After creation, get_row_count() = ", maze.get_row_count());
        maze.resize(10, 5);
        console.log("After resize(), get_row_count() = ", maze.get_row_count());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeGetRowCountExpectedOutput() {
    return [
        "After creation, get_row_count() =  0",
        "After resize(), get_row_count() =  10"
    ];
}

// Test MazeWasm::get_col_count() example
function testMazeGetColCount() {
    let maze = null;
    try {
        maze = new MazeWasm();
        console.log("After creation, get_col_count() = ", maze.get_col_count());
        maze.resize(10, 5);
        console.log("After resize(), get_col_count() = ", maze.get_col_count());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeGetColCountExpectedOutput() {
    return [
        "After creation, get_col_count() =  0",
        "After resize(), get_col_count() =  5"
    ];
}

// Test MazeWasm::get_cell() example
function testMazeGetCell() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("get_cell(1, 2) = ", maze.get_cell(1, 2));
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeGetCellExpectedOutput() {
    return [
        "get_cell(1, 2) =  { cell_type: 0 }"
    ];
}

// Test MazeWasm::set_start_cell() example
function testMazeSetStartCell() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("Before set_start_cell(), get_cell(1, 2) = ", maze.get_cell(1, 2));
        maze.set_start_cell(1, 2);
        console.log("After set_start_cell(), get_cell(1, 2) = ", maze.get_cell(1, 2));
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeSetStartCellExpectedOutput() {
    return [
        "Before set_start_cell(), get_cell(1, 2) =  { cell_type: 0 }",
        "After set_start_cell(), get_cell(1, 2) =  { cell_type: 1 }"
    ];
}

// Test MazeWasm::get_start_cell() example
function testMazeGetStartCell() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_start_cell(1, 2);
        console.log("get_start_cell() = ", maze.get_start_cell());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeGetStartCellExpectedOutput() {
    return [
        "get_start_cell() =  { row: 1, col: 2 }"
    ];
}

// Test MazeWasm::set_finish_cell() example
function testMazeSetFinishCell() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("Before set_finish_cell(), get_cell(3, 4) = ", maze.get_cell(3, 4));
        maze.set_finish_cell(3, 4);
        console.log("After set_finish_cell(), get_cell(3, 4) = ", maze.get_cell(3, 4));
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeSetFinishCellExpectedOutput() {
    return [
        "Before set_finish_cell(), get_cell(3, 4) =  { cell_type: 0 }",
        "After set_finish_cell(), get_cell(3, 4) =  { cell_type: 2 }"
    ];
}

// Test MazeWasm::get_finish_cell() example
function testMazeGetFinishCell() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_finish_cell(9, 4);
        console.log("get_finish_cell() = ", maze.get_finish_cell());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeGetFinishCellExpectedOutput() {
    return [
        "get_finish_cell() =  { row: 9, col: 4 }"
    ];
}

// Test MazeWasm::set_wall_cells() example
function testMazeSetWallCells() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_wall_cells(0, 1, 0, 3);
        for (let col = 0; col < 5; col++) {
            console.log(`After set_walls_cell(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
        }
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeSetWallCellsExpectedOutput() {
    return [
        "After set_walls_cell(), cell_type at (0, 0) =  0",
        "After set_walls_cell(), cell_type at (0, 1) =  3",
        "After set_walls_cell(), cell_type at (0, 2) =  3",
        "After set_walls_cell(), cell_type at (0, 3) =  3",
        "After set_walls_cell(), cell_type at (0, 4) =  0"
    ];
}

// Test MazeWasm::set_key_cells() example
function testMazeSetKeyCells() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_key_cells(0, 2, 0, 2);
        for (let col = 0; col < 5; col++) {
            console.log(`After set_key_cells(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
        }
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeSetKeyCellsExpectedOutput() {
    return [
        "After set_key_cells(), cell_type at (0, 0) =  0",
        "After set_key_cells(), cell_type at (0, 1) =  0",
        "After set_key_cells(), cell_type at (0, 2) =  4",
        "After set_key_cells(), cell_type at (0, 3) =  0",
        "After set_key_cells(), cell_type at (0, 4) =  0"
    ];
}

// Test MazeWasm::set_door_cells() example
function testMazeSetDoorCells() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_door_cells(0, 2, 0, 2);
        for (let col = 0; col < 5; col++) {
            console.log(`After set_door_cells(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
        }
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeSetDoorCellsExpectedOutput() {
    return [
        "After set_door_cells(), cell_type at (0, 0) =  0",
        "After set_door_cells(), cell_type at (0, 1) =  0",
        "After set_door_cells(), cell_type at (0, 2) =  5",
        "After set_door_cells(), cell_type at (0, 3) =  0",
        "After set_door_cells(), cell_type at (0, 4) =  0"
    ];
}

// Test MazeWasm::set_enemy_cells() example
function testMazeSetEnemyCells() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_enemy_cells(0, 2, 0, 2);
        for (let col = 0; col < 5; col++) {
            console.log(`After set_enemy_cells(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
        }
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeSetEnemyCellsExpectedOutput() {
    return [
        "After set_enemy_cells(), cell_type at (0, 0) =  0",
        "After set_enemy_cells(), cell_type at (0, 1) =  0",
        "After set_enemy_cells(), cell_type at (0, 2) =  6",
        "After set_enemy_cells(), cell_type at (0, 3) =  0",
        "After set_enemy_cells(), cell_type at (0, 4) =  0"
    ];
}

// Test MazeWasm::set_health_cells() example
function testMazeSetHealthCells() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_health_cells(0, 2, 0, 2);
        for (let col = 0; col < 5; col++) {
            console.log(`After set_health_cells(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
        }
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeSetHealthCellsExpectedOutput() {
    return [
        "After set_health_cells(), cell_type at (0, 0) =  0",
        "After set_health_cells(), cell_type at (0, 1) =  0",
        "After set_health_cells(), cell_type at (0, 2) =  7",
        "After set_health_cells(), cell_type at (0, 3) =  0",
        "After set_health_cells(), cell_type at (0, 4) =  0"
    ];
}

// Test MazeWasm::clear_cells() example
function testMazeClearCells() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_wall_cells(0, 1, 0, 3);
        for (let col = 0; col < 5; col++) {
            console.log(`After set_walls_cell(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
        }
        maze.clear_cells(0, 2, 3, 4);
        for (let col = 0; col < 5; col++) {
            console.log(`After clear_walls(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
        }
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeClearCellsExpectedOutput() {
    return [
        "After set_walls_cell(), cell_type at (0, 0) =  0",
        "After set_walls_cell(), cell_type at (0, 1) =  3",
        "After set_walls_cell(), cell_type at (0, 2) =  3",
        "After set_walls_cell(), cell_type at (0, 3) =  3",
        "After set_walls_cell(), cell_type at (0, 4) =  0",
        "After clear_walls(), cell_type at (0, 0) =  0",
        "After clear_walls(), cell_type at (0, 1) =  3",
        "After clear_walls(), cell_type at (0, 2) =  0",
        "After clear_walls(), cell_type at (0, 3) =  0",
        "After clear_walls(), cell_type at (0, 4) =  0"
    ];
}

// Test MazeWasm::to_json() example
function testMazeToJSON() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(6, 5);
        maze.set_wall_cells(0, 1, 2, 4);
        let json = maze.to_json();
        console.log("to_json() returned: ", json);
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeToJSONExpectedOutput() {
    return [
        `to_json() returned:  {"id":"","name":"","definition":{"grid":[[" ","W","W","W","W"],[" ","W","W","W","W"],[" ","W","W","W","W"],[" "," "," "," "," "],[" "," "," "," "," "],[" "," "," "," "," "]]}}`
    ];
}

// Test MazeWasm::from_json() example
function testMazeFromJSON() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.from_json(`{
                    \"id\":\"maze_id\",
                    \"name\":\"test\",
                    \"definition\": {
                        \"grid\":[
                            [\"S\", \"W\", \" \", \" \", \"W\"],
                            [\" \", \"W\", \" \", \"W\", \" \"],
                            [\" \", \" \", \" \", \"W\", \"F\"],
                            [\"W\", \" \", \"W\", \" \", \" \"],
                            [\" \", \" \", \" \", \"W\", \" \"],
                            [\"W\", \"W\", \" \", \" \", \" \"],
                            [\"W\", \"W\", \" \", \"W\", \" \"]
                        ]
                }}`);
        for (let row = 0; row < maze.get_row_count(); row++) {
            for (let col = 0; col < maze.get_col_count(); col++) {
                console.log(`After from_json(), cell_type at (${row}, ${col}) = `, maze.get_cell(row, col).cell_type);
            }
        }
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeFromJSONExpectedOutput() {
    return [
        "After from_json(), cell_type at (0, 0) =  1",
        "After from_json(), cell_type at (0, 1) =  3",
        "After from_json(), cell_type at (0, 2) =  0",
        "After from_json(), cell_type at (0, 3) =  0",
        "After from_json(), cell_type at (0, 4) =  3",
        "After from_json(), cell_type at (1, 0) =  0",
        "After from_json(), cell_type at (1, 1) =  3",
        "After from_json(), cell_type at (1, 2) =  0",
        "After from_json(), cell_type at (1, 3) =  3",
        "After from_json(), cell_type at (1, 4) =  0",
        "After from_json(), cell_type at (2, 0) =  0",
        "After from_json(), cell_type at (2, 1) =  0",
        "After from_json(), cell_type at (2, 2) =  0",
        "After from_json(), cell_type at (2, 3) =  3",
        "After from_json(), cell_type at (2, 4) =  2",
        "After from_json(), cell_type at (3, 0) =  3",
        "After from_json(), cell_type at (3, 1) =  0",
        "After from_json(), cell_type at (3, 2) =  3",
        "After from_json(), cell_type at (3, 3) =  0",
        "After from_json(), cell_type at (3, 4) =  0",
        "After from_json(), cell_type at (4, 0) =  0",
        "After from_json(), cell_type at (4, 1) =  0",
        "After from_json(), cell_type at (4, 2) =  0",
        "After from_json(), cell_type at (4, 3) =  3",
        "After from_json(), cell_type at (4, 4) =  0",
        "After from_json(), cell_type at (5, 0) =  3",
        "After from_json(), cell_type at (5, 1) =  3",
        "After from_json(), cell_type at (5, 2) =  0",
        "After from_json(), cell_type at (5, 3) =  0",
        "After from_json(), cell_type at (5, 4) =  0",
        "After from_json(), cell_type at (6, 0) =  3",
        "After from_json(), cell_type at (6, 1) =  3",
        "After from_json(), cell_type at (6, 2) =  0",
        "After from_json(), cell_type at (6, 3) =  3",
        "After from_json(), cell_type at (6, 4) =  0"
    ];
}

// Test MazeWasm::solve() example
function testMazeSolve() {
    let maze = null;
    let solution = null;
    try {
        maze = new MazeWasm();
        maze.from_json(`{
                    \"id\":\"maze_id\",
                    \"name\":\"test\",
                    \"definition\": {
                        \"grid\":[
                            [\"S\", \"W\", \" \", \" \", \"W\"],
                            [\" \", \"W\", \" \", \"W\", \" \"],
                            [\" \", \" \", \" \", \"W\", \"F\"],
                            [\"W\", \" \", \"W\", \" \", \" \"],
                            [\" \", \" \", \" \", \"W\", \" \"],
                            [\"W\", \"W\", \" \", \" \", \" \"],
                            [\"W\", \"W\", \" \", \"W\", \" \"]
                        ]
                }}`);
        solution = maze.solve();
        let solutionPoints = solution.get_path_points();
        console.log("Maze solve() succeeded. Solution points are: ", solutionPoints);
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (solution) solution.free();
        if (maze) maze.free();
    }
}

function testMazeSolveExpectedOutput() {
    return [
        "Maze solve() succeeded. Solution points are:  [\n" +
        "  { row: 0, col: 0 },\n" +
        "  { row: 1, col: 0 },\n" +
        "  { row: 2, col: 0 },\n" +
        "  { row: 2, col: 1 },\n" +
        "  { row: 3, col: 1 },\n" +
        "  { row: 4, col: 1 },\n" +
        "  { row: 4, col: 2 },\n" +
        "  { row: 5, col: 2 },\n" +
        "  { row: 5, col: 3 },\n" +
        "  { row: 5, col: 4 },\n" +
        "  { row: 4, col: 4 },\n" +
        "  { row: 3, col: 4 },\n" +
        "  { row: 2, col: 4 }\n" +
        "]"
    ];
}

// Test MazeWasm::generate() example
function testMazeGenerate() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.generate(
            7,
            5,
            GenerationAlgorithmWasm.RecursiveBacktracking,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined
        );
        let json = maze.to_json();
        console.log("Maze generate() succeeded. Dimensions: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return json.length > 0;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeGenerateExpectedOutput() {
    return [
        "Maze generate() succeeded. Dimensions:  7 row(s) x  5  column(s)"
    ];
}

// Test MazeWasm::generate() example with explicit seed
function testMazeGenerateSeeded() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.generate(
            9,
            7,
            GenerationAlgorithmWasm.RecursiveBacktracking,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined,
            12345,
            undefined,
            undefined,
            undefined,
            undefined,
            undefined
        );
        console.log("Maze generate() with seed succeeded. Dimensions: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        let dimensionsMatch = maze.get_row_count() === 9 && maze.get_col_count() === 7;
        return dimensionsMatch;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeGenerateSeededExpectedOutput() {
    return [
        "Maze generate() with seed succeeded. Dimensions:  9 row(s) x  7  column(s)"
    ];
}

// Test MazeSolutionWasm::get_path_points() example
function testMazeSolutionGetPathPoints() {
    let maze = null;
    let solution = null;
    try {
        maze = new MazeWasm();
        maze.from_json(`{
            \"id\":\"maze_id\",
            \"name\":\"test\",
            \"definition\": {
                \"grid\":[
                    [\"S\", \"W\", \" \", \" \", \"W\"],
                    [\" \", \"W\", \" \", \"W\", \" \"],
                    [\" \", \" \", \" \", \"W\", \"F\"],
                    [\"W\", \" \", \"W\", \" \", \" \"],
                    [\" \", \" \", \" \", \"W\", \" \"],
                    [\"W\", \"W\", \" \", \" \", \" \"],
                    [\"W\", \"W\", \" \", \"W\", \" \"]
                ]
        }}`);
        solution = maze.solve();
        let solutionPoints = solution.get_path_points();
        console.log("Successfully solved maze. Solution points are: ", solutionPoints);
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (solution) solution.free();
        if (maze) maze.free();
    }
}

function testMazeSolutionGetPathPointsExpectedOutput() {
    return [
        "Successfully solved maze. Solution points are:  [\n" +
        "  { row: 0, col: 0 },\n" +
        "  { row: 1, col: 0 },\n" +
        "  { row: 2, col: 0 },\n" +
        "  { row: 2, col: 1 },\n" +
        "  { row: 3, col: 1 },\n" +
        "  { row: 4, col: 1 },\n" +
        "  { row: 4, col: 2 },\n" +
        "  { row: 5, col: 2 },\n" +
        "  { row: 5, col: 3 },\n" +
        "  { row: 5, col: 4 },\n" +
        "  { row: 4, col: 4 },\n" +
        "  { row: 3, col: 4 },\n" +
        "  { row: 2, col: 4 }\n" +
        "]"
    ];
}

// Test MazeGameWasm::from_json() example
function testMazeGameFromJson() {
    let game = null;
    try {
        let json = '{"grid":[["S"," ","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("player_row() = ", game.player_row());
        console.log("player_col() = ", game.player_col());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameFromJsonExpectedOutput() {
    return [
        "player_row() =  0",
        "player_col() =  0"
    ];
}

// Test MazeGameWasm::move_player() example
function testMazeGameMovePlayer() {
    let game = null;
    try {
        let json = '{"grid":[["S"," ","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("move_player(Right) = ", game.move_player(DirectionWasm.Right));
        console.log("player_col() = ", game.player_col());
        console.log("move_player(Right) = ", game.move_player(DirectionWasm.Right));
        console.log("player_col() = ", game.player_col());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameMovePlayerExpectedOutput() {
    return [
        "move_player(Right) =  1",
        "player_col() =  1",
        "move_player(Right) =  3",
        "player_col() =  2"
    ];
}

// Test MazeGameWasm::player_row() example
function testMazeGamePlayerRow() {
    let game = null;
    try {
        let json = '{"grid":[["S"," ","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("player_row() = ", game.player_row());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGamePlayerRowExpectedOutput() {
    return [
        "player_row() =  0"
    ];
}

// Test MazeGameWasm::player_col() example
function testMazeGamePlayerCol() {
    let game = null;
    try {
        let json = '{"grid":[["S"," ","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("player_col() = ", game.player_col());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGamePlayerColExpectedOutput() {
    return [
        "player_col() =  0"
    ];
}

// Test MazeGameWasm::player_direction() example
function testMazeGamePlayerDirection() {
    let game = null;
    try {
        let json = '{"grid":[["S"," ","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("player_direction() = ", game.player_direction());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGamePlayerDirectionExpectedOutput() {
    return [
        "player_direction() =  0"
    ];
}

// Test MazeGameWasm::is_complete() example
function testMazeGameIsComplete() {
    let game = null;
    try {
        let json = '{"grid":[["S","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("is_complete() before move = ", game.is_complete());
        game.move_player(DirectionWasm.Right);
        console.log("is_complete() after move = ", game.is_complete());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameIsCompleteExpectedOutput() {
    return [
        "is_complete() before move =  false",
        "is_complete() after move =  true"
    ];
}

// Test MazeGameWasm::is_lost() example
function testMazeGameIsLost() {
    let game = null;
    try {
        let json = '{"grid":[["S","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("is_lost() = ", game.is_lost());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameIsLostExpectedOutput() {
    return [
        "is_lost() =  false"
    ];
}

// Test MazeGameWasm::lose_reason() example
function testMazeGameLoseReason() {
    let game = null;
    try {
        let json = '{"grid":[["S","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("lose_reason() = ", game.lose_reason());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameLoseReasonExpectedOutput() {
    return [
        "lose_reason() =  null"
    ];
}

// Test MazeGameWasm::visited_cells() example
function testMazeGameVisitedCells() {
    let game = null;
    try {
        let json = '{"grid":[["S"," ","F"]]}';
        game = MazeGameWasm.from_json(json);
        game.move_player(DirectionWasm.Right);
        console.log("visited_cells() = ", game.visited_cells());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameVisitedCellsExpectedOutput() {
    return [
        "visited_cells() =  [ { row: 0, col: 0 }, { row: 0, col: 1 } ]"
    ];
}

// Test MazeGameWasm::pickup() example
function testMazeGamePickup() {
    let game = null;
    try {
        let json = '{"grid":[["S","K","F"]]}';
        game = MazeGameWasm.from_json(json);
        game.move_player(DirectionWasm.Right); // onto the key — auto-collected
        console.log("pickup() = ", game.pickup()); // null: already collected
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGamePickupExpectedOutput() {
    return [
        "pickup() =  null"
    ];
}

// Test MazeGameWasm::doors() example
function testMazeGameDoors() {
    let game = null;
    try {
        let json = '{"grid":[["S","D","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("doors() = ", game.doors());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameDoorsExpectedOutput() {
    return [
        "doors() =  [ { row: 0, col: 1, state: 'locked' } ]"
    ];
}

// Test MazeGameWasm::keys() example
function testMazeGameKeys() {
    let game = null;
    try {
        let json = '{"grid":[["S","K","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("keys() = ", game.keys());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameKeysExpectedOutput() {
    return [
        "keys() =  [ { row: 0, col: 1, id: 0 } ]"
    ];
}

// Test MazeGameWasm::bag() example
function testMazeGameBag() {
    let game = null;
    try {
        let json = '{"grid":[["S","K","F"]]}';
        game = MazeGameWasm.from_json(json);
        game.move_player(DirectionWasm.Right); // onto the key — auto-collected
        console.log("bag() = ", game.bag());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameBagExpectedOutput() {
    return [
        "bag() =  [ { type: 'key', id: 0 } ]"
    ];
}

// Test MazeGameWasm::tick() example
function testMazeGameTick() {
    let game = null;
    try {
        let json = '{"grid":[["S","K","D","F"]]}';
        game = MazeGameWasm.from_json(json);
        game.move_player(DirectionWasm.Right); // onto the key — auto-collected
        game.tick(0);                           // flush the keyCollected event
        game.move_player(DirectionWasm.Right); // start unlocking the door
        console.log("tick(1000) = ", game.tick(1000));
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameTickExpectedOutput() {
    return [
        "tick(1000) =  [ { type: 'doorOpened', row: 0, col: 2 } ]"
    ];
}

// Test MazeGameWasm::hp() example
function testMazeGameHp() {
    let game = null;
    try {
        let json = '{"grid":[["S"," ","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("hp() = ", game.hp());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameHpExpectedOutput() {
    return [
        "hp() =  3"
    ];
}

// Test MazeGameWasm::max_hp() example
function testMazeGameMaxHp() {
    let game = null;
    try {
        let json = '{"grid":[["S"," ","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("maxHp() = ", game.max_hp());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameMaxHpExpectedOutput() {
    return [
        "maxHp() =  3"
    ];
}

// Test MazeGameWasm::enemies() example
function testMazeGameEnemies() {
    let game = null;
    try {
        let json = '{"grid":[["S","E","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("enemies() = ", game.enemies());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameEnemiesExpectedOutput() {
    return [
        "enemies() =  [ { row: 0, col: 1, id: 0, damage: 1, movePeriodMs: 1500 } ]"
    ];
}

// Test MazeGameWasm::health_pickups() example
function testMazeGameHealthPickups() {
    let game = null;
    try {
        let json = '{"grid":[["S","H","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("healthPickups() = ", game.health_pickups());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameHealthPickupsExpectedOutput() {
    return [
        "healthPickups() =  [ { row: 0, col: 1, id: 0 } ]"
    ];
}

// Test MazeGameWasm::time_until_next_event_ms() example
function testMazeGameTimeUntilNextEventMs() {
    let game = null;
    try {
        let json = '{"grid":[["S"," ","F"]]}';
        game = MazeGameWasm.from_json(json);
        console.log("timeUntilNextEventMs() = ", game.time_until_next_event_ms());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (game) game.free();
    }
}

function testMazeGameTimeUntilNextEventMsExpectedOutput() {
    return [
        "timeUntilNextEventMs() =  null"
    ];
}

// Test MazeWasm::get_cell_entity() example
function testMazeGetCellEntity() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(1, 3);
        maze.set_enemy_cells(0, 1, 0, 1);
        maze.set_cell_entity(0, 1, { type: "E", enemyType: "ghost", damage: 2 });
        console.log("get_cell_entity(0, 1) = ", maze.get_cell_entity(0, 1));
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeGetCellEntityExpectedOutput() {
    return [
        "get_cell_entity(0, 1) =  { type: 'E', enemyType: 'ghost', damage: 2 }"
    ];
}

// Test MazeWasm::set_cell_entity() example
function testMazeSetCellEntity() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(1, 3);
        maze.set_health_cells(0, 1, 0, 1);
        maze.set_cell_entity(0, 1, { type: "H", healthStyle: "potion", healAmount: 2 });
        console.log("get_cell_entity(0, 1) = ", maze.get_cell_entity(0, 1));
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeSetCellEntityExpectedOutput() {
    return [
        "get_cell_entity(0, 1) =  { type: 'H', healthStyle: 'potion', healAmount: 2 }"
    ];
}

// Test MazeWasm::clear_cell_entity() example
function testMazeClearCellEntity() {
    let maze = null;
    try {
        maze = new MazeWasm();
        maze.resize(1, 3);
        maze.set_enemy_cells(0, 1, 0, 1);
        maze.set_cell_entity(0, 1, { type: "E", damage: 2 });
        maze.clear_cell_entity(0, 1);
        console.log("get_cell_entity(0, 1) = ", maze.get_cell_entity(0, 1)); // null
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    } finally {
        if (maze) maze.free();
    }
}

function testMazeClearCellEntityExpectedOutput() {
    return [
        "get_cell_entity(0, 1) =  null"
    ];
}

// Tests
const tests = [
    { name: "MazeWasm:new() example", testFunction: testMazeNew, expectedOutput: testMazeNewExpectedOutput },
    { name: "MazeWasm:reset() example", testFunction: testMazeReset, expectedOutput: testMazeResetExpectedOutput },
    { name: "MazeWasm:resize() example", testFunction: testMazeResize, expectedOutput: testMazeResizeExpectedOutput },
    { name: "MazeWasm:insert_rows() example", testFunction: testMazeInsertRows, expectedOutput: testMazeInsertRowsExpectedOutput },
    { name: "MazeWasm:delete_rows() example", testFunction: testMazeDeleteRows, expectedOutput: testMazeDeleteRowsExpectedOutput },
    { name: "MazeWasm:insert_cols() example", testFunction: testMazeInsertCols, expectedOutput: testMazeInsertColsExpectedOutput },
    { name: "MazeWasm:delete_cols() example", testFunction: testMazeDeleteCols, expectedOutput: testMazeDeleteColsExpectedOutput },
    { name: "MazeWasm:is_empty() example", testFunction: testMazeIsEmpty, expectedOutput: testMazeIsEmptyExpectedOutput },
    { name: "MazeWasm:get_row_count() example", testFunction: testMazeGetRowCount, expectedOutput: testMazeGetRowCountExpectedOutput },
    { name: "MazeWasm:get_col_count() example", testFunction: testMazeGetColCount, expectedOutput: testMazeGetColCountExpectedOutput },
    { name: "MazeWasm:get_cell() example", testFunction: testMazeGetCell, expectedOutput: testMazeGetCellExpectedOutput },
    { name: "MazeWasm:get_cell_entity() example", testFunction: testMazeGetCellEntity, expectedOutput: testMazeGetCellEntityExpectedOutput },
    { name: "MazeWasm:set_cell_entity() example", testFunction: testMazeSetCellEntity, expectedOutput: testMazeSetCellEntityExpectedOutput },
    { name: "MazeWasm:clear_cell_entity() example", testFunction: testMazeClearCellEntity, expectedOutput: testMazeClearCellEntityExpectedOutput },
    { name: "MazeWasm:set_start_cell() example", testFunction: testMazeSetStartCell, expectedOutput: testMazeSetStartCellExpectedOutput },
    { name: "MazeWasm:get_start_cell() example", testFunction: testMazeGetStartCell, expectedOutput: testMazeGetStartCellExpectedOutput },
    { name: "MazeWasm:set_finish_cell() example", testFunction: testMazeSetFinishCell, expectedOutput: testMazeSetFinishCellExpectedOutput },
    { name: "MazeWasm:get_finish_cell() example", testFunction: testMazeGetFinishCell, expectedOutput: testMazeGetFinishCellExpectedOutput },
    { name: "MazeWasm:set_wall_cells() example", testFunction: testMazeSetWallCells, expectedOutput: testMazeSetWallCellsExpectedOutput },
    { name: "MazeWasm:set_key_cells() example", testFunction: testMazeSetKeyCells, expectedOutput: testMazeSetKeyCellsExpectedOutput },
    { name: "MazeWasm:set_door_cells() example", testFunction: testMazeSetDoorCells, expectedOutput: testMazeSetDoorCellsExpectedOutput },
    { name: "MazeWasm:set_enemy_cells() example", testFunction: testMazeSetEnemyCells, expectedOutput: testMazeSetEnemyCellsExpectedOutput },
    { name: "MazeWasm:set_health_cells() example", testFunction: testMazeSetHealthCells, expectedOutput: testMazeSetHealthCellsExpectedOutput },
    { name: "MazeWasm:clear_cells() example", testFunction: testMazeClearCells, expectedOutput: testMazeClearCellsExpectedOutput },
    { name: "MazeWasm:to_json() example", testFunction: testMazeToJSON, expectedOutput: testMazeToJSONExpectedOutput },
    { name: "MazeWasm:from_json() example", testFunction: testMazeFromJSON, expectedOutput: testMazeFromJSONExpectedOutput },
    { name: "MazeWasm:solve() example", testFunction: testMazeSolve, expectedOutput: testMazeSolveExpectedOutput },
    { name: "MazeWasm:generate() example", testFunction: testMazeGenerate, expectedOutput: testMazeGenerateExpectedOutput },
    { name: "MazeWasm:generate() example (seeded)", testFunction: testMazeGenerateSeeded, expectedOutput: testMazeGenerateSeededExpectedOutput },
    { name: "MazeSolutionWasm:get_path_points() example", testFunction: testMazeSolutionGetPathPoints, expectedOutput: testMazeSolutionGetPathPointsExpectedOutput },
    { name: "MazeGameWasm:from_json() example", testFunction: testMazeGameFromJson, expectedOutput: testMazeGameFromJsonExpectedOutput },
    { name: "MazeGameWasm:move_player() example", testFunction: testMazeGameMovePlayer, expectedOutput: testMazeGameMovePlayerExpectedOutput },
    { name: "MazeGameWasm:player_row() example", testFunction: testMazeGamePlayerRow, expectedOutput: testMazeGamePlayerRowExpectedOutput },
    { name: "MazeGameWasm:player_col() example", testFunction: testMazeGamePlayerCol, expectedOutput: testMazeGamePlayerColExpectedOutput },
    { name: "MazeGameWasm:player_direction() example", testFunction: testMazeGamePlayerDirection, expectedOutput: testMazeGamePlayerDirectionExpectedOutput },
    { name: "MazeGameWasm:is_complete() example", testFunction: testMazeGameIsComplete, expectedOutput: testMazeGameIsCompleteExpectedOutput },
    { name: "MazeGameWasm:is_lost() example", testFunction: testMazeGameIsLost, expectedOutput: testMazeGameIsLostExpectedOutput },
    { name: "MazeGameWasm:lose_reason() example", testFunction: testMazeGameLoseReason, expectedOutput: testMazeGameLoseReasonExpectedOutput },
    { name: "MazeGameWasm:visited_cells() example", testFunction: testMazeGameVisitedCells, expectedOutput: testMazeGameVisitedCellsExpectedOutput },
    { name: "MazeGameWasm:pickup() example", testFunction: testMazeGamePickup, expectedOutput: testMazeGamePickupExpectedOutput },
    { name: "MazeGameWasm:doors() example", testFunction: testMazeGameDoors, expectedOutput: testMazeGameDoorsExpectedOutput },
    { name: "MazeGameWasm:keys() example", testFunction: testMazeGameKeys, expectedOutput: testMazeGameKeysExpectedOutput },
    { name: "MazeGameWasm:bag() example", testFunction: testMazeGameBag, expectedOutput: testMazeGameBagExpectedOutput },
    { name: "MazeGameWasm:tick() example", testFunction: testMazeGameTick, expectedOutput: testMazeGameTickExpectedOutput },
    { name: "MazeGameWasm:hp() example", testFunction: testMazeGameHp, expectedOutput: testMazeGameHpExpectedOutput },
    { name: "MazeGameWasm:max_hp() example", testFunction: testMazeGameMaxHp, expectedOutput: testMazeGameMaxHpExpectedOutput },
    { name: "MazeGameWasm:enemies() example", testFunction: testMazeGameEnemies, expectedOutput: testMazeGameEnemiesExpectedOutput },
    { name: "MazeGameWasm:health_pickups() example", testFunction: testMazeGameHealthPickups, expectedOutput: testMazeGameHealthPickupsExpectedOutput },
    { name: "MazeGameWasm:time_until_next_event_ms() example", testFunction: testMazeGameTimeUntilNextEventMs, expectedOutput: testMazeGameTimeUntilNextEventMsExpectedOutput },
];

const errorTemplate = (test, i, expected, logRows) =>
    `Test "${test.name}" generated unexpected output content in row ${i + 1}:
  Expected Length: ${expected[i].length}
  Expected Content:  "${expected[i]}"
  Generated Length: ${logRows[i].length}
  Generated Content:  "${logRows[i]}"`;

// Function to run all tests
function runTests(hideResults) {
    const originalConsoleLog = console.log;
    let logRows = [];

    function interceptConsoleLog() {
        console.log = function (...args) {
            const message = util.format(...args);
            logRows.push(message);
            if (!hideResults)
                originalConsoleLog(message);
        };
    };

    function resetConsoleLog() {
        console.log = originalConsoleLog;
        logRows = [];
    }

    function expectedMatchesConsoleLog(test) {
        let matches = true;
        const expected = test.expectedOutput();
        if (logRows.length == expected.length) {
            for (let i = 0; i < logRows.length; i++) {
                if (logRows[i] != expected[i]) {
                    console.error(errorTemplate(test, i, expected, logRows));
                    matches = false;
                }
            }

        } else {
            console.error(`Test "${test.name}" did not generate the expected number of output rows (expected: ${expected.length}, found: ${logRows.length})`);
            matches = false;
        }
        return matches;
    }

    let allSucceeded = true;
    let successCount = 0;
    for (let i = 0; i < tests.length; i++) {
        const test = tests[i];
        resetConsoleLog();
        console.log(`Running test ${i + 1} of ${tests.length} => ${test.name}...`);
        interceptConsoleLog();
        const result = test.testFunction();
        if (result) {
            if (test.expectedOutput) {
                const resultsMatch = expectedMatchesConsoleLog(test);
                if (!resultsMatch) {
                    allSucceeded = false;
                } else {
                    successCount++;
                }
            } else {
                console.error(`Test "${test.name}" does not have an expected output function defined.`);
                allSucceeded = false;
            }
        } else {
            console.error(`Test "${test.name}" failed to run successfully.`);
            allSucceeded = false;
        }
    }
    resetConsoleLog();
    if (successCount != tests.length) {
        console.error(`${tests.length - successCount} of the ${tests.length} JavaScript examples tests failed`)
    }
    return allSucceeded;
}

async function run_tests(hide_results) {
    await loadWasm();
    return runTests(hide_results);
}

export { run_tests };
