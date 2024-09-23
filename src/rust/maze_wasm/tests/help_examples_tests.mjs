import { readFile } from 'fs/promises';
import init, { MazeWasm, MazeCellTypeWasm } from '../pkg/maze_wasm.js'; // Default import of the WebAssembly module

// Custom function to handle loading WASM in Node.js
async function loadWasm() {
    // Read the .wasm file manually
    const wasmBuffer = await readFile('../pkg/maze_wasm_bg.wasm');
    // Initialize the WASM module using the buffer
    await init({ module_or_path: wasmBuffer });
}

async function run_tests() {
    try {
        await loadWasm();
        console.log('WASM module initialized successfully!');

        const maze = new MazeWasm();
        maze.resize(10, 4);
        console.log("Number rows = ", maze.get_row_count());
        console.log("Number cols = ", maze.get_col_count());

        return true;
    } catch (error) {
        console.error('Error initializing the WASM module:', error);
        return false;
    }
}

export { run_tests };