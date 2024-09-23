import { readFile } from 'fs/promises';
import init, { MazeWasm, MazeCellTypeWasm } from '../../pkg/maze_wasm.js';

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

// Test MazeWasm::is_empty() example
function testMazeIsEmpty() {
    try {
        let maze = new MazeWasm();
        console.log("Ater creation, is_empty() = ", maze.is_empty());
        maze.resize(1, 2);
        console.log("Ater resize(), is_empty() = ", maze.is_empty());
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    }
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

// Test MazeWasm::from_json() example
function testMazeFromJSON() {
    try {
        let maze = new MazeWasm();
        maze.from_json(`{
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

// Test MazeWasm::solve() example
function testMazeSolve() {
    try {
        let maze = new MazeWasm();
        maze.from_json(`{
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
        let solution_points = solution.get_path_points();
        console.log("Maze solve() succeeded. Solution points are: ", solution_points);
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    }
}

// Test MazeSolutionWasm::get_path_points() example
function testMazeSolutionGetPathPoints() {
    try {
        let maze = new MazeWasm();
        maze.from_json(`{
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
        let solution_points = solution.get_path_points();
        console.log("Successfully solved maze. Solution points are: ", solution_points);
        return true;
    } catch (e) {
        console.error("Operation failed: ", e);
        return false;
    }
}

// Tests
const tests = [
    { name: "MazeWasm:new() example", testFunction: testMazeNew },
    { name: "MazeWasm:reset() example", testFunction: testMazeReset },
    { name: "MazeWasm:resize() example", testFunction: testMazeResize },
    { name: "MazeWasm:insert_rows() example", testFunction: testMazeInsertRows },
    { name: "MazeWasm:delete_rows() example", testFunction: testMazeDeleteRows },
    { name: "MazeWasm:insert_cols() example", testFunction: testMazeInsertCols },
    { name: "MazeWasm:delete_cols() example", testFunction: testMazeDeleteCols },
    { name: "MazeWasm:is_empty() example", testFunction: testMazeIsEmpty },
    { name: "MazeWasm:get_row_count() example", testFunction: testMazeGetRowCount },
    { name: "MazeWasm:get_col_count() example", testFunction: testMazeGetColCount },
    { name: "MazeWasm:get_cell() example", testFunction: testMazeGetCell },
    { name: "MazeWasm:set_start_cell() example", testFunction: testMazeSetStartCell },
    { name: "MazeWasm:get_start_cell() example", testFunction: testMazeGetStartCell },
    { name: "MazeWasm:set_finish_cell() example", testFunction: testMazeSetFinishCell },
    { name: "MazeWasm:get_finish_cell() example", testFunction: testMazeGetFinishCell },
    { name: "MazeWasm:set_wall_cells() example", testFunction: testMazeSetWallCells },
    { name: "MazeWasm:clear_cells() example", testFunction: testMazeClearCells },
    { name: "MazeWasm:to_json() example", testFunction: testMazeToJSON },
    { name: "MazeWasm:from_json() example", testFunction: testMazeFromJSON },
    { name: "MazeWasm:solve() example", testFunction: testMazeSolve },
    { name: "MazeSolutionWasm:get_path_points() example", testFunction: testMazeSolutionGetPathPoints },
];

// Function to run all tests
function runTests(hide_results) {
    const originalConsoleLog = console.log;

    let enableConsoleLog = function () {
        console.log = originalConsoleLog;
    };

    let disableConsoleLog = function () {
        console.log = function () { };
    };

    let allSucceeded = true;
    for (let i = 0; i < tests.length; i++) {
        const test = tests[i];
        enableConsoleLog();
        console.log(`Running test ${i + 1} of ${tests.length} => ${test.name}...`);
        if (hide_results)
            disableConsoleLog();
        const result = test.testFunction();
        if (!result) {
            console.error(`Test "${test.name}" failed.`);
            allSucceeded = false;
        }
    }
    enableConsoleLog();
    return allSucceeded;
}

async function run_tests(hide_results) {
    await loadWasm();
    return runTests(hide_results);
}

export { run_tests };