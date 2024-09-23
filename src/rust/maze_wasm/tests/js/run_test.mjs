// JavaScript test runner - executes an async function run_tests(hide_results) defined in the target JavaScript test file
// 
// This is required to be run from node as follows:

// node <path_to_this_file> <path_to_target_file_containing_run_tests>

const testFileName = process.argv[2];

if (!testFileName) {
    console.error('Please provide the name of the JavaScript file to test.');
    process.exit(1);
}

async function run() {
    try {
        const module = await import(testFileName);
        const success = await module.run_tests(true);
        if (success) {
            console.log("All JavaScript tests ran successfully");
        } else {
            console.error("One or more of the JavaScript tests failed - check output logs for details");
        }
        return success;
    } catch (error) {
        console.error('Error running the JavaScript tests:', error);
        return false;
    }
}

let success = await run();
process.exit(success ? 0 : 1);
