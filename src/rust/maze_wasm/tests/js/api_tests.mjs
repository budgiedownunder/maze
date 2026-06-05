// This file exports a single function run_tests() which runs the tests for
// the maze_wasm JavaScript API, using 'mocha' and 'chai'.
// 
import { readFile } from 'fs/promises';
import init, { DirectionWasm, MazeGameWasm, MazeWasm, MazeCellTypeWasm, MoveResultWasm, GenerationAlgorithmWasm } from '../../pkg/maze_wasm.js';
import Mocha from 'mocha';
import { expect } from 'chai';

// Custom function to handle loading WASM in Node.js
async function loadWasm() {
    const wasmBuffer = await readFile('../../pkg/maze_wasm_bg.wasm');
    await init({ module_or_path: wasmBuffer });
}

function invalidArgumentError(name, expected, provided) {
    return `invalid '${name}' argument provided - expected '${expected}' but '${provided}' provided`
}

function invalidJSONStringArgumentError(typeProvided) {
    return invalidArgumentError("json_string", "string", typeProvided);
}

function argmentTooLargeError(name, value) {
    return `invalid '${name}' (${value}) - too large`;
}

function indexOutOfBoundsError(name, index) {
    return `invalid '${name}' index (${index})`;
}

function eofParsingValueError() {
    return "EOF while parsing a value at line 1 column 0";
}
function eofParsingObjectError() {
    return "EOF while parsing an object at line 1 column 1"
}

function missingFieldError(field, line, column) {
    return `missing field \`${field}\` at line ${line} column ${column}`;
}

function expectedValueError(line, column) {
    return `expected value at line ${line} column ${column}`;
}

function trailingCommaError(line, column) {
    return `trailing comma at line ${line} column ${column}`;
}

function expectedTokenError(token, line, column) {
    return `expected \`${token}\` at line ${line} column ${column}`;
}

function noCellDefinedError(name) {
    return `no ${name} cell defined`;
}

function noCellFoundError(name) {
    return `no ${name} cell found within maze`;
}

function invalidPointError(name, row, column) {
    return `invalid '${name}' point [${row}, ${column}]`;
}

function generateRowCountError() {
    return "row_count must be at least 3";
}

function generateColCountError() {
    return "col_count must be at least 3";
}

function generateStartOutOfBoundsError() {
    return "start is out of bounds";
}

function generateFinishOutOfBoundsError() {
    return "finish is out of bounds";
}

function runBadArgTests(callback) {
    let argTests = [
        { value: undefined, desc: "undefined" },
        { value: null, desc: "unknown" },
        { value: -1, desc: "negative number" },
        { value: "some_text", desc: "string" },
        { value: true, desc: "boolean" },
        { value: {}, desc: "object" }
    ];

    for (let i = 0; i < argTests.length; i++) {
        callback(argTests[i]);
    }
}

function runBadOptArgTests(callback) {
    let argTests = [
        { value: -1, desc: "negative number" },
        { value: "some_text", desc: "string" },
        { value: true, desc: "boolean" },
        { value: {}, desc: "object" }
    ];

    for (let i = 0; i < argTests.length; i++) {
        callback(argTests[i]);
    }
}

function verifyCellType(maze, startRow, startCol, endRow, endCol, cellType) {
    for (let row = startRow; row <= endRow; row++) {
        for (let col = startCol; col <= endCol; col++) {
            let cellInfo = maze.get_cell(row, col);
            expect(cellInfo.cell_type).to.equal(cellType);
        }
    }
}

// ── wasm object lifetime tracking ──────────────────────────────────────────
// wasm-bindgen objects (MazeWasm / MazeGameWasm / MazeSolutionWasm) own an
// internal wasm pointer that must be released with .free(); otherwise it leaks
// until GC / process exit. Tests register every object they create via track()
// (or makeGame() for game sessions), and an afterEach() in each suite frees them.
let trackedWasmObjects = [];
function track(obj) {
    trackedWasmObjects.push(obj);
    return obj;
}
// Bound reference avoids the literal `makeGame(` (kept distinct so
// makeGame's own body isn't a game session it would try to free recursively).
const boundGameFromJson = MazeGameWasm.from_json.bind(MazeGameWasm);
function makeGame(json) {
    return track(boundGameFromJson(json));
}
function freeTrackedWasmObjects() {
    while (trackedWasmObjects.length > 0) {
        const obj = trackedWasmObjects.pop();
        if (obj) {
            try { obj.free(); } catch { /* already freed */ }
        }
    }
}

function registerMazeTests() {
    describe('MazeWasm API', function () {
        afterEach(freeTrackedWasmObjects);

        // MazeWasm::new()
        it('should successfully create a new maze', function () {
            expect(() => track(new MazeWasm())).to.not.throw();
        });

        // MazeWasm::is_empty()
        it('should expect is_empty() to return true for a new maze', function () {
            expect(track(new MazeWasm()).is_empty()).to.equal(true);
        });

        // MazeWasm::get_row_count()
        it('should expect get_row_count() to return zero for a new maze', function () {
            expect(track(new MazeWasm()).get_row_count()).to.equal(0);
        });

        // MazeWasm::get_col_count()
        it('should expect get_col_count() to return zero for a new maze', function () {
            expect(track(new MazeWasm()).get_col_count()).to.equal(0);
        });

        // MazeWasm::from_json()
        it('should expect from_json() to fail if provided with a numeric argument', function () {
            expect(() => track(new MazeWasm()).from_json(1)).to.throw(invalidJSONStringArgumentError('number'));
        });

        it('should expect from_json() to fail if provided with a empty object argument', function () {
            expect(() => track(new MazeWasm()).from_json({})).to.throw(invalidJSONStringArgumentError('object'));
        });

        it('should expect from_json() to fail if provided with a boolean argument', function () {
            expect(() => track(new MazeWasm()).from_json(true)).to.throw(invalidJSONStringArgumentError('boolean'));
        });

        it('should expect from_json() to fail if provided with a null argument', function () {
            expect(() => track(new MazeWasm()).from_json(null)).to.throw(invalidJSONStringArgumentError('unknown'));
        });

        it('should expect from_json() to fail if provided with an undefined argument', function () {
            expect(() => track(new MazeWasm()).from_json(undefined)).to.throw(invalidJSONStringArgumentError('undefined'));
        });

        it('should expect from_json() to fail if provided with an empty string argument', function () {
            expect(() => track(new MazeWasm()).from_json("")).to.throw(eofParsingValueError());
        });

        it('should expect from_json() to fail if provided with a string argument with a missing object close', function () {
            expect(() => track(new MazeWasm()).from_json("{")).to.throw(eofParsingObjectError());
        });

        it('should expect from_json() to fail if provided with a string argument with a missing id field', function () {
            expect(() => track(new MazeWasm()).from_json("{}")).to.throw(missingFieldError("id", 1, 2));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing name field', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id"}`)).to.throw(missingFieldError("name", 1, 16));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing name field value', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id","name":}`)).to.throw(expectedValueError(1, 24));
        });

        it('should expect from_json() to fail if provided with a string argument with a trailing comma', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id","name":"test",}`)).to.throw(trailingCommaError(1, 31));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing colon token for definition value', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id","name":"test", "definition"}`)).to.throw(expectedTokenError(":", 1, 44));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing definition field value', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id","name":"test", "definition":}`)).to.throw(expectedValueError(1, 45));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing grid field', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id","name":"test", "definition":{}}`)).to.throw(missingFieldError("grid", 1, 47));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing colon token for grid field value', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id","name":"test", "definition":{"grid"}}`)).to.throw(expectedTokenError(":", 1, 52));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing grid value', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id","name":"test", "definition":{"grid":}}`)).to.throw(expectedValueError(1, 53));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing grid value closing array bracket', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id","name":"test", "definition":{"grid":[}}`)).to.throw(expectedValueError(1, 54));
        });

        it('should expect from_json() to succeed if provided with a valid string argument with an empty array for the grid value', function () {
            expect(() => track(new MazeWasm()).from_json(`{"id":"maze_id","name":"test", "definition":{"grid":[]}}`)).to.not.throw();
        });

        // MazeWasm::resize()
        it('should expect resize() to modify number of rows and columns in a maze', function () {
            let maze = track(new MazeWasm());
            let oldIsEmpty = maze.is_empty();
            let oldRowCount = maze.get_row_count();
            let oldColCount = maze.get_col_count();
            maze.resize(10, 5);
            let newIsEmpty = maze.is_empty();
            let newRowCount = maze.get_row_count();
            let newColCount = maze.get_col_count();

            expect((oldIsEmpty == true) && (oldRowCount == 0) && (oldColCount == 0) && (newIsEmpty == false) &&
                (newRowCount == 10) && (newColCount == 5)).to.equal(true);
        });

        // MazeWasm::reset()
        it('should expect reset() to clear all rows and columns in a maze', function () {
            let maze = track(new MazeWasm());
            maze.resize(10, 5);
            let oldIsEmpty = maze.is_empty();
            let oldRowCount = maze.get_row_count();
            let oldColCount = maze.get_col_count();
            maze.reset();
            let newIsEmpty = maze.is_empty();
            let newRowCount = maze.get_row_count();
            let newColCount = maze.get_col_count();

            expect((oldIsEmpty == false) && (oldRowCount == 10) && (oldColCount == 5) && (newIsEmpty == true) &&
                (newRowCount == 0) && (newColCount == 0)).to.equal(true);
        });

        // MazeWasm::get_start_cell()
        it('should expect get_start_cell() to fail for a new maze', function () {
            expect(() => track(new MazeWasm()).get_start_cell()).to.throw(noCellDefinedError("start"));
        });

        // MazeWasm::set_start_cell()
        runBadArgTests(function (argTest) {
            it(`should expect set_start_cell() to fail for a maze when passed an invalid 'start_row' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.set_start_cell(argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        it('should expect set_start_cell() to fail for a new maze when all arguments supplied', function () {
            expect(() => track(new MazeWasm()).set_start_cell(1, 1)).to.throw(invalidPointError("start", 1, 1));
        });

        it('should expect set_start_cell() to succeed for a valid maze point and get_start_cell() should then return that cell', function () {
            let maze = track(new MazeWasm());
            maze.resize(10, 5);
            maze.set_start_cell(0, 1);
            expect(maze.get_start_cell()).to.deep.equal({ row: 0, col: 1 });
        });

        // MazeWasm::get_finish_cell()
        it('should expect get_finish_cell() to fail for a new maze', function () {
            expect(() => track(new MazeWasm()).get_finish_cell()).to.throw(noCellDefinedError("finish"));
        });

        // MazeWasm::set_finish_cell()
        it('should expect set_finish_cell() to fail for a new maze', function () {
            expect(() => track(new MazeWasm()).set_finish_cell(1, 1)).to.throw(invalidPointError("finish", 1, 1));
        });

        it('should expect set_finish_cell() to succeed for a valid maze point and get_finish_cell() should then return that cell', function () {
            let maze = track(new MazeWasm());
            maze.resize(10, 5);
            maze.set_finish_cell(9, 4);
            expect(maze.get_finish_cell()).to.deep.equal({ row: 9, col: 4 });
        });

        // MazeWasm::get_cell()
        runBadArgTests(function (argTest) {
            it(`should expect get_cell() to fail for a maze when passed an invalid 'row' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.get_cell(argTest.value)).to.throw(invalidArgumentError("row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect get_cell() to fail for a maze when passed an invalid 'col' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.get_cell(1, argTest.value)).to.throw(invalidArgumentError("col", "unsigned integer", argTest.desc));
            });
        });

        it('should expect get_cell() to succeed for a maze with no cells set when passed a valid location and for the cell type to be empty', function () {
            let maze = track(new MazeWasm());
            maze.resize(2, 2);
            let cellType = maze.get_cell(1, 1)
            expect(cellType).to.deep.equal({ cell_type: 0 });
        });

        // MazeWasm::set_wall_cells() / set_key_cells() / set_door_cells() / set_enemy_cells() / set_health_cells()
        // Every typed setter shares one signature (start_row, start_col, end_row,
        // end_col) and one set of behaviours: identical bad-argument handling, the
        // same from/to out-of-bounds errors, and a write that get_cell()'s cell_type
        // and to_json()'s character both reflect. The single data-driven block below
        // covers all five so a new setter only needs a row here.
        [
            { method: 'set_wall_cells', cellType: MazeCellTypeWasm.Wall, char: 'W' },
            { method: 'set_key_cells', cellType: MazeCellTypeWasm.Key, char: 'K' },
            { method: 'set_door_cells', cellType: MazeCellTypeWasm.Door, char: 'D' },
            { method: 'set_enemy_cells', cellType: MazeCellTypeWasm.Enemy, char: 'E' },
            { method: 'set_health_cells', cellType: MazeCellTypeWasm.Health, char: 'H' },
        ].forEach(function (setter) {
            runBadArgTests(function (argTest) {
                it(`should expect ${setter.method}() to fail for a maze when passed invalid 'start_row' argument (${argTest.desc})`, function () {
                    let maze = track(new MazeWasm());
                    maze.resize(2, 2);
                    expect(() => maze[setter.method](argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
                });
            });

            runBadArgTests(function (argTest) {
                it(`should expect ${setter.method}() to fail for a maze when passed invalid 'start_col' argument (${argTest.desc})`, function () {
                    let maze = track(new MazeWasm());
                    maze.resize(2, 2);
                    expect(() => maze[setter.method](0, argTest.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", argTest.desc));
                });
            });

            runBadArgTests(function (argTest) {
                it(`should expect ${setter.method}() to fail for a maze when passed invalid 'end_row' argument (${argTest.desc})`, function () {
                    let maze = track(new MazeWasm());
                    maze.resize(2, 2);
                    expect(() => maze[setter.method](0, 0, argTest.value)).to.throw(invalidArgumentError("end_row", "unsigned integer", argTest.desc));
                });
            });

            runBadArgTests(function (argTest) {
                it(`should expect ${setter.method}() to fail for a maze when passed invalid 'end_col' argument (${argTest.desc})`, function () {
                    let maze = track(new MazeWasm());
                    maze.resize(2, 2);
                    expect(() => maze[setter.method](0, 0, 0, argTest.value)).to.throw(invalidArgumentError("end_col", "unsigned integer", argTest.desc));
                });
            });

            it(`should expect ${setter.method}() to fail for a maze when passed out of bounds 'start_row' argument`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze[setter.method](2, 0, 0, 0)).to.throw(invalidPointError("from", 2, 0));
            });

            it(`should expect ${setter.method}() to fail for a maze when passed out of bounds 'start_col' argument`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze[setter.method](1, 2, 0, 0)).to.throw(invalidPointError("from", 1, 2));
            });

            it(`should expect ${setter.method}() to fail for a maze when passed out of bounds 'end_row' argument`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze[setter.method](1, 1, 2, 0)).to.throw(invalidPointError("to", 2, 0));
            });

            it(`should expect ${setter.method}() to fail for a maze when passed out of bounds 'end_col' argument`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze[setter.method](1, 1, 1, 2)).to.throw(invalidPointError("to", 1, 2));
            });

            it(`should expect ${setter.method}() to succeed for valid arguments and for get_cell() to return the correct cell_type before/after`, function () {
                let maze = track(new MazeWasm());
                maze.resize(3, 3);
                let startRow = 1, startCol = 1, endRow = 2, endCol = 2;
                verifyCellType(maze, startRow, startCol, endRow, endCol, MazeCellTypeWasm.Empty);
                maze[setter.method](startRow, startCol, endRow, endCol);
                verifyCellType(maze, startRow, startCol, endRow, endCol, setter.cellType);
            });

            it(`should expect ${setter.method}() to write the '${setter.char}' character into the grid as seen via to_json()`, function () {
                let maze = track(new MazeWasm());
                maze.resize(3, 3);
                maze[setter.method](1, 1, 1, 1);
                let parsed = JSON.parse(maze.to_json());
                expect(parsed.definition.grid[1][1]).to.equal(setter.char);
            });
        });

        // MazeWasm::clear_cells()
        runBadArgTests(function (argTest) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'start_row' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.clear_cells(argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'start_col' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.clear_cells(0, argTest.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'end_row' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.clear_cells(0, 0, argTest.value)).to.throw(invalidArgumentError("end_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'end_col' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.clear_cells(0, 0, 0, argTest.value)).to.throw(invalidArgumentError("end_col", "unsigned integer", argTest.desc));
            });
        });

        it('should expect clear_cells() to succeed for a new maze', function () {
            let maze = track(new MazeWasm());
            maze.resize(2, 2);
            let startRow = 0, startCol = 0, endRow = 1, endCol = 1;
            verifyCellType(maze, startRow, startCol, endRow, endCol, MazeCellTypeWasm.Empty);
            maze.set_wall_cells(startRow, startCol, endRow, endCol);
            verifyCellType(maze, startRow, startCol, endRow, endCol, MazeCellTypeWasm.Wall);
            maze.clear_cells(startRow, startCol, endRow, endCol);
            verifyCellType(maze, startRow, startCol, endRow, endCol, MazeCellTypeWasm.Empty);
        });

        // MazeWasm::delete_rows()
        runBadArgTests(function (argTest) {
            it(`should expect delete_rows() to fail for a maze when passed passed invalid 'start_row' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.delete_rows(argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect delete_rows() to fail for a maze when passed passed invalid 'count' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.delete_rows(0, argTest.value)).to.throw(invalidArgumentError("count", "unsigned integer", argTest.desc));
            });
        });

        it(`should expect delete_rows() to fail if 'start_row' out of bounds`, function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 3);
            expect(() => maze.delete_rows(3, 4)).to.throw(indexOutOfBoundsError("start_row", 3));
        });

        it(`should expect delete_rows() to fail if too large 'count' is supplied`, function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 2);
            expect(() => maze.delete_rows(1, 3)).to.throw(argmentTooLargeError("count", 3));
        });

        it('should expect delete_rows() to succeed for valid arguments and for get_row_count() to return the updated row count', function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 3);
            let oldCount = maze.get_row_count();
            maze.delete_rows(1, 2);
            let newCount = maze.get_row_count();
            expect((oldCount == 3) && (newCount == 1)).to.equal(true);
        });

        // MazeWasm::insert_rows()
        runBadArgTests(function (argTest) {
            it(`should expect insert_rows() to fail for a maze when passed passed invalid 'start_row' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.insert_rows(argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect insert_rows() to fail for a maze when passed passed invalid 'count' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.insert_rows(0, argTest.value)).to.throw(invalidArgumentError("count", "unsigned integer", argTest.desc));
            });
        });

        it(`should expect insert_rows() to fail if 'start_row' out of bounds`, function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 3);
            expect(() => maze.insert_rows(4, 1)).to.throw(indexOutOfBoundsError("start_row", 4));
        });

        it('should expect insert_rows() to succeed when inserting between existing rows and for get_row_count() to return the updated row count', function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 3);
            let oldCount = maze.get_row_count();
            maze.insert_rows(1, 2);
            let newCount = maze.get_row_count();
            expect((oldCount == 3) && (newCount == 5)).to.equal(true);
        });

        it('should expect insert_rows() to allow insertion after last row and for get_row_count() to return the updated row count', function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 3);
            let oldCount = maze.get_row_count();
            maze.insert_rows(oldCount, 2);
            let newCount = maze.get_row_count();
            expect((oldCount == 3) && (newCount == 5)).to.equal(true);
        });

        // MazeWasm::delete_cols()
        runBadArgTests(function (argTest) {
            it(`should expect delete_cols() to fail for a maze when passed passed invalid 'start_col' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.delete_cols(argTest.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect delete_cols() to fail for a maze when passed passed invalid 'count' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.delete_cols(0, argTest.value)).to.throw(invalidArgumentError("count", "unsigned integer", argTest.desc));
            });
        });

        it(`should expect delete_cols() to fail if 'start_col' out of bounds`, function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 2);
            expect(() => maze.delete_cols(3, 4)).to.throw(indexOutOfBoundsError("start_col", 3));
        });

        it(`should expect delete_cols() to fail if too large 'count' is supplied`, function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 2);
            expect(() => maze.delete_cols(1, 3)).to.throw(argmentTooLargeError("count", 3));
        });

        it('should expect delete_cols() to succeed for valid arguments and for get_col_count() to return the updated column count', function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 2);
            let oldCount = maze.get_col_count();
            maze.delete_cols(1, 1);
            let newCount = maze.get_col_count();
            expect((oldCount == 2) && (newCount == 1)).to.equal(true);
        });

        // MazeWasm::insert_cols()
        runBadArgTests(function (argTest) {
            it(`should expect insert_cols() to fail for a maze when passed passed invalid 'start_col' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.insert_cols(argTest.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect insert_cols() to fail for a maze when passed passed invalid 'count' argument (${argTest.desc})`, function () {
                let maze = track(new MazeWasm());
                maze.resize(2, 2);
                expect(() => maze.insert_cols(0, argTest.value)).to.throw(invalidArgumentError("count", "unsigned integer", argTest.desc));
            });
        });

        it(`should expect insert_cols() to fail if 'start_col' out of bounds`, function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 3);
            expect(() => maze.insert_cols(4, 1)).to.throw(indexOutOfBoundsError("start_col", 4));
        });

        it('should expect insert_cols() to succeed when inserting between existing columns and for get_col_count() to return the updated column count', function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 3);
            let oldCount = maze.get_col_count();
            maze.insert_cols(1, 2);
            let newCount = maze.get_col_count();
            expect((oldCount == 3) && (newCount == 5)).to.equal(true);
        });

        it('should expect insert_cols() to allow insertion after last column and for get_col_count() to return the updated row count', function () {
            let maze = track(new MazeWasm());
            maze.resize(3, 3);
            let oldCount = maze.get_col_count();
            maze.insert_cols(oldCount, 2);
            let newCount = maze.get_col_count();
            expect((oldCount == 3) && (newCount == 5)).to.equal(true);
        });

        // MazeWasm::solve()
        it('should expect solve() to fail for a new maze', function () {
            let maze = track(new MazeWasm());
            expect(() => track(maze.solve())).to.throw(noCellFoundError("start"));
        });

        it('should expect solve() to fail for a resized maze with no start cell set', function () {
            let maze = track(new MazeWasm());
            maze.resize(10, 5);
            expect(() => track(maze.solve())).to.throw(noCellFoundError("start"));
        });

        it('should expect solve() to fail for a resized maze with no finish cell set', function () {
            let maze = track(new MazeWasm());
            maze.resize(10, 5);
            maze.set_start_cell(0, 0);
            expect(() => track(maze.solve())).to.throw(noCellFoundError("finish"));
        });

        it('should expect solve() to succeed for a resized maze with start and finish cells set and for get_path_points() to return expected path', function () {
            let maze = track(new MazeWasm());
            maze.resize(10, 5);
            maze.set_start_cell(0, 0);
            maze.set_finish_cell(9, 4);
            let solution = track(maze.solve());
            expect(solution.get_path_points()).to.deep.equal([
                { row: 0, col: 0 },
                { row: 0, col: 1 },
                { row: 0, col: 2 },
                { row: 0, col: 3 },
                { row: 0, col: 4 },
                { row: 1, col: 4 },
                { row: 2, col: 4 },
                { row: 3, col: 4 },
                { row: 4, col: 4 },
                { row: 5, col: 4 },
                { row: 6, col: 4 },
                { row: 7, col: 4 },
                { row: 8, col: 4 },
                { row: 9, col: 4 }
            ]);
        });

        // MazeWasm::generate()
        runBadArgTests(function (argTest) {
            it(`should expect generate() to fail when passed an invalid 'row_count' argument (${argTest.desc})`, function () {
                expect(() => track(new MazeWasm()).generate(argTest.value, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                    undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("row_count", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when row_count is less than 3', function () {
            expect(() => track(new MazeWasm()).generate(2, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.throw(generateRowCountError());
        });

        runBadArgTests(function (argTest) {
            it(`should expect generate() to fail when passed an invalid 'col_count' argument (${argTest.desc})`, function () {
                expect(() => track(new MazeWasm()).generate(7, argTest.value, GenerationAlgorithmWasm.RecursiveBacktracking,
                    undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("col_count", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when col_count is less than 3', function () {
            expect(() => track(new MazeWasm()).generate(7, 2, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.throw(generateColCountError());
        });

        runBadOptArgTests(function (argTest) {
            it(`should expect generate() to fail when passed an invalid 'start_row' argument (${argTest.desc})`, function () {
                expect(() => track(new MazeWasm()).generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                    argTest.value, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when start point is out of bounds', function () {
            expect(() => track(new MazeWasm()).generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                10, 0, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.throw(generateStartOutOfBoundsError());
        });

        it('should expect generate() to succeed with a valid explicit start point', function () {
            let maze = track(new MazeWasm());
            expect(() => maze.generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                0, 0, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.not.throw();
            expect(maze.get_start_cell()).to.deep.equal({ row: 0, col: 0 });
        });

        runBadOptArgTests(function (argTest) {
            it(`should expect generate() to fail when passed an invalid 'finish_row' argument (${argTest.desc})`, function () {
                expect(() => track(new MazeWasm()).generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                    undefined, undefined, argTest.value, undefined, undefined, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("finish_row", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when finish point is out of bounds', function () {
            expect(() => track(new MazeWasm()).generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, 10, 0, undefined, undefined, undefined, undefined))
                .to.throw(generateFinishOutOfBoundsError());
        });

        it('should expect generate() to succeed with a valid explicit finish point', function () {
            let maze = track(new MazeWasm());
            expect(() => maze.generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, 6, 4, undefined, undefined, undefined, undefined))
                .to.not.throw();
            expect(maze.get_finish_cell()).to.deep.equal({ row: 6, col: 4 });
        });

        runBadOptArgTests(function (argTest) {
            it(`should expect generate() to fail when passed an invalid 'min_spine_length' argument (${argTest.desc})`, function () {
                expect(() => track(new MazeWasm()).generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                    undefined, undefined, undefined, undefined, argTest.value, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("min_spine_length", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when min_spine_length is impossible to satisfy', function () {
            expect(() => track(new MazeWasm()).generate(3, 3, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, 1000, 1, undefined, undefined))
                .to.throw();
        });

        it('should expect generate() to succeed with a valid min_spine_length', function () {
            let maze = track(new MazeWasm());
            expect(() => maze.generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, 3, undefined, undefined, undefined))
                .to.not.throw();
            expect(maze.get_row_count()).to.equal(7);
            expect(maze.get_col_count()).to.equal(5);
        });

        it('should expect generate() to succeed with valid row_count and col_count and return a maze of the correct dimensions', function () {
            let maze = track(new MazeWasm());
            expect(() => maze.generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.not.throw();
            expect(maze.get_row_count()).to.equal(7);
            expect(maze.get_col_count()).to.equal(5);
        });

        // MazeWasm::get_cell_entity() / set_cell_entity() / clear_cell_entity()
        // The per-cell entity override read/write surface. One data-driven block
        // covers all four entity types: set the matching cell character, write an
        // override, read it back, round-trip via to_json()/from_json(), and clear.
        [
            { setCells: 'set_enemy_cells', char: 'E', entity: { type: 'E', enemyType: 'ghost', damage: 2, movePeriodMs: 800 } },
            { setCells: 'set_health_cells', char: 'H', entity: { type: 'H', healthStyle: 'potion', healAmount: 3 } },
            { setCells: 'set_key_cells', char: 'K', entity: { type: 'K', keyHolder: 'chest' } },
            { setCells: 'set_door_cells', char: 'D', entity: { type: 'D', doorStyle: 'portcullis' } },
        ].forEach(function (t) {
            it(`should expect get_cell_entity() to return null for a ${t.char} cell with no override`, function () {
                let maze = track(new MazeWasm());
                maze.resize(1, 3);
                maze[t.setCells](0, 1, 0, 1);
                expect(maze.get_cell_entity(0, 1)).to.equal(null);
            });

            it(`should expect set_cell_entity() + get_cell_entity() to round-trip a ${t.char} override`, function () {
                let maze = track(new MazeWasm());
                maze.resize(1, 3);
                maze[t.setCells](0, 1, 0, 1);
                maze.set_cell_entity(0, 1, t.entity);
                expect(maze.get_cell_entity(0, 1)).to.deep.equal(t.entity);
            });

            it(`should expect a ${t.char} override to survive a to_json()/from_json() round-trip`, function () {
                let maze = track(new MazeWasm());
                maze.resize(1, 3);
                maze[t.setCells](0, 1, 0, 1);
                maze.set_cell_entity(0, 1, t.entity);
                let reloaded = track(new MazeWasm());
                reloaded.from_json(maze.to_json());
                expect(reloaded.get_cell_entity(0, 1)).to.deep.equal(t.entity);
            });

            it(`should expect clear_cell_entity() to remove a ${t.char} override`, function () {
                let maze = track(new MazeWasm());
                maze.resize(1, 3);
                maze[t.setCells](0, 1, 0, 1);
                maze.set_cell_entity(0, 1, t.entity);
                maze.clear_cell_entity(0, 1);
                expect(maze.get_cell_entity(0, 1)).to.equal(null);
            });

            it(`should expect set_cell_entity() to fail when the ${t.char} entity type does not match the cell character`, function () {
                let maze = track(new MazeWasm());
                maze.resize(1, 3);
                // Cell (0,1) is left empty, so the typed entity must be rejected.
                expect(() => maze.set_cell_entity(0, 1, t.entity)).to.throw();
            });
        });

        it('should expect to_json() to emit an overridden cell in the array-of-one form', function () {
            let maze = track(new MazeWasm());
            maze.resize(1, 3);
            maze.set_enemy_cells(0, 1, 0, 1);
            maze.set_cell_entity(0, 1, { type: 'E', damage: 2 });
            let parsed = JSON.parse(maze.to_json());
            expect(parsed.definition.grid[0][1]).to.deep.equal([{ type: 'E', damage: 2 }]);
        });

        it('should expect an override-less cell to stay a bare character in to_json()', function () {
            let maze = track(new MazeWasm());
            maze.resize(1, 3);
            maze.set_enemy_cells(0, 1, 0, 1);
            let parsed = JSON.parse(maze.to_json());
            expect(parsed.definition.grid[0][1]).to.equal('E');
        });

        it('should expect get_cell_entity() to fail when out of bounds', function () {
            let maze = track(new MazeWasm());
            maze.resize(1, 3);
            expect(() => maze.get_cell_entity(5, 0)).to.throw('row out of bounds');
            expect(() => maze.get_cell_entity(0, 5)).to.throw('column out of bounds');
        });

    });
}

function registerMazeSolutionTests() {
    describe('MazeSolutionWasm API', function () {
        afterEach(freeTrackedWasmObjects);

        // MazeSolutionWasm::get_path_points()
        it('should expect get_path_points() to return expected path following a successful solve()', function () {
            let maze = track(new MazeWasm());
            maze.resize(10, 5);
            maze.set_start_cell(0, 0);
            maze.set_finish_cell(9, 4);
            let solution = track(maze.solve());
            expect(solution.get_path_points()).to.deep.equal([
                { row: 0, col: 0 },
                { row: 0, col: 1 },
                { row: 0, col: 2 },
                { row: 0, col: 3 },
                { row: 0, col: 4 },
                { row: 1, col: 4 },
                { row: 2, col: 4 },
                { row: 3, col: 4 },
                { row: 4, col: 4 },
                { row: 5, col: 4 },
                { row: 6, col: 4 },
                { row: 7, col: 4 },
                { row: 8, col: 4 },
                { row: 9, col: 4 }
            ]);
        });
    });
}

function registerMazeGameTests() {
    describe('MazeGame API', function () {
        afterEach(freeTrackedWasmObjects);

        // MazeGame::from_json()
        it('should expect from_json() to throw on invalid JSON', function () {
            expect(() => makeGame("")).to.throw();
        });

        it('should expect from_json() to throw on a maze with no start cell', function () {
            expect(() => makeGame('{"grid":[[" "," ","F"]]}')).to.throw(/no start cell/);
        });

        it('should expect from_json() to succeed with a valid maze JSON string', function () {
            expect(() => makeGame('{"grid":[["S"," ","F"]]}')).to.not.throw();
        });

        // MazeGame::player_row()
        it('should expect player_row() to return 0 after from_json()', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.player_row()).to.equal(0);
        });

        // MazeGame::player_col()
        it('should expect player_col() to return 0 after from_json()', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.player_col()).to.equal(0);
        });

        // MazeGame::player_direction()
        it('should expect player_direction() to return DirectionWasm.None after from_json()', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.player_direction()).to.equal(DirectionWasm.None);
        });

        // MazeGame::is_complete()
        it('should expect is_complete() to return false after from_json()', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.is_complete()).to.equal(false);
        });

        // MazeGame::visited_cells()
        it('should expect visited_cells() to contain only the start cell after from_json()', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.visited_cells()).to.deep.equal([{ row: 0, col: 0 }]);
        });

        // MazeGame::move_player() — move into empty cell
        it('should expect move_player(DirectionWasm.Right) to return MoveResultWasm.Moved when moving into an empty cell', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Moved);
        });

        // MazeGame::move_player() — move into wall
        it('should expect move_player(DirectionWasm.Right) to return MoveResultWasm.Blocked when moving into a wall', function () {
            let game = makeGame('{"grid":[["S","W","F"]]}');
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Blocked);
        });

        // MazeGame::move_player() — out-of-bounds move
        it('should expect move_player(DirectionWasm.Up) to return MoveResultWasm.Blocked when moving out of bounds', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.move_player(DirectionWasm.Up)).to.equal(MoveResultWasm.Blocked);
        });

        // MazeGame::move_player() — reach finish
        it('should expect move_player(DirectionWasm.Right) to return MoveResultWasm.Complete when moving into the finish cell', function () {
            let game = makeGame('{"grid":[["S","F"]]}');
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Complete);
        });

        // MazeGame::move_player() — DirectionWasm.None
        it('should expect move_player(DirectionWasm.None) to return MoveResultWasm.None', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.move_player(DirectionWasm.None)).to.equal(MoveResultWasm.None);
        });

        // MazeGame::player_direction() — updates after move
        it('should expect player_direction() to return DirectionWasm.Right after moving right', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.player_direction()).to.equal(DirectionWasm.Right);
        });

        // MazeGame::player_direction() — updates even after blocked move
        it('should expect player_direction() to update even after a blocked move', function () {
            let game = makeGame('{"grid":[["S","W","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.player_direction()).to.equal(DirectionWasm.Right);
        });

        // MazeGame::visited_cells() — grows after successful move
        it('should expect visited_cells() to grow after a successful move', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.visited_cells()).to.deep.equal([
                { row: 0, col: 0 },
                { row: 0, col: 1 }
            ]);
        });

        // MazeGame::visited_cells() — unchanged after blocked move
        it('should expect visited_cells() to not change after a blocked move', function () {
            let game = makeGame('{"grid":[["S","W","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.visited_cells()).to.deep.equal([{ row: 0, col: 0 }]);
        });

        // MazeGame::visited_cells() — finish cell included on complete
        it('should expect visited_cells() to include the finish cell when the game is complete', function () {
            let game = makeGame('{"grid":[["S","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.visited_cells()).to.deep.equal([
                { row: 0, col: 0 },
                { row: 0, col: 1 }
            ]);
        });

        // MazeGame::is_complete() — true after reaching finish
        it('should expect is_complete() to return true after reaching the finish cell', function () {
            let game = makeGame('{"grid":[["S","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.is_complete()).to.equal(true);
        });

        // MazeGame::keys() — lists uncollected key cells
        it('should expect keys() to list uncollected key cells with ids', function () {
            let game = makeGame('{"grid":[["S","K","F"]]}');
            expect(game.keys()).to.deep.equal([{ row: 0, col: 1, id: 0 }]);
        });

        // MazeGame::bag() — empty initially
        it('should expect bag() to be empty after from_json()', function () {
            let game = makeGame('{"grid":[["S","K","F"]]}');
            expect(game.bag()).to.deep.equal([]);
        });

        // MazeGame::move_player() — moving onto a key auto-collects it
        it('should expect move_player onto a key cell to auto-collect it into the bag and clear keys()', function () {
            let game = makeGame('{"grid":[["S","K","F"]]}');
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Moved);
            expect(game.bag()).to.deep.equal([{ type: 'key', id: 0 }]);
            expect(game.keys()).to.deep.equal([]);
        });

        // MazeGame::move_player() — auto-collecting a key queues a keyCollected event
        it('should expect move_player onto a key cell to queue a keyCollected event for the next tick', function () {
            let game = makeGame('{"grid":[["S","K","F"]]}');
            game.move_player(DirectionWasm.Right); // onto the key — auto-collected
            expect(game.tick(0)).to.deep.equal([{ type: 'keyCollected', id: 0, row: 0, col: 1 }]);
        });

        // MazeGame::pickup() — null when not on a collectible
        it('should expect pickup() to return null when not standing on a key', function () {
            let game = makeGame('{"grid":[["S","K","F"]]}');
            expect(game.pickup()).to.equal(null);
        });

        // MazeGame::pickup() — null after the key was auto-collected on walk-over
        it('should expect pickup() to return null after a key is auto-collected on walk-over', function () {
            let game = makeGame('{"grid":[["S","K","F"]]}');
            game.move_player(DirectionWasm.Right); // onto the key — auto-collected
            expect(game.pickup()).to.equal(null);
            expect(game.bag()).to.deep.equal([{ type: 'key', id: 0 }]);
            expect(game.keys()).to.deep.equal([]);
        });

        // MazeGame::doors() — locked initially
        it('should expect doors() to list door cells as locked initially', function () {
            let game = makeGame('{"grid":[["S","D","F"]]}');
            expect(game.doors()).to.deep.equal([{ row: 0, col: 1, state: 'locked' }]);
        });

        // MazeGame::move_player() — locked door without a key
        it('should expect move_player into a locked door without a key to return BlockedByLockedDoor', function () {
            let game = makeGame('{"grid":[["S","D","F"]]}');
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.BlockedByLockedDoor);
        });

        // MazeGame::move_player() — locked door with a key begins unlocking
        it('should expect move_player into a locked door while holding a key to return StartedUnlocking', function () {
            let game = makeGame('{"grid":[["S","K","D","F"]]}');
            game.move_player(DirectionWasm.Right); // onto the key — auto-collected
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.StartedUnlocking);
            expect(game.doors()).to.deep.equal([{ row: 0, col: 2, state: 'opening' }]);
        });

        // MazeGame::tick() — no events when nothing is opening
        it('should expect tick() to return no events when no door is opening', function () {
            let game = makeGame('{"grid":[["S","D","F"]]}');
            expect(game.tick(1000)).to.deep.equal([]);
        });

        // MazeGame::tick() — opens an opening door after the countdown
        it('should expect tick() to open an opening door and emit a doorOpened event', function () {
            let game = makeGame('{"grid":[["S","K","D","F"]]}');
            game.move_player(DirectionWasm.Right); // onto the key — auto-collected
            game.tick(0);                           // flush the keyCollected event
            game.move_player(DirectionWasm.Right); // StartedUnlocking
            expect(game.tick(1000)).to.deep.equal([{ type: 'doorOpened', row: 0, col: 2 }]);
            expect(game.doors()).to.deep.equal([{ row: 0, col: 2, state: 'open' }]);
        });

        // MazeGame — an opened door becomes passable and the maze completable
        it('should expect an opened door to be passable (Moved) and allow completing the maze', function () {
            let game = makeGame('{"grid":[["S","K","D","F"]]}');
            game.move_player(DirectionWasm.Right); // onto the key — auto-collected
            game.tick(0);                           // flush the keyCollected event
            game.move_player(DirectionWasm.Right); // StartedUnlocking
            game.tick(1000);                        // door opens
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Moved);
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Complete);
            expect(game.is_complete()).to.equal(true);
        });

        // MazeGame::hp() / max_hp()
        it('should expect hp() and max_hp() to both return 3 on a fresh game', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.hp()).to.equal(3);
            expect(game.max_hp()).to.equal(3);
        });

        // MazeGame::enemies() — empty array when grid has no enemy cells
        it('should expect enemies() to return an empty array when no enemy cells exist', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.enemies()).to.deep.equal([]);
        });

        // MazeGame::enemies() — one entry per 'E' cell with id 0, carrying the
        // resolved damage / movePeriodMs (defaults here; no rig override → no
        // enemyType field).
        it('should expect enemies() to return one entry with id 0 per E cell', function () {
            let game = makeGame('{"grid":[["S","E","F"]]}');
            expect(game.enemies()).to.deep.equal([
                { row: 0, col: 1, id: 0, damage: 1, movePeriodMs: 1500 },
            ]);
        });

        // MazeGame::enemies() — ids assigned in row-major scan order
        it('should expect enemies() ids to follow row-major scan order across rows', function () {
            let game = makeGame('{"grid":[["S"," ","E"],[" ","E"," "],["E"," ","F"]]}');
            expect(game.enemies()).to.deep.equal([
                { row: 0, col: 2, id: 0, damage: 1, movePeriodMs: 1500 },
                { row: 1, col: 1, id: 1, damage: 1, movePeriodMs: 1500 },
                { row: 2, col: 0, id: 2, damage: 1, movePeriodMs: 1500 },
            ]);
        });

        // MazeGame::enemies() — a per-cell enemy override surfaces its resolved
        // damage / movePeriodMs and its enemyType rig on the live enemy.
        it('should expect enemies() to surface a per-cell enemy override', function () {
            let game = makeGame('{"grid":[["S",[{"type":"E","enemyType":"ghost","damage":3,"movePeriodMs":600.0}],"F"]]}');
            expect(game.enemies()).to.deep.equal([
                { row: 0, col: 1, id: 0, damage: 3, movePeriodMs: 600, enemyType: 'ghost' },
            ]);
        });

        // MazeGame::health_pickups() — empty array when grid has no 'H' cells
        it('should expect health_pickups() to return an empty array when no H cells exist', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.health_pickups()).to.deep.equal([]);
        });

        // MazeGame::health_pickups() — one entry per 'H' cell with id 0
        it('should expect health_pickups() to return one entry with id 0 per H cell', function () {
            let game = makeGame('{"grid":[["S","H","F"]]}');
            expect(game.health_pickups()).to.deep.equal([{ row: 0, col: 1, id: 0 }]);
        });

        // MazeGame::health_pickups() — ids assigned in row-major scan order
        it('should expect health_pickups() ids to follow row-major scan order', function () {
            let game = makeGame('{"grid":[["S"," ","H"],[" "," "," "],["H"," ","F"]]}');
            expect(game.health_pickups()).to.deep.equal([
                { row: 0, col: 2, id: 0 },
                { row: 2, col: 0, id: 1 },
            ]);
        });

        // MazeGame::grid() — pure-char grid for the host renderer
        it('should expect grid() to return the pure-char grid', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.grid()).to.deep.equal([['S', ' ', 'F']]);
        });

        // MazeGame::grid() — overridden cells come back as their bare char (not the array form)
        it('should expect grid() to return chars for overridden cells', function () {
            let game = makeGame('{"grid":[["S",[{"type":"H","healthStyle":"potion"}],"F"]]}');
            expect(game.grid()).to.deep.equal([['S', 'H', 'F']]);
        });

        // MazeGame::cell_overrides() — empty when the maze has no overrides
        it('should expect cell_overrides() to return an empty array when no overrides exist', function () {
            let game = makeGame('{"grid":[["S","H","F"]]}');
            expect(game.cell_overrides()).to.deep.equal([]);
        });

        // MazeGame::cell_overrides() — surfaces a static per-cell override
        it('should expect cell_overrides() to surface a per-cell override', function () {
            let game = makeGame('{"grid":[["S",[{"type":"H","healthStyle":"potion"}],"F"]]}');
            expect(game.cell_overrides()).to.deep.equal([
                { row: 0, col: 1, entity: { type: 'H', healthStyle: 'potion' } },
            ]);
        });

        // Move into an enemy decrements hp by the per-enemy damage value
        it('should expect move into an enemy to decrement hp by the enemy damage', function () {
            let game = makeGame('{"grid":[["S","E","F"]]}');
            expect(game.hp()).to.equal(3);
            game.move_player(DirectionWasm.Right);
            expect(game.hp()).to.equal(2);
        });

        // Move into an enemy queues PlayerDamaged; tick flushes it
        it('should expect move into an enemy to queue a playerDamaged event flushed on the next tick', function () {
            let game = makeGame('{"grid":[["S","E","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.tick(1)).to.deep.equal([{ type: 'playerDamaged', hpAfter: 2 }]);
        });

        // Final-hit collision returns Killed + flips is_lost + lose_reason
        it('should expect move into the third enemy at hp 1 to return Killed and flip lose state to killed', function () {
            let game = makeGame('{"grid":[["S","E","E","E","F"]]}');
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Moved);
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Moved);
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Killed);
            expect(game.hp()).to.equal(0);
            expect(game.is_lost()).to.equal(true);
            expect(game.lose_reason()).to.equal('killed');
        });

        // Move onto 'H' below max HP auto-heals, clears the cell, fires playerHealed on next tick
        it('should expect move onto H below max hp to auto-heal, clear the cell, and fire playerHealed', function () {
            let game = makeGame('{"grid":[["S","E","H","F"]]}');
            game.move_player(DirectionWasm.Right); // onto enemy, hp 3 → 2
            game.move_player(DirectionWasm.Right); // onto H, hp 2 → 3, cell cleared
            expect(game.hp()).to.equal(3);
            expect(game.tick(1)).to.deep.equal([
                { type: 'playerDamaged', hpAfter: 2 },
                { type: 'playerHealed', hpAfter: 3, row: 0, col: 2 },
            ]);
            expect(game.health_pickups()).to.deep.equal([]);
        });

        // Move onto 'H' at max HP spares the cell, fires playerNotHealed with reason
        it('should expect move onto H at max hp to spare the cell and fire playerNotHealed with reason', function () {
            let game = makeGame('{"grid":[["S","H","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.hp()).to.equal(3);
            expect(game.tick(1)).to.deep.equal([
                {
                    type: 'playerNotHealed',
                    row: 0,
                    col: 1,
                    reason: 'already_at_max_hp',
                    message: 'Already at maximum health',
                },
            ]);
            expect(game.health_pickups()).to.deep.equal([{ row: 0, col: 1, id: 0 }]);
        });

        // time_until_next_event_ms() — idle game has no upcoming events
        it('should expect time_until_next_event_ms() to be null on an idle game with no enemies or opening doors', function () {
            let game = makeGame('{"grid":[["S"," ","F"]]}');
            expect(game.time_until_next_event_ms()).to.equal(null);
        });

        // time_until_next_event_ms() — reports the soonest planned enemy commit
        it('should expect time_until_next_event_ms() to report the soonest planned enemy commit', function () {
            let game = makeGame('{"grid":[["S","E","F"]]}');
            // Default move_period_ms = 1500.
            expect(game.time_until_next_event_ms()).to.equal(1500);
        });

        // time_until_next_event_ms() — flushes to 0 once events are pending from a Move
        it('should expect time_until_next_event_ms() to be 0 when events are pending from a prior move', function () {
            let game = makeGame('{"grid":[["S","E","F"]]}');
            game.move_player(DirectionWasm.Right); // queues PlayerDamaged
            expect(game.time_until_next_event_ms()).to.equal(0);
        });

        // time_until_next_event_ms() — subtracts elapsed accum from the remaining period
        it('should expect time_until_next_event_ms() to subtract elapsed accum from the remaining period', function () {
            let game = makeGame('{"grid":[["S","E","F"]]}');
            game.tick(0);   // drain the move-time queued events first
            game.tick(400); // accum_ms = 400
            const remaining = game.time_until_next_event_ms();
            expect(remaining).to.be.closeTo(1100, 0.001);
        });
    });
}

function registerTests() {
    registerMazeTests();
    registerMazeSolutionTests();
    registerMazeGameTests();
}

async function run_tests() {
    await loadWasm();
    // Initialize tests
    const mocha = new Mocha();
    mocha.suite.emit('pre-require', global, 'solution', mocha);
    registerTests();
    // Run tests
    return new Promise((resolve) => {
        mocha.run(failures => {
            if (failures) {
                resolve(false);
            } else {
                resolve(true);
            }
        });
    });
}

export { run_tests };