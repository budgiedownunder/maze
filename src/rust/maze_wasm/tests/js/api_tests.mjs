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

function registerMazeTests() {
    describe('MazeWasm API', function () {
        // MazeWasm::new()
        it('should successfully create a new maze', function () {
            expect(() => new MazeWasm()).to.not.throw();
        });

        // MazeWasm::is_empty()
        it('should expect is_empty() to return true for a new maze', function () {
            expect(new MazeWasm().is_empty()).to.equal(true);
        });

        // MazeWasm::get_row_count()
        it('should expect get_row_count() to return zero for a new maze', function () {
            expect(new MazeWasm().get_row_count()).to.equal(0);
        });

        // MazeWasm::get_col_count()
        it('should expect get_col_count() to return zero for a new maze', function () {
            expect(new MazeWasm().get_col_count()).to.equal(0);
        });

        // MazeWasm::from_json()
        it('should expect from_json() to fail if provided with a numeric argument', function () {
            expect(() => new MazeWasm().from_json(1)).to.throw(invalidJSONStringArgumentError('number'));
        });

        it('should expect from_json() to fail if provided with a empty object argument', function () {
            expect(() => new MazeWasm().from_json({})).to.throw(invalidJSONStringArgumentError('object'));
        });

        it('should expect from_json() to fail if provided with a boolean argument', function () {
            expect(() => new MazeWasm().from_json(true)).to.throw(invalidJSONStringArgumentError('boolean'));
        });

        it('should expect from_json() to fail if provided with a null argument', function () {
            expect(() => new MazeWasm().from_json(null)).to.throw(invalidJSONStringArgumentError('unknown'));
        });

        it('should expect from_json() to fail if provided with an undefined argument', function () {
            expect(() => new MazeWasm().from_json(undefined)).to.throw(invalidJSONStringArgumentError('undefined'));
        });

        it('should expect from_json() to fail if provided with an empty string argument', function () {
            expect(() => new MazeWasm().from_json("")).to.throw(eofParsingValueError());
        });

        it('should expect from_json() to fail if provided with a string argument with a missing object close', function () {
            expect(() => new MazeWasm().from_json("{")).to.throw(eofParsingObjectError());
        });

        it('should expect from_json() to fail if provided with a string argument with a missing id field', function () {
            expect(() => new MazeWasm().from_json("{}")).to.throw(missingFieldError("id", 1, 2));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing name field', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id"}`)).to.throw(missingFieldError("name", 1, 16));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing name field value', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id","name":}`)).to.throw(expectedValueError(1, 24));
        });

        it('should expect from_json() to fail if provided with a string argument with a trailing comma', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id","name":"test",}`)).to.throw(trailingCommaError(1, 31));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing colon token for definition value', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id","name":"test", "definition"}`)).to.throw(expectedTokenError(":", 1, 44));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing definition field value', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id","name":"test", "definition":}`)).to.throw(expectedValueError(1, 45));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing grid field', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id","name":"test", "definition":{}}`)).to.throw(missingFieldError("grid", 1, 47));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing colon token for grid field value', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id","name":"test", "definition":{"grid"}}`)).to.throw(expectedTokenError(":", 1, 52));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing grid value', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id","name":"test", "definition":{"grid":}}`)).to.throw(expectedValueError(1, 53));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing grid value closing array bracket', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id","name":"test", "definition":{"grid":[}}`)).to.throw(expectedValueError(1, 54));
        });

        it('should expect from_json() to succeed if provided with a valid string argument with an empty array for the grid value', function () {
            expect(() => new MazeWasm().from_json(`{"id":"maze_id","name":"test", "definition":{"grid":[]}}`)).to.not.throw();
        });

        // MazeWasm::resize()
        it('should expect resize() to modify number of rows and columns in a maze', function () {
            let maze = new MazeWasm();
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
            let maze = new MazeWasm();
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
            expect(() => new MazeWasm().get_start_cell()).to.throw(noCellDefinedError("start"));
        });

        // MazeWasm::set_start_cell()
        runBadArgTests(function (argTest) {
            it(`should expect set_start_cell() to fail for a maze when passed an invalid 'start_row' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_start_cell(argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        it('should expect set_start_cell() to fail for a new maze when all arguments supplied', function () {
            expect(() => new MazeWasm().set_start_cell(1, 1)).to.throw(invalidPointError("start", 1, 1));
        });

        it('should expect set_start_cell() to succeed for a valid maze point and get_start_cell() should then return that cell', function () {
            let maze = new MazeWasm();
            maze.resize(10, 5);
            maze.set_start_cell(0, 1);
            expect(maze.get_start_cell()).to.deep.equal({ row: 0, col: 1 });
        });

        // MazeWasm::get_finish_cell()
        it('should expect get_finish_cell() to fail for a new maze', function () {
            expect(() => new MazeWasm().get_finish_cell()).to.throw(noCellDefinedError("finish"));
        });

        // MazeWasm::set_finish_cell()
        it('should expect set_finish_cell() to fail for a new maze', function () {
            expect(() => new MazeWasm().set_finish_cell(1, 1)).to.throw(invalidPointError("finish", 1, 1));
        });

        it('should expect set_finish_cell() to succeed for a valid maze point and get_finish_cell() should then return that cell', function () {
            let maze = new MazeWasm();
            maze.resize(10, 5);
            maze.set_finish_cell(9, 4);
            expect(maze.get_finish_cell()).to.deep.equal({ row: 9, col: 4 });
        });

        // MazeWasm::get_cell()
        runBadArgTests(function (argTest) {
            it(`should expect get_cell() to fail for a maze when passed an invalid 'row' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.get_cell(argTest.value)).to.throw(invalidArgumentError("row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect get_cell() to fail for a maze when passed an invalid 'col' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.get_cell(1, argTest.value)).to.throw(invalidArgumentError("col", "unsigned integer", argTest.desc));
            });
        });

        it('should expect get_cell() to succeed for a maze with no cells set when passed a valid location and for the cell type to be empty', function () {
            let maze = new MazeWasm();
            maze.resize(2, 2);
            let cellType = maze.get_cell(1, 1)
            expect(cellType).to.deep.equal({ cell_type: 0 });
        });

        // MazeWasm::set_wall_cells()
        runBadArgTests(function (argTest) {
            it(`should expect set_wall_cells() to fail for a maze when passed passed invalid 'start_row' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_wall_cells(argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect set_wall_cells() to fail for a maze when passed invalid 'start_col' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_wall_cells(0, argTest.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect set_wall_cells() to fail for a maze when passed invalid 'end_row' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_wall_cells(0, 0, argTest.value)).to.throw(invalidArgumentError("end_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect set_wall_cells() to fail for a maze when passed invalid 'end_col' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_wall_cells(0, 0, 0, argTest.value)).to.throw(invalidArgumentError("end_col", "unsigned integer", argTest.desc));
            });
        });

        it(`should expect set_wall_cells() to fail for a maze when passed out of bounds 'start_row' argument`, function () {
            let maze = new MazeWasm();
            maze.resize(2, 2);
            expect(() => maze.set_wall_cells(2, 0, 0, 0)).to.throw(invalidPointError("from", 2, 0));
        });

        it(`should expect set_wall_cells() to fail for a maze when passed out of bounds 'start_col' argument`, function () {
            let maze = new MazeWasm();
            maze.resize(2, 2);
            expect(() => maze.set_wall_cells(1, 2, 0, 0)).to.throw(invalidPointError("from", 1, 2));
        });

        it(`should expect set_wall_cells() to fail for a maze when passed out of bounds 'end_row' argument`, function () {
            let maze = new MazeWasm();
            maze.resize(2, 2);
            expect(() => maze.set_wall_cells(1, 1, 2, 0)).to.throw(invalidPointError("to", 2, 0));
        });

        it(`should expect set_wall_cells() to fail for a maze when passed out of bounds 'end_col' argument`, function () {
            let maze = new MazeWasm();
            maze.resize(2, 2);
            expect(() => maze.set_wall_cells(1, 1, 1, 2)).to.throw(invalidPointError("to", 1, 2));
        });

        it(`should expect set_wall_cells() to succeed for a maze when passed valid arguments and for get_cell() to return the correct cell_type before/after`, function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            let startRow = 1, startCol = 1, endRow = 2, endCol = 2;
            verifyCellType(maze, startRow, startCol, endRow, endCol, MazeCellTypeWasm.Empty);
            maze.set_wall_cells(startRow, startCol, endRow, endCol);
            verifyCellType(maze, startRow, startCol, endRow, endCol, MazeCellTypeWasm.Wall);
        });

        // MazeWasm::clear_cells()
        runBadArgTests(function (argTest) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'start_row' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.clear_cells(argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'start_col' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.clear_cells(0, argTest.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'end_row' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.clear_cells(0, 0, argTest.value)).to.throw(invalidArgumentError("end_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'end_col' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.clear_cells(0, 0, 0, argTest.value)).to.throw(invalidArgumentError("end_col", "unsigned integer", argTest.desc));
            });
        });

        it('should expect clear_cells() to succeed for a new maze', function () {
            let maze = new MazeWasm();
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
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.delete_rows(argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect delete_rows() to fail for a maze when passed passed invalid 'count' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.delete_rows(0, argTest.value)).to.throw(invalidArgumentError("count", "unsigned integer", argTest.desc));
            });
        });

        it(`should expect delete_rows() to fail if 'start_row' out of bounds`, function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            expect(() => maze.delete_rows(3, 4)).to.throw(indexOutOfBoundsError("start_row", 3));
        });

        it(`should expect delete_rows() to fail if too large 'count' is supplied`, function () {
            let maze = new MazeWasm();
            maze.resize(3, 2);
            expect(() => maze.delete_rows(1, 3)).to.throw(argmentTooLargeError("count", 3));
        });

        it('should expect delete_rows() to succeed for valid arguments and for get_row_count() to return the updated row count', function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            let oldCount = maze.get_row_count();
            maze.delete_rows(1, 2);
            let newCount = maze.get_row_count();
            expect((oldCount == 3) && (newCount == 1)).to.equal(true);
        });

        // MazeWasm::insert_rows()
        runBadArgTests(function (argTest) {
            it(`should expect insert_rows() to fail for a maze when passed passed invalid 'start_row' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.insert_rows(argTest.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect insert_rows() to fail for a maze when passed passed invalid 'count' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.insert_rows(0, argTest.value)).to.throw(invalidArgumentError("count", "unsigned integer", argTest.desc));
            });
        });

        it(`should expect insert_rows() to fail if 'start_row' out of bounds`, function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            expect(() => maze.insert_rows(4, 1)).to.throw(indexOutOfBoundsError("start_row", 4));
        });

        it('should expect insert_rows() to succeed when inserting between existing rows and for get_row_count() to return the updated row count', function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            let oldCount = maze.get_row_count();
            maze.insert_rows(1, 2);
            let newCount = maze.get_row_count();
            expect((oldCount == 3) && (newCount == 5)).to.equal(true);
        });

        it('should expect insert_rows() to allow insertion after last row and for get_row_count() to return the updated row count', function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            let oldCount = maze.get_row_count();
            maze.insert_rows(oldCount, 2);
            let newCount = maze.get_row_count();
            expect((oldCount == 3) && (newCount == 5)).to.equal(true);
        });

        // MazeWasm::delete_cols()
        runBadArgTests(function (argTest) {
            it(`should expect delete_cols() to fail for a maze when passed passed invalid 'start_col' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.delete_cols(argTest.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect delete_cols() to fail for a maze when passed passed invalid 'count' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.delete_cols(0, argTest.value)).to.throw(invalidArgumentError("count", "unsigned integer", argTest.desc));
            });
        });

        it(`should expect delete_cols() to fail if 'start_col' out of bounds`, function () {
            let maze = new MazeWasm();
            maze.resize(3, 2);
            expect(() => maze.delete_cols(3, 4)).to.throw(indexOutOfBoundsError("start_col", 3));
        });

        it(`should expect delete_cols() to fail if too large 'count' is supplied`, function () {
            let maze = new MazeWasm();
            maze.resize(3, 2);
            expect(() => maze.delete_cols(1, 3)).to.throw(argmentTooLargeError("count", 3));
        });

        it('should expect delete_cols() to succeed for valid arguments and for get_col_count() to return the updated column count', function () {
            let maze = new MazeWasm();
            maze.resize(3, 2);
            let oldCount = maze.get_col_count();
            maze.delete_cols(1, 1);
            let newCount = maze.get_col_count();
            expect((oldCount == 2) && (newCount == 1)).to.equal(true);
        });

        // MazeWasm::insert_cols()
        runBadArgTests(function (argTest) {
            it(`should expect insert_cols() to fail for a maze when passed passed invalid 'start_col' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.insert_cols(argTest.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", argTest.desc));
            });
        });

        runBadArgTests(function (argTest) {
            it(`should expect insert_cols() to fail for a maze when passed passed invalid 'count' argument (${argTest.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.insert_cols(0, argTest.value)).to.throw(invalidArgumentError("count", "unsigned integer", argTest.desc));
            });
        });

        it(`should expect insert_cols() to fail if 'start_col' out of bounds`, function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            expect(() => maze.insert_cols(4, 1)).to.throw(indexOutOfBoundsError("start_col", 4));
        });

        it('should expect insert_cols() to succeed when inserting between existing columns and for get_col_count() to return the updated column count', function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            let oldCount = maze.get_col_count();
            maze.insert_cols(1, 2);
            let newCount = maze.get_col_count();
            expect((oldCount == 3) && (newCount == 5)).to.equal(true);
        });

        it('should expect insert_cols() to allow insertion after last column and for get_col_count() to return the updated row count', function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            let oldCount = maze.get_col_count();
            maze.insert_cols(oldCount, 2);
            let newCount = maze.get_col_count();
            expect((oldCount == 3) && (newCount == 5)).to.equal(true);
        });

        // MazeWasm::solve()
        it('should expect solve() to fail for a new maze', function () {
            let maze = new MazeWasm();
            expect(() => maze.solve()).to.throw(noCellFoundError("start"));
        });

        it('should expect solve() to fail for a resized maze with no start cell set', function () {
            let maze = new MazeWasm();
            maze.resize(10, 5);
            expect(() => maze.solve()).to.throw(noCellFoundError("start"));
        });

        it('should expect solve() to fail for a resized maze with no finish cell set', function () {
            let maze = new MazeWasm();
            maze.resize(10, 5);
            maze.set_start_cell(0, 0);
            expect(() => maze.solve()).to.throw(noCellFoundError("finish"));
        });

        it('should expect solve() to succeed for a resized maze with start and finish cells set and for get_path_points() to return expected path', function () {
            let maze = new MazeWasm();
            maze.resize(10, 5);
            maze.set_start_cell(0, 0);
            maze.set_finish_cell(9, 4);
            let solution = maze.solve();
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
                expect(() => new MazeWasm().generate(argTest.value, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                    undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("row_count", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when row_count is less than 3', function () {
            expect(() => new MazeWasm().generate(2, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.throw(generateRowCountError());
        });

        runBadArgTests(function (argTest) {
            it(`should expect generate() to fail when passed an invalid 'col_count' argument (${argTest.desc})`, function () {
                expect(() => new MazeWasm().generate(7, argTest.value, GenerationAlgorithmWasm.RecursiveBacktracking,
                    undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("col_count", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when col_count is less than 3', function () {
            expect(() => new MazeWasm().generate(7, 2, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.throw(generateColCountError());
        });

        runBadOptArgTests(function (argTest) {
            it(`should expect generate() to fail when passed an invalid 'start_row' argument (${argTest.desc})`, function () {
                expect(() => new MazeWasm().generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                    argTest.value, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("start_row", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when start point is out of bounds', function () {
            expect(() => new MazeWasm().generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                10, 0, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.throw(generateStartOutOfBoundsError());
        });

        it('should expect generate() to succeed with a valid explicit start point', function () {
            let maze = new MazeWasm();
            expect(() => maze.generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                0, 0, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.not.throw();
            expect(maze.get_start_cell()).to.deep.equal({ row: 0, col: 0 });
        });

        runBadOptArgTests(function (argTest) {
            it(`should expect generate() to fail when passed an invalid 'finish_row' argument (${argTest.desc})`, function () {
                expect(() => new MazeWasm().generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                    undefined, undefined, argTest.value, undefined, undefined, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("finish_row", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when finish point is out of bounds', function () {
            expect(() => new MazeWasm().generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, 10, 0, undefined, undefined, undefined, undefined))
                .to.throw(generateFinishOutOfBoundsError());
        });

        it('should expect generate() to succeed with a valid explicit finish point', function () {
            let maze = new MazeWasm();
            expect(() => maze.generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, 6, 4, undefined, undefined, undefined, undefined))
                .to.not.throw();
            expect(maze.get_finish_cell()).to.deep.equal({ row: 6, col: 4 });
        });

        runBadOptArgTests(function (argTest) {
            it(`should expect generate() to fail when passed an invalid 'min_spine_length' argument (${argTest.desc})`, function () {
                expect(() => new MazeWasm().generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                    undefined, undefined, undefined, undefined, argTest.value, undefined, undefined, undefined))
                    .to.throw(invalidArgumentError("min_spine_length", "unsigned integer", argTest.desc));
            });
        });

        it('should expect generate() to fail when min_spine_length is impossible to satisfy', function () {
            expect(() => new MazeWasm().generate(3, 3, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, 1000, 1, undefined, undefined))
                .to.throw();
        });

        it('should expect generate() to succeed with a valid min_spine_length', function () {
            let maze = new MazeWasm();
            expect(() => maze.generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, 3, undefined, undefined, undefined))
                .to.not.throw();
            expect(maze.get_row_count()).to.equal(7);
            expect(maze.get_col_count()).to.equal(5);
        });

        it('should expect generate() to succeed with valid row_count and col_count and return a maze of the correct dimensions', function () {
            let maze = new MazeWasm();
            expect(() => maze.generate(7, 5, GenerationAlgorithmWasm.RecursiveBacktracking,
                undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined))
                .to.not.throw();
            expect(maze.get_row_count()).to.equal(7);
            expect(maze.get_col_count()).to.equal(5);
        });

    });
}

function registerMazeSolutionTests() {
    describe('MazeSolutionWasm API', function () {
        // MazeSolutionWasm::get_path_points()
        it('should expect get_path_points() to return expected path following a successful solve()', function () {
            let maze = new MazeWasm();
            maze.resize(10, 5);
            maze.set_start_cell(0, 0);
            maze.set_finish_cell(9, 4);
            let solution = maze.solve();
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
        // MazeGame::from_json()
        it('should expect from_json() to throw on invalid JSON', function () {
            expect(() => MazeGameWasm.from_json("")).to.throw();
        });

        it('should expect from_json() to throw on a maze with no start cell', function () {
            expect(() => MazeGameWasm.from_json('{"grid":[[" "," ","F"]]}')).to.throw(/no start cell/);
        });

        it('should expect from_json() to succeed with a valid maze JSON string', function () {
            expect(() => MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}')).to.not.throw();
        });

        // MazeGame::player_row()
        it('should expect player_row() to return 0 after from_json()', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            expect(game.player_row()).to.equal(0);
        });

        // MazeGame::player_col()
        it('should expect player_col() to return 0 after from_json()', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            expect(game.player_col()).to.equal(0);
        });

        // MazeGame::player_direction()
        it('should expect player_direction() to return DirectionWasm.None after from_json()', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            expect(game.player_direction()).to.equal(DirectionWasm.None);
        });

        // MazeGame::is_complete()
        it('should expect is_complete() to return false after from_json()', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            expect(game.is_complete()).to.equal(false);
        });

        // MazeGame::visited_cells()
        it('should expect visited_cells() to contain only the start cell after from_json()', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            expect(game.visited_cells()).to.deep.equal([{ row: 0, col: 0 }]);
        });

        // MazeGame::move_player() — move into empty cell
        it('should expect move_player(DirectionWasm.Right) to return MoveResultWasm.Moved when moving into an empty cell', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Moved);
        });

        // MazeGame::move_player() — move into wall
        it('should expect move_player(DirectionWasm.Right) to return MoveResultWasm.Blocked when moving into a wall', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S","W","F"]]}');
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Blocked);
        });

        // MazeGame::move_player() — out-of-bounds move
        it('should expect move_player(DirectionWasm.Up) to return MoveResultWasm.Blocked when moving out of bounds', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            expect(game.move_player(DirectionWasm.Up)).to.equal(MoveResultWasm.Blocked);
        });

        // MazeGame::move_player() — reach finish
        it('should expect move_player(DirectionWasm.Right) to return MoveResultWasm.Complete when moving into the finish cell', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S","F"]]}');
            expect(game.move_player(DirectionWasm.Right)).to.equal(MoveResultWasm.Complete);
        });

        // MazeGame::move_player() — DirectionWasm.None
        it('should expect move_player(DirectionWasm.None) to return MoveResultWasm.None', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            expect(game.move_player(DirectionWasm.None)).to.equal(MoveResultWasm.None);
        });

        // MazeGame::player_direction() — updates after move
        it('should expect player_direction() to return DirectionWasm.Right after moving right', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.player_direction()).to.equal(DirectionWasm.Right);
        });

        // MazeGame::player_direction() — updates even after blocked move
        it('should expect player_direction() to update even after a blocked move', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S","W","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.player_direction()).to.equal(DirectionWasm.Right);
        });

        // MazeGame::visited_cells() — grows after successful move
        it('should expect visited_cells() to grow after a successful move', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S"," ","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.visited_cells()).to.deep.equal([
                { row: 0, col: 0 },
                { row: 0, col: 1 }
            ]);
        });

        // MazeGame::visited_cells() — unchanged after blocked move
        it('should expect visited_cells() to not change after a blocked move', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S","W","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.visited_cells()).to.deep.equal([{ row: 0, col: 0 }]);
        });

        // MazeGame::visited_cells() — finish cell included on complete
        it('should expect visited_cells() to include the finish cell when the game is complete', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.visited_cells()).to.deep.equal([
                { row: 0, col: 0 },
                { row: 0, col: 1 }
            ]);
        });

        // MazeGame::is_complete() — true after reaching finish
        it('should expect is_complete() to return true after reaching the finish cell', function () {
            let game = MazeGameWasm.from_json('{"grid":[["S","F"]]}');
            game.move_player(DirectionWasm.Right);
            expect(game.is_complete()).to.equal(true);
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