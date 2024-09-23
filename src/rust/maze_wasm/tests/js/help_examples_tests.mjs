import { readFile } from 'fs/promises';
import init, { MazeWasm, MazeCellTypeWasm } from '../../pkg/maze_wasm.js';

// Custom function to handle loading WASM in Node.js
async function loadWasm() {
    const wasmBuffer = await readFile('../../pkg/maze_wasm_bg.wasm');
    await init({ module_or_path: wasmBuffer });
}

async function run_tests() {
    try {
        await loadWasm();
        console.log('WASM module initialized successfully!');

        const maze = new MazeWasm();
        maze.resize(10, 3);
        console.log("Number rows = ", maze.get_row_count());
        console.log("Number cols = ", maze.get_col_count());

        return true;
    } catch (error) {
        console.error('Error initializing the WASM module:', error);
        return false;
    }
}

export { run_tests };