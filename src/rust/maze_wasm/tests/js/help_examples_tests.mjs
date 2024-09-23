import { readFile } from 'fs/promises';
import init, { MazeWasm, MazeCellTypeWasm } from '../../pkg/maze_wasm.js';

// Custom function to handle loading WASM in Node.js
async function loadWasm() {
    const wasmBuffer = await readFile('../../pkg/maze_wasm_bg.wasm');
    await init({ module_or_path: wasmBuffer });
}

function testSolutionGetPathPoints() {
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
    { name: "solution:get_path_points() example", testFunction: testSolutionGetPathPoints },
];

// Function to run all tests
function runTests() {
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
        disableConsoleLog();
        const result = test.testFunction();
        if (!result) {
            console.log(`Test "${test.name}" failed.`);
            allSucceeded = false;
        }
    }
    enableConsoleLog();
    return allSucceeded;
}

async function run_tests() {
    await loadWasm();
    return runTests();
}

export { run_tests };