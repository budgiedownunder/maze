mod mock_app;
use crate::mock_app::MockApp;
use maze::Definition;
use maze::Maze;
use maze_cli::app::App;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

lazy_static::lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

fn to_vec_strings(vec_of_str: Vec<&str>) -> Vec<String> {
    vec_of_str.iter().map(|&s| s.to_string()).collect()
}

fn vec_append_string_copies(v: &mut Vec<String>, s: &str, n: usize) {
    v.resize(v.len() + n, s.to_string());
}

fn delete_file(file: &str) {
    let _ = fs::remove_file(file);
    let mut count = 0;
    loop {
        // Secondary check, in case there is lag in the operating system
        if !Path::new(file).exists() {
            break;
        }
        count += 1;
        if count == 10 {
            break;
        }
        sleep(Duration::from_millis(10));
    }
}

fn delete_files_with_ext(dir: &str, extension: &str) -> std::io::Result<()> {
    let files = fs::read_dir(dir)?;
    for file in files {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    if let Some(file_name) = path.file_name() {
                        let file_name_str = file_name.to_string_lossy();
                        delete_file(&file_name_str);
                    }
                }
            }
        }
    }
    Ok(())
}

fn do_quit_and_verify(
    app: &mut MockApp,
    expected_output: &mut Vec<String>,
    reset_input: bool,
) -> Result<(), Box<dyn Error>> {
    app.add_input_key('Q', reset_input);
    expected_output.push("Exiting...".to_string());
    app.run()?;
    delete_files_with_ext(".", "json")?;
    app.verify_output(&expected_output)?;
    Ok(())
}

fn do_quit_run_and_verify(
    app: &mut MockApp,
    expected_output: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    do_quit_and_verify(app, expected_output, false)?;
    Ok(())
}

fn add_press_any_key_steps(app: &mut MockApp, expected_output: &mut Vec<String>) {
    expected_output.push(MockApp::get_press_any_key_text().to_string());
    app.add_input_key(' ', false);
}

fn do_press_any_key_quit_run_and_verify(
    app: &mut MockApp,
    expected_output: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    add_press_any_key_steps(app, expected_output);
    do_quit_run_and_verify(app, expected_output)?;
    Ok(())
}

fn add_enter_number_steps(
    app: &mut MockApp,
    expected_output: &mut Vec<String>,
    prompt: &str,
    has_range: bool,
    lower: &str,
    upper: &str,
    bad_values: &[&str],
    good_value: &str,
) {
    for bad_value in bad_values.iter() {
        expected_output.push(prompt.to_string());
        app.add_input_line(bad_value, false);
        if has_range {
            expected_output.push(format!("Invalid value '{}' (out of bounds), please enter an integer value between {} and {} (inclusive)", bad_value, lower, upper));
        } else {
            expected_output.push(format!("Invalid value '{}' (out of bounds), please enter an integer value greater or equal to {}", bad_value, lower));
        }
    }
    expected_output.push(prompt.to_string());
    app.add_input_line(good_value, false);
}

fn add_save_maze_as_steps(
    app: &mut MockApp,
    expected_output: &mut Vec<String>,
    operation_key: char,
    reset_output: bool,
    current_name: &str,
    new_name: &str,
    expect_prompt_overwrite: bool,
    abandon_overwrite: bool,
) {
    app.add_input_key(operation_key, reset_output);
    expected_output.push(format!("Current name is '{}'", current_name));
    expected_output.push("Enter name of maze to save as: ".to_string());
    app.add_input_line(new_name, false);
    if expect_prompt_overwrite {
        expected_output.push(format!(
            "A maze with the name '{}' already exists. Overwrite it? (Y/N)",
            new_name
        ));
        if abandon_overwrite {
            app.add_input_key('N', false);
            expected_output.push("Maze not saved".to_string());
            return;
        }
        app.add_input_key('Y', false);
    }
    expected_output.push(format!("Saved '{}' to '{}.json'", new_name, new_name));
}

fn add_delete_maze_steps(
    app: &mut MockApp,
    expected_output: &mut Vec<String>,
    reset_output: bool,
    name: &str,
    names_expected: Vec<&str>,
    expect_exists: bool,
) {
    app.add_input_key('X', reset_output);
    expected_output.push(format!("Available mazes = {}\n", names_expected.len()));
    for (i, name) in names_expected.iter().enumerate() {
        expected_output.push(format!("{} - {}", i + 1, name));
    }
    if names_expected.len() > 0 {
        expected_output.push("Enter name of maze to delete: ".to_string());
        app.add_input_line(name, false);
        expected_output.push(format!(
            "Are you sure you want to delete the maze '{}'? (Y/N)",
            name
        ));
        app.add_input_key('Y', false);
        if expect_exists {
            expected_output.push("Maze deleted".to_string());
        } else {
            expected_output.push("Failed: File or directory not found".to_string());
        }
    } else {
        expected_output.push("There are no mazes available to delete".to_string());
    }
}

#[test]
fn should_be_able_to_quit_on_start() -> Result<(), Box<dyn Error>> {
    let mut expected_output = vec![];
    do_quit_and_verify(&mut MockApp::new(), &mut expected_output, true)?;
    Ok(())
}

#[test]
fn should_be_able_to_insert_rows_into_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('I', true);
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app, &mut expected_output,
        "Number rows to insert: ",
        false, "0", "", &[],"5",
    );
    expected_output.push("Success - new dimensions: 5 row(s), 0 column(s)".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_prevent_insert_invalid_rows_into_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('I', true);
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app, &mut expected_output,
        "Number rows to insert: ",
        false, "0", "", &["B", "-2"],"5",
    );
    expected_output.push("Success - new dimensions: 5 row(s), 0 column(s)".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_prevent_insert_invalid_rows_into_non_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('I', true);
    expected_output.push("Current dimensions: 10 row(s), 5 column(s)".to_string());
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Insert at row: ",
        true, "1", "11", &["A", "-1", "12"], "1"
    );
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Number rows to insert: ",
        false, "0", "", &["B", "-2"],"5"
    );
    expected_output.push("Success - new dimensions: 15 row(s), 5 column(s)".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_delete_rows_from_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('D', true);
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    expected_output.push("Definition is empty - no rows to delete".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_delete_invalid_rows_from_non_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('D', true);
    expected_output.push("Current dimensions: 10 row(s), 5 column(s)".to_string());
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app, &mut expected_output,
        "Delete rows from: ",
        true, "1", "10", &["A", "-1", "11"], "3"
    );
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Number rows to delete: ",
        true, "1", "8", &["A", "-1", "9"], "4"
    );
    expected_output.push("Success - new dimensions: 6 row(s), 5 column(s)".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_insert_cols_into_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('N', true);
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    expected_output
        .push("Definition is empty - insert some rows before adding columns".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_prevent_insert_invalid_cols_into_non_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('N', true);
    expected_output.push("Current dimensions: 10 row(s), 5 column(s)".to_string());
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Insert at column: ",
        true, "1", "6", &["B", "-1", "12"], "5"
    );
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Number columns to insert: ",
        false, "0", "", &["B", "-2"], "7"
    );
    expected_output.push("Success - new dimensions: 10 row(s), 12 column(s)".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_delete_cols_from_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('L', true);
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    expected_output.push("Definition has no columns to delete".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_delete_invalid_cols_from_non_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('L', true);
    expected_output.push("Current dimensions: 10 row(s), 5 column(s)".to_string());
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Delete columns from: ",
        true, "1", "5", &["A", "-1", "6"], "4"
    );
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Number columns to delete: ",
        true, "1", "2", &["A", "-1", "4"], "2"
    );
    expected_output.push("Success - new dimensions: 10 row(s), 3 column(s)".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

fn run_set_endpoint_test_in_empty_maze(
    operation_key: char,
    name: &str,
) -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let operation_message = format!("Set {}", name);
    let expected_error_message = format!(
        "Maze has no cells - add some rows and columns first before setting the {} cell",
        name
    );
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key(operation_key, true);
    expected_output.push(operation_message);
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    expected_output.push(expected_error_message);
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_set_start_in_empty_maze() -> Result<(), Box<dyn Error>> {
    run_set_endpoint_test_in_empty_maze('A', "start")?;
    Ok(())
}

#[test]
fn should_not_be_able_to_set_finish_in_empty_maze() -> Result<(), Box<dyn Error>> {
    run_set_endpoint_test_in_empty_maze('F', "finish")?;
    Ok(())
}

fn run_modify_endpoint_test(
    operation_key: char,
    operation: &str,
    endpoint_char: char,
) -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(3, 5));
    let modified_row = format!("░░{}░░", endpoint_char);
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key(operation_key, true);
    expected_output.push(operation.to_string());
    expected_output.push("Current dimensions: 3 row(s), 5 column(s)".to_string());
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Row:",
        true, "1", "3", &["A", "-1", "11"], "2"
    );
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Column:",
        true, "1", "5", &["B", "-1", "6"], "3"
    );
    add_press_any_key_steps(&mut mock_app, &mut expected_output);
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    mock_app.add_input_key('P', false);
    expected_output.push("Current dimensions: 3 row(s), 5 column(s)".to_string());
    expected_output.push("\nDefinition:\n".to_string());
    expected_output.push("░░░░░".to_string());
    expected_output.push(modified_row);
    expected_output.push("░░░░░".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_set_start_in_non_empty_maze() -> Result<(), Box<dyn Error>> {
    run_modify_endpoint_test('A', "Set start", 'S')?;
    Ok(())
}

#[test]
fn should_not_be_able_to_set_walls_in_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('W', true);
    expected_output.push("Set walls".to_string());
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    expected_output.push(
        "Maze has no cells - add some rows and columns first before modifying walls".to_string(),
    );
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

fn run_modify_walls_test(
    operation_key: char,
    operation: &str,
    change_char: char,
) -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let modified_row = format!("░{}{}{}░", change_char, change_char, change_char);
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key(operation_key, true);
    expected_output.push(operation.to_string());
    expected_output.push("Current dimensions: 10 row(s), 5 column(s)".to_string());
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app,&mut expected_output,
        "Start row:",
        true, "1", "10", &["A", "-1", "11"], "3"
    );
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app, &mut expected_output,
        "Start column:",
        true, "1", "5", &["B", "-1", "6"], "2"
    );
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app, &mut expected_output,
        "End row:",
        true, "1", "10", &["C", "-1", "11"], "5"
    );
    #[rustfmt::skip]
    add_enter_number_steps(
        &mut mock_app, &mut expected_output,
        "End column:",
        true, "1", "5", &["D", "-1", "6"], "4"
    );
    add_press_any_key_steps(&mut mock_app, &mut expected_output);
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    mock_app.add_input_key('P', false);
    expected_output.push("Current dimensions: 10 row(s), 5 column(s)".to_string());
    expected_output.push("\nDefinition:\n".to_string());
    let blank_row = "░░░░░".to_string();
    vec_append_string_copies(&mut expected_output, &blank_row, 2);
    vec_append_string_copies(&mut expected_output, &modified_row, 3);
    vec_append_string_copies(&mut expected_output, &blank_row, 5);
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_set_walls_in_non_empty_maze() -> Result<(), Box<dyn Error>> {
    run_modify_walls_test('W', "Set walls", '█')?;
    Ok(())
}

#[test]
fn should_not_be_able_to_clear_walls_in_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('C', true);
    expected_output.push("Clear walls".to_string());
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    expected_output.push(
        "Maze has no cells - add some rows and columns first before modifying walls".to_string(),
    );
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_clear_walls_in_non_empty_maze() -> Result<(), Box<dyn Error>> {
    run_modify_walls_test('C', "Clear walls", '░')?;
    Ok(())
}

#[test]
fn should_be_able_to_resize_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('R', true);
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    expected_output.push("Enter new row count: ".to_string());
    mock_app.add_input_line("5", false);
    expected_output.push("Enter new column count: ".to_string());
    mock_app.add_input_line("10", false);
    expected_output.push("Success - new dimensions: 5 row(s), 10 column(s)".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('E', true);
    expected_output
        .push("Set maze to empty? [current dimensions: 10 row(s), 5 column(s)] (Y/N)".to_string());
    mock_app.add_input_key('Y', false);
    expected_output.push("Maze set to empty".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_solve_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('S', true);
    expected_output.push("Failed to solve maze: no start cell found within maze".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_solve_maze_with_no_start_cell() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    #[rustfmt::skip]
    let grid: Vec<Vec<char>> = vec![
        vec![' ', 'F'],
    ];
    mock_app.current_maze = Maze::from_vec(grid);
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('S', true);
    expected_output.push("Failed to solve maze: no start cell found within maze".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_solve_maze_with_no_finish_cell() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    #[rustfmt::skip]
    let grid: Vec<Vec<char>> = vec![
        vec!['S', ' '],
     ];
    mock_app.current_maze = Maze::from_vec(grid);
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('S', true);
    expected_output.push("Failed to solve maze: no finish cell found within maze".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_solve_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    #[rustfmt::skip]
    let grid: Vec<Vec<char>> = vec![
        vec!['S', 'W', ' ', 'F', 'W'],
        vec![' ', 'W', ' ', 'W', ' '],
        vec![' ', ' ', ' ', 'W', ' '],
     ];
    mock_app.current_maze = Maze::from_vec(grid);
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('S', true);
    expected_output.push("\nSuccessfully solved maze:\n".to_string());
    expected_output.push("S█→F█".to_string());
    expected_output.push("↓█↑█░".to_string());
    expected_output.push("→→↑█░".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_solve_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    #[rustfmt::skip]
    let grid: Vec<Vec<char>> = vec![
        vec!['S', 'W', 'W', 'F', 'W'],
        vec![' ', 'W', ' ', 'W', ' '],
        vec![' ', ' ', ' ', 'W', ' '],
     ];
    mock_app.current_maze = Maze::from_vec(grid);
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('S', true);
    expected_output.push("Failed to solve maze: no solution found".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_print_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('P', true);
    expected_output.push("Current dimensions: 0 row(s), 0 column(s)".to_string());
    expected_output.push("\nDefinition:\n".to_string());
    expected_output.push("Maze is empty".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_print_maze_with_content() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(2, 3));
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('P', true);
    expected_output.push("Current dimensions: 2 row(s), 3 column(s)".to_string());
    expected_output.push("\nDefinition:\n".to_string());
    vec_append_string_copies(&mut expected_output, "░░░", 2);
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_open_non_existant_maze() -> Result<(), Box<dyn Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_file("does_not_exist.json");
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('O', true);
    expected_output.push("Enter name of maze to open: ".to_string());
    mock_app.add_input_line("does_not_exist", false);
    expected_output.push("Failed: File or directory not found".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_no_mazes_listed() -> Result<(), Box<dyn Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_files_with_ext(".", "json")?;
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    mock_app.add_input_key('U', true);
    expected_output.push("Available mazes = 0\n".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_mazes_listed_after_save() -> Result<(), Box<dyn Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_files_with_ext(".", "json")?;
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    #[rustfmt::skip]
    add_save_maze_as_steps(&mut mock_app, &mut expected_output, 'V', true, "", "saved_maze", false, false);
    add_press_any_key_steps(&mut mock_app, &mut expected_output);
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    mock_app.add_input_key('U', false);
    expected_output.push("Available mazes = 1\n".to_string());
    expected_output.push("1 - saved_maze".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_open_a_saved_maze() -> Result<(), Box<dyn Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_files_with_ext(".", "json")?;
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    #[rustfmt::skip]
    add_save_maze_as_steps(&mut mock_app, &mut expected_output, 'V', true, "", "saved_maze", false, false);
    add_press_any_key_steps(&mut mock_app, &mut expected_output);
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    mock_app.add_input_key('O', false);
    expected_output.push("Enter name of maze to open: ".to_string());
    mock_app.add_input_line("saved_maze", false);
    expected_output
        .push("Maze 'saved_maze' successfully loaded from 'saved_maze.json'".to_string());
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_save_maze() -> Result<(), Box<dyn Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_files_with_ext(".", "json")?;
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    #[rustfmt::skip]
    add_save_maze_as_steps(&mut mock_app, &mut expected_output, 'V', true, "", "saved_maze", false, false);
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_save_new_maze_as_and_overwrite() -> Result<(), Box<dyn Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_files_with_ext(".", "json")?;
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    #[rustfmt::skip]
    add_save_maze_as_steps(&mut mock_app, &mut expected_output, 'Z', true, "", "first_name", false, false);
    add_press_any_key_steps(&mut mock_app, &mut expected_output);
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    #[rustfmt::skip]
    add_save_maze_as_steps(&mut mock_app, &mut expected_output, 'Z', false, "first_name", "second_name", false, false);
    add_press_any_key_steps(&mut mock_app, &mut expected_output);
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    #[rustfmt::skip]
    add_save_maze_as_steps(&mut mock_app, &mut expected_output, 'Z', false, "second_name", "first_name", true, false);
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_save_new_maze_as_and_abandon_overwrite() -> Result<(), Box<dyn Error>> {
    // /*
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_files_with_ext(".", "json")?;
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    #[rustfmt::skip]
    add_save_maze_as_steps(&mut mock_app, &mut expected_output, 'Z', true, "", "first_name", false, false);
    add_press_any_key_steps(&mut mock_app, &mut expected_output);
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    add_save_maze_as_steps(
        &mut mock_app,
        &mut expected_output,
        'Z',
        false,
        "first_name",
        "first_name",
        true,
        true,
    );
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_unable_delete_when_no_mazes() -> Result<(), Box<dyn Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_files_with_ext(".", "json")?;
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    #[rustfmt::skip]
    add_delete_maze_steps(&mut mock_app, &mut expected_output, true, "", [].to_vec(), false);
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_be_only_able_to_delete_maze_after_save() -> Result<(), Box<dyn Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_files_with_ext(".", "json")?;
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    add_save_maze_as_steps(
        &mut mock_app,
        &mut expected_output,
        'V',
        true,
        "",
        "saved_maze",
        false,
        false,
    );
    add_press_any_key_steps(&mut mock_app, &mut expected_output);
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    #[rustfmt::skip]
    add_delete_maze_steps(&mut mock_app, &mut expected_output,
        false, "saved_maze", ["saved_maze"].to_vec(), true);
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_delete_invalid_maze_after_save() -> Result<(), Box<dyn Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    delete_files_with_ext(".", "json")?;
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output: Vec<String> = vec![];
    #[rustfmt::skip]
    add_save_maze_as_steps(&mut mock_app, &mut expected_output, 'V', true, "", "saved_maze", false, false);
    add_press_any_key_steps(&mut mock_app, &mut expected_output);
    expected_output.extend(to_vec_strings(MockApp::get_menu_lines()));
    #[rustfmt::skip]
    add_delete_maze_steps(&mut mock_app, &mut expected_output,
        false, "does_not_exist", ["saved_maze"].to_vec(), false);
    do_press_any_key_quit_run_and_verify(&mut mock_app, &mut expected_output)?;
    Ok(())
}
