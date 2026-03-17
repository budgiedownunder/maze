// This file exports a single function run_tests() which runs the tests for each
// JavaScript example confirming that they work
import { readFile } from 'fs/promises';
import init, { MazeWasm, MazeCellTypeWasm, GenerationAlgorithmWasm } from '../../pkg/maze_wasm.js';
import util from 'util';

// Custom function to handle loading WASM in Node.js
async function loadWasm() {
    const wasmBuffer = await readFile('../../pkg/maze_wasm_bg.wasm');
    await init({ module_or_path: wasmBuffer });
}

// Test MazeWasm::new() example
function testMazeNew() {
    try {
        let maze = new MazeWasm();
        console.log("Successfully created maze. Dimensions: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    }
}

function testMazeNewExpectedOutput() {
    return [
        "Successfully created maze. Dimensions:  0 row(s) x  0  column(s)"
    ];
}

// Test MazeWasm::reset() example
function testMazeReset() {
    try {
        let maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.reset();
        console.log("After reset(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.resize(10, 5);
        console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.insert_rows(0, 5);
        console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.insert_rows(0, 5);
        console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.delete_rows(2, 3);
        console.log("After delete_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        console.log("After creation, dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.insert_rows(0, 1);
        console.log("After insert_rows(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.insert_cols(0, 10);
        console.log("After insert_cols(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("After resize(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        maze.delete_cols(1, 3);
        console.log("After delete_cols(), dimensions are: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        console.log("After creation, is_empty() = ", maze.is_empty());
        maze.resize(1, 2);
        console.log("After resize(), is_empty() = ", maze.is_empty());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        console.log("After creation, get_row_count() = ", maze.get_row_count());
        maze.resize(10, 5);
        console.log("After resize(), get_row_count() = ", maze.get_row_count());
        return true;
    } catch (e) {
        consolele.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        console.log("After creation, get_col_count() = ", maze.get_col_count());
        maze.resize(10, 5);
        console.log("After resize(), get_col_count() = ", maze.get_col_count());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("get_cell(1, 2) = ", maze.get_cell(1, 2));
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    }
}

function testMazeGetCellExpectedOutput() {
    return [
        "get_cell(1, 2) =  { cell_type: 0 }"
    ];
}

// Test MazeWasm::set_start_cell() example
function testMazeSetStartCell() {
    try {
        let maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("Before set_start_cell(), get_cell(1, 2) = ", maze.get_cell(1, 2));
        maze.set_start_cell(1, 2);
        console.log("After set_start_cell(), get_cell(1, 2) = ", maze.get_cell(1, 2));
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_start_cell(1, 2);
        console.log("get_start_cell() = ", maze.get_start_cell());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    }
}

function testMazeGetStartCellExpectedOutput() {
    return [
        "get_start_cell() =  { row: 1, col: 2 }"
    ];
}

// Test MazeWasm::set_finish_cell() example
function testMazeSetFinishCell() {
    try {
        let maze = new MazeWasm();
        maze.resize(10, 5);
        console.log("Before set_finish_cell(), get_cell(3, 4) = ", maze.get_cell(3, 4));
        maze.set_finish_cell(3, 4);
        console.log("After set_finish_cell(), get_cell(3, 4) = ", maze.get_cell(3, 4));
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_finish_cell(9, 4);
        console.log("get_finish_cell() = ", maze.get_finish_cell());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    }
}

function testMazeGetFinishCellExpectedOutput() {
    return [
        "get_finish_cell() =  { row: 9, col: 4 }"
    ];
}

// Test MazeWasm::set_wall_cells() example
function testMazeSetWallCells() {
    try {
        let maze = new MazeWasm();
        maze.resize(10, 5);
        maze.set_wall_cells(0, 1, 0, 3);
        for (let col = 0; col < 5; col++) {
            console.log(`After set_walls_cell(), cell_type at (0, ${col}) = `, maze.get_cell(0, col).cell_type);
        }
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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

// Test MazeWasm::clear_cells() example
function testMazeClearCells() {
    try {
        let maze = new MazeWasm();
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
    try {
        let maze = new MazeWasm();
        maze.resize(6, 5);
        maze.set_wall_cells(0, 1, 2, 4);
        let json = maze.to_json();
        console.log("to_json() returned: ", json);
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    }
}

function testMazeToJSONExpectedOutput() {
    return [
        `to_json() returned:  {"id":"","name":"","definition":{"grid":[[" ","W","W","W","W"],[" ","W","W","W","W"],[" ","W","W","W","W"],[" "," "," "," "," "],[" "," "," "," "," "],[" "," "," "," "," "]]}}`
    ];
}

// Test MazeWasm::from_json() example
function testMazeFromJSON() {
    try {
        let maze = new MazeWasm();
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
    try {
        let maze = new MazeWasm();
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
        let solution = maze.solve();
        let solutionPoints = solution.get_path_points();
        console.log("Maze solve() succeeded. Solution points are: ", solutionPoints);
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    try {
        let maze = new MazeWasm();
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
            undefined
        );
        let json = maze.to_json();
        console.log("Maze generate() succeeded. Dimensions: ", maze.get_row_count(), "row(s) x ", maze.get_col_count(), " column(s)");
        return json.length > 0;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    }
}

function testMazeGenerateExpectedOutput() {
    return [
        "Maze generate() succeeded. Dimensions:  7 row(s) x  5  column(s)"
    ];
}

// Test MazeSolutionWasm::get_path_points() example
function testMazeSolutionGetPathPoints() {
    try {
        let maze = new MazeWasm();
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
        let solution = maze.solve();
        let solutionPoints = solution.get_path_points();
        console.log("Successfully solved maze. Solution points are: ", solutionPoints);
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
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
    { name: "MazeWasm:set_start_cell() example", testFunction: testMazeSetStartCell, expectedOutput: testMazeSetStartCellExpectedOutput },
    { name: "MazeWasm:get_start_cell() example", testFunction: testMazeGetStartCell, expectedOutput: testMazeGetStartCellExpectedOutput },
    { name: "MazeWasm:set_finish_cell() example", testFunction: testMazeSetFinishCell, expectedOutput: testMazeSetFinishCellExpectedOutput },
    { name: "MazeWasm:get_finish_cell() example", testFunction: testMazeGetFinishCell, expectedOutput: testMazeGetFinishCellExpectedOutput },
    { name: "MazeWasm:set_wall_cells() example", testFunction: testMazeSetWallCells, expectedOutput: testMazeSetWallCellsExpectedOutput },
    { name: "MazeWasm:clear_cells() example", testFunction: testMazeClearCells, expectedOutput: testMazeClearCellsExpectedOutput },
    { name: "MazeWasm:to_json() example", testFunction: testMazeToJSON, expectedOutput: testMazeToJSONExpectedOutput },
    { name: "MazeWasm:from_json() example", testFunction: testMazeFromJSON, expectedOutput: testMazeFromJSONExpectedOutput },
    { name: "MazeWasm:solve() example", testFunction: testMazeSolve, expectedOutput: testMazeSolveExpectedOutput },
    { name: "MazeWasm:generate() example", testFunction: testMazeGenerate, expectedOutput: testMazeGenerateExpectedOutput },
    { name: "MazeSolutionWasm:get_path_points() example", testFunction: testMazeSolutionGetPathPoints, expectedOutput: testMazeSolutionGetPathPointsExpectedOutput },
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