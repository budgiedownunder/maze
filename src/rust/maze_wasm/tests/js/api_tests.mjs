// This file exports a single function run_tests() which runs the tests for
// the maze_wasm JavaScript API
import { readFile } from 'fs/promises';
import init, { MazeWasm, MazeCellTypeWasm } from '../../pkg/maze_wasm.js';
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

function run_bad_arg_tests(callback) {
    let arg_tests = [
        { value: undefined, desc: "undefined" },
        { value: null, desc: "unknown" },
        { value: -1, desc: "negative number" },
        { value: "some_text", desc: "string" },
        { value: true, desc: "boolean" },
        { value: {}, desc: "object" }
    ];

    for (let i = 0; i < arg_tests.length; i++) {
        callback(arg_tests[i]);
    }
}

function verifyCellType(maze, start_row, start_col, end_row, end_col, cellType) {
    for (let row = start_row; row <= end_row; row++) {
        for (let col = start_col; col <= end_col; col++) {
            let cell_info = maze.get_cell(row, col);
            expect(cell_info.cell_type).to.equal(cellType);
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

        it('should expect from_json() to fail if provided with a string argument with a missing name field', function () {
            expect(() => new MazeWasm().from_json("{}")).to.throw(missingFieldError("name", 1, 2));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing name field value', function () {
            expect(() => new MazeWasm().from_json(`{"name":}`)).to.throw(expectedValueError(1, 9));
        });

        it('should expect from_json() to fail if provided with a string argument with a trailing comma', function () {
            expect(() => new MazeWasm().from_json(`{"name":"test",}`)).to.throw(trailingCommaError(1, 16));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing colon token for definition value', function () {
            expect(() => new MazeWasm().from_json(`{"name":"test", "definition"}`)).to.throw(expectedTokenError(":", 1, 29));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing definition field value', function () {
            expect(() => new MazeWasm().from_json(`{"name":"test", "definition":}`)).to.throw(expectedValueError(1, 30));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing grid field', function () {
            expect(() => new MazeWasm().from_json(`{"name":"test", "definition":{}}`)).to.throw(missingFieldError("grid", 1, 32));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing colon token for grid field value', function () {
            expect(() => new MazeWasm().from_json(`{"name":"test", "definition":{"grid"}}`)).to.throw(expectedTokenError(":", 1, 37));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing grid value', function () {
            expect(() => new MazeWasm().from_json(`{"name":"test", "definition":{"grid":}}`)).to.throw(expectedValueError(1, 38));
        });

        it('should expect from_json() to fail if provided with a string argument with a missing grid value closing array bracket', function () {
            expect(() => new MazeWasm().from_json(`{"name":"test", "definition":{"grid":[}}`)).to.throw(expectedValueError(1, 39));
        });

        it('should expect from_json() to succeed if provided with a valid string argument with an empty array for the grid value', function () {
            expect(() => new MazeWasm().from_json(`{"name":"test", "definition":{"grid":[]}}`)).to.not.throw();
        });

        // MazeWasm::resize()
        it('should expect resize() to modify number of rows and columns in a maze', function () {
            let maze = new MazeWasm();
            let old_is_empty = maze.is_empty();
            let old_row_count = maze.get_row_count();
            let old_col_count = maze.get_col_count();
            maze.resize(10, 5);
            let new_is_empty = maze.is_empty();
            let new_row_count = maze.get_row_count();
            let new_col_count = maze.get_col_count();

            expect((old_is_empty == true) && (old_row_count == 0) && (old_col_count == 0) && (new_is_empty == false) &&
                (new_row_count == 10) && (new_col_count == 5)).to.equal(true);
        });

        // MazeWasm::reset()
        it('should expect reset() to clear all rows and columns in a maze', function () {
            let maze = new MazeWasm();
            maze.resize(10, 5);
            let old_is_empty = maze.is_empty();
            let old_row_count = maze.get_row_count();
            let old_col_count = maze.get_col_count();
            maze.reset();
            let new_is_empty = maze.is_empty();
            let new_row_count = maze.get_row_count();
            let new_col_count = maze.get_col_count();

            expect((old_is_empty == false) && (old_row_count == 10) && (old_col_count == 5) && (new_is_empty == true) &&
                (new_row_count == 0) && (new_col_count == 0)).to.equal(true);
        });

        // MazeWasm::get_start_cell()
        it('should expect get_start_cell() to fail for a new maze', function () {
            expect(() => new MazeWasm().get_start_cell()).to.throw(noCellDefinedError("start"));
        });

        // MazeWasm::set_start_cell()
        run_bad_arg_tests(function (arg_test) {
            it(`should expect set_start_cell() to fail for a maze when passed an invalid 'start_row' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_start_cell(arg_test.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", arg_test.desc));
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
        run_bad_arg_tests(function (arg_test) {
            it(`should expect get_cell() to fail for a maze when passed an invalid 'row' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.get_cell(arg_test.value)).to.throw(invalidArgumentError("row", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect get_cell() to fail for a maze when passed an invalid 'col' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.get_cell(1, arg_test.value)).to.throw(invalidArgumentError("col", "unsigned integer", arg_test.desc));
            });
        });

        it('should expect get_cell() to succeed for a maze with no cells set when passed a valid location and for the cell type to be empty', function () {
            let maze = new MazeWasm();
            maze.resize(2, 2);
            let cellType = maze.get_cell(1, 1)
            expect(cellType).to.deep.equal({ cell_type: 0 });
        });

        // MazeWasm::set_wall_cells()
        run_bad_arg_tests(function (arg_test) {
            it(`should expect set_wall_cells() to fail for a maze when passed passed invalid 'start_row' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_wall_cells(arg_test.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect set_wall_cells() to fail for a maze when passed invalid 'start_col' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_wall_cells(0, arg_test.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect set_wall_cells() to fail for a maze when passed invalid 'end_row' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_wall_cells(0, 0, arg_test.value)).to.throw(invalidArgumentError("end_row", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect set_wall_cells() to fail for a maze when passed invalid 'end_col' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.set_wall_cells(0, 0, 0, arg_test.value)).to.throw(invalidArgumentError("end_col", "unsigned integer", arg_test.desc));
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
            let start_row = 1, start_col = 1, end_row = 2, end_col = 2;
            verifyCellType(maze, start_row, start_col, end_row, end_col, MazeCellTypeWasm.Empty);
            maze.set_wall_cells(start_row, start_col, end_row, end_col);
            verifyCellType(maze, start_row, start_col, end_row, end_col, MazeCellTypeWasm.Wall);
        });

        // MazeWasm::clear_cells()
        run_bad_arg_tests(function (arg_test) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'start_row' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.clear_cells(arg_test.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'start_col' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.clear_cells(0, arg_test.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'end_row' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.clear_cells(0, 0, arg_test.value)).to.throw(invalidArgumentError("end_row", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect clear_cells() to fail for a maze when passed passed invalid 'end_col' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.clear_cells(0, 0, 0, arg_test.value)).to.throw(invalidArgumentError("end_col", "unsigned integer", arg_test.desc));
            });
        });

        it('should expect clear_cells() to succeed for a new maze', function () {
            let maze = new MazeWasm();
            maze.resize(2, 2);
            let start_row = 0, start_col = 0, end_row = 1, end_col = 1;
            verifyCellType(maze, start_row, start_col, end_row, end_col, MazeCellTypeWasm.Empty);
            maze.set_wall_cells(start_row, start_col, end_row, end_col);
            verifyCellType(maze, start_row, start_col, end_row, end_col, MazeCellTypeWasm.Wall);
            maze.clear_cells(start_row, start_col, end_row, end_col);
            verifyCellType(maze, start_row, start_col, end_row, end_col, MazeCellTypeWasm.Empty);
        });

        // MazeWasm::delete_rows()
        run_bad_arg_tests(function (arg_test) {
            it(`should expect delete_rows() to fail for a maze when passed passed invalid 'start_row' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.delete_rows(arg_test.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect delete_rows() to fail for a maze when passed passed invalid 'count' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.delete_rows(0, arg_test.value)).to.throw(invalidArgumentError("count", "unsigned integer", arg_test.desc));
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
            let old_count = maze.get_row_count();
            maze.delete_rows(1, 2);
            let new_count = maze.get_row_count();
            expect((old_count == 3) && (new_count == 1)).to.equal(true);
        });

        // MazeWasm::insert_rows()
        run_bad_arg_tests(function (arg_test) {
            it(`should expect insert_rows() to fail for a maze when passed passed invalid 'start_row' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.insert_rows(arg_test.value)).to.throw(invalidArgumentError("start_row", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect insert_rows() to fail for a maze when passed passed invalid 'count' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.insert_rows(0, arg_test.value)).to.throw(invalidArgumentError("count", "unsigned integer", arg_test.desc));
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
            let old_count = maze.get_row_count();
            maze.insert_rows(1, 2);
            let new_count = maze.get_row_count();
            expect((old_count == 3) && (new_count == 5)).to.equal(true);
        });

        it('should expect insert_rows() to allow insertion after last row and for get_row_count() to return the updated row count', function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            let old_count = maze.get_row_count();
            maze.insert_rows(old_count, 2);
            let new_count = maze.get_row_count();
            expect((old_count == 3) && (new_count == 5)).to.equal(true);
        });

        // MazeWasm::delete_cols()
        run_bad_arg_tests(function (arg_test) {
            it(`should expect delete_cols() to fail for a maze when passed passed invalid 'start_col' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.delete_cols(arg_test.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect delete_cols() to fail for a maze when passed passed invalid 'count' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.delete_cols(0, arg_test.value)).to.throw(invalidArgumentError("count", "unsigned integer", arg_test.desc));
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
            let old_count = maze.get_col_count();
            maze.delete_cols(1, 1);
            let new_count = maze.get_col_count();
            expect((old_count == 2) && (new_count == 1)).to.equal(true);
        });

        // MazeWasm::insert_cols()
        run_bad_arg_tests(function (arg_test) {
            it(`should expect insert_cols() to fail for a maze when passed passed invalid 'start_col' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.insert_cols(arg_test.value)).to.throw(invalidArgumentError("start_col", "unsigned integer", arg_test.desc));
            });
        });

        run_bad_arg_tests(function (arg_test) {
            it(`should expect insert_cols() to fail for a maze when passed passed invalid 'count' argument (${arg_test.desc})`, function () {
                let maze = new MazeWasm();
                maze.resize(2, 2);
                expect(() => maze.insert_cols(0, arg_test.value)).to.throw(invalidArgumentError("count", "unsigned integer", arg_test.desc));
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
            let old_count = maze.get_col_count();
            maze.insert_cols(1, 2);
            let new_count = maze.get_col_count();
            expect((old_count == 3) && (new_count == 5)).to.equal(true);
        });

        it('should expect insert_cols() to allow insertion after last column and for get_col_count() to return the updated row count', function () {
            let maze = new MazeWasm();
            maze.resize(3, 3);
            let old_count = maze.get_col_count();
            maze.insert_cols(old_count, 2);
            let new_count = maze.get_col_count();
            expect((old_count == 3) && (new_count == 5)).to.equal(true);
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

function registerTests() {
    registerMazeTests();
    registerMazeSolutionTests();
}

async function run_tests() {
    await loadWasm();
    // Initialize tests
    const mocha = new Mocha();
    mocha.suite.emit('pre-require', global, 'solution', mocha);
    registerTests(mocha);
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