mod mock_app;
use crate::mock_app::MockApp;
use maze::Definition;
use maze::Maze;
use maze_cli::app::App;
use std::error::Error;

#[test]
fn should_be_able_to_quit_on_start() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let expected_output = vec!["Exiting..."];
    mock_app.add_input_key('Q', true);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_insert_rows_into_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output = vec![
        "Current dimensions: 0 row(s), 0 column(s)",
        "Number rows to insert: ",
        "Success - new dimensions: 5 row(s), 0 column(s)",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('I', true);
    mock_app.add_input_line("5", false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_prevent_insert_invalid_rows_into_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output = vec![
        "Current dimensions: 0 row(s), 0 column(s)",
        "Number rows to insert: ",
        "Invalid value 'B' (out of bounds), please enter an integer value greater or equal to 0",
        "Number rows to insert: ",
        "Invalid value '-2' (out of bounds), please enter an integer value greater or equal to 0",
        "Number rows to insert: ",
        "Success - new dimensions: 5 row(s), 0 column(s)",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('I', true);
    mock_app.add_input_line("B", false);
    mock_app.add_input_line("-2", false);
    mock_app.add_input_line("5", false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_prevent_insert_invalid_rows_into_non_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output = vec![
        "Current dimensions: 10 row(s), 5 column(s)",
        "Insert at row: ",
        "Invalid value 'A' (out of bounds), please enter an integer value between 1 and 11 (inclusive)",
        "Insert at row: ",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 11 (inclusive)",
        "Insert at row: ",
        "Invalid value '12' (out of bounds), please enter an integer value between 1 and 11 (inclusive)",
        "Insert at row: ",
        "Number rows to insert: ",
        "Invalid value 'B' (out of bounds), please enter an integer value greater or equal to 0",
        "Number rows to insert: ",
        "Invalid value '-2' (out of bounds), please enter an integer value greater or equal to 0",
        "Number rows to insert: ",
        "Success - new dimensions: 15 row(s), 5 column(s)",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('I', true);
    mock_app.add_input_line("A", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("12", false);
    mock_app.add_input_line("1", false);
    mock_app.add_input_line("B", false);
    mock_app.add_input_line("-2", false);
    mock_app.add_input_line("5", false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_delete_rows_from_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output = vec![
        "Current dimensions: 0 row(s), 0 column(s)",
        "Definition is empty - no rows to delete",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('D', true);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_delete_invalid_rows_from_non_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output = vec![
        "Current dimensions: 10 row(s), 5 column(s)",
        "Delete rows from: ",
        "Invalid value 'A' (out of bounds), please enter an integer value between 1 and 10 (inclusive)",
        "Delete rows from: ",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 10 (inclusive)",
        "Delete rows from: ",
        "Invalid value '11' (out of bounds), please enter an integer value between 1 and 10 (inclusive)",
        "Delete rows from: ",
        "Number rows to delete: ",
        "Invalid value 'A' (out of bounds), please enter an integer value between 1 and 8 (inclusive)",
        "Number rows to delete: ",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 8 (inclusive)",
        "Number rows to delete: ",
        "Invalid value '11' (out of bounds), please enter an integer value between 1 and 8 (inclusive)",
        "Number rows to delete: ",
        "Success - new dimensions: 6 row(s), 5 column(s)",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('D', true);
    mock_app.add_input_line("A", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("11", false);
    mock_app.add_input_line("3", false);
    mock_app.add_input_line("A", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("11", false);
    mock_app.add_input_line("4", false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_insert_cols_into_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output = vec![
        "Current dimensions: 0 row(s), 0 column(s)",
        "Definition is empty - insert some rows before adding columns",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('N', true);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_prevent_insert_invalid_cols_into_non_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output = vec![
        "Current dimensions: 10 row(s), 5 column(s)",
        "Insert at column: ",
        "Invalid value 'A' (out of bounds), please enter an integer value between 1 and 6 (inclusive)",
        "Insert at column: ",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 6 (inclusive)",
        "Insert at column: ",
        "Invalid value '12' (out of bounds), please enter an integer value between 1 and 6 (inclusive)",
        "Insert at column: ",
        "Number columns to insert: ",
        "Invalid value 'B' (out of bounds), please enter an integer value greater or equal to 0",
        "Number columns to insert: ",
        "Invalid value '-2' (out of bounds), please enter an integer value greater or equal to 0",
        "Number columns to insert: ",
        "Success - new dimensions: 10 row(s), 12 column(s)",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('N', true);
    mock_app.add_input_line("A", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("12", false);
    mock_app.add_input_line("5", false);
    mock_app.add_input_line("B", false);
    mock_app.add_input_line("-2", false);
    mock_app.add_input_line("7", false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_delete_cols_from_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output = vec![
        "Current dimensions: 0 row(s), 0 column(s)",
        "Definition has no columns to delete",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('L', true);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_not_be_able_to_delete_invalid_cols_from_non_empty_maze() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output = vec![
        "Current dimensions: 10 row(s), 5 column(s)",
        "Delete columns from: ",
        "Invalid value 'A' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "Delete columns from: ",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "Delete columns from: ",
        "Invalid value '6' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "Delete columns from: ",
        "Number columns to delete: ",
        "Invalid value 'A' (out of bounds), please enter an integer value between 1 and 2 (inclusive)",
        "Number columns to delete: ",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 2 (inclusive)",
        "Number columns to delete: ",
        "Invalid value '4' (out of bounds), please enter an integer value between 1 and 2 (inclusive)",
        "Number columns to delete: ",
        "Success - new dimensions: 10 row(s), 3 column(s)",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('L', true);
    mock_app.add_input_line("A", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("6", false);
    mock_app.add_input_line("4", false);
    mock_app.add_input_line("A", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("4", false);
    mock_app.add_input_line("2", false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

fn run_set_endpoint_test_in_empty_maze(
    operation_key: char,
    name: &str,
) -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let operation_message = format!("Set {}", name);
    let expected_error_message = &format!(
        "Maze has no cells - add some rows and columns first before setting the {} cell",
        name
    );
    let mut expected_output = vec![
        operation_message.as_str(),
        "Current dimensions: 0 row(s), 0 column(s)",
        expected_error_message,
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key(operation_key, true);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
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
    let mut expected_output:Vec<&str> = vec![
        operation,
        "Current dimensions: 3 row(s), 5 column(s)",
        "Row:",
        "Invalid value 'A' (out of bounds), please enter an integer value between 1 and 3 (inclusive)",
        "Row:",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 3 (inclusive)",
        "Row:",
        "Invalid value '11' (out of bounds), please enter an integer value between 1 and 3 (inclusive)",
        "Row:",
        "Column:",
        "Invalid value 'B' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "Column:",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "Column:",
        "Invalid value '6' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "Column:",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Current dimensions: 3 row(s), 5 column(s)");
    expected_output.push("\nDefinition:\n");
    expected_output.push("░░░░░");
    expected_output.push(&modified_row);
    expected_output.push("░░░░░");
    expected_output.push(MockApp::get_press_any_key_text());
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key(operation_key, true);
    mock_app.add_input_line("A", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("11", false);
    mock_app.add_input_line("2", false);
    mock_app.add_input_line("B", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("6", false);
    mock_app.add_input_line("3", false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('P', false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
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
    let mut expected_output = vec![
        "Set walls",
        "Current dimensions: 0 row(s), 0 column(s)",
        "Maze has no cells - add some rows and columns first before modifying walls",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('W', true);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
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
    let mut expected_output:Vec<&str> = vec![
        operation,
        "Current dimensions: 10 row(s), 5 column(s)",
        "Start row:",
        "Invalid value 'A' (out of bounds), please enter an integer value between 1 and 10 (inclusive)",
        "Start row:",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 10 (inclusive)",
        "Start row:",
        "Invalid value '11' (out of bounds), please enter an integer value between 1 and 10 (inclusive)",
        "Start row:",
        "Start column:",
        "Invalid value 'B' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "Start column:",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "Start column:",
        "Invalid value '6' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "Start column:",
        "End row:",
        "Invalid value 'C' (out of bounds), please enter an integer value between 1 and 10 (inclusive)",
        "End row:",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 10 (inclusive)",
        "End row:",
        "Invalid value '11' (out of bounds), please enter an integer value between 1 and 10 (inclusive)",
        "End row:",
        "End column:",
        "Invalid value 'D' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "End column:",
        "Invalid value '-1' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "End column:",
        "Invalid value '6' (out of bounds), please enter an integer value between 1 and 5 (inclusive)",
        "End column:",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Current dimensions: 10 row(s), 5 column(s)");
    expected_output.push("\nDefinition:\n");
    expected_output.push("░░░░░");
    expected_output.push("░░░░░");
    expected_output.push(&modified_row);
    expected_output.push(&modified_row);
    expected_output.push(&modified_row);
    expected_output.push("░░░░░");
    expected_output.push("░░░░░");
    expected_output.push("░░░░░");
    expected_output.push("░░░░░");
    expected_output.push("░░░░░");
    expected_output.push(MockApp::get_press_any_key_text());
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key(operation_key, true);
    mock_app.add_input_line("A", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("11", false);
    mock_app.add_input_line("3", false);
    mock_app.add_input_line("B", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("6", false);
    mock_app.add_input_line("2", false);
    mock_app.add_input_line("C", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("11", false);
    mock_app.add_input_line("5", false);
    mock_app.add_input_line("D", false);
    mock_app.add_input_line("-1", false);
    mock_app.add_input_line("6", false);
    mock_app.add_input_line("4", false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('P', false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
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
    let mut expected_output = vec![
        "Clear walls",
        "Current dimensions: 0 row(s), 0 column(s)",
        "Maze has no cells - add some rows and columns first before modifying walls",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('C', true);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_clear_walls_in_non_empty_maze() -> Result<(), Box<dyn Error>> {
    run_modify_walls_test('C', "Clear walls", '░')?;
    Ok(())
}

#[test]
fn should_be_able_to_resize_maze_and_then_quit() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output = vec![
        "Current dimensions: 0 row(s), 0 column(s)",
        "Enter new row count: ",
        "Enter new column count: ",
        "Success - new dimensions: 5 row(s), 10 column(s)",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('R', true);
    mock_app.add_input_line("5", false);
    mock_app.add_input_line("10", false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_empty_maze_and_then_quit() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output = vec![
        "Set maze to empty? [current dimensions: 10 row(s), 5 column(s)] (Y/N)",
        "Maze set to empty",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('E', true);
    mock_app.add_input_key('Y', false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_print_empty_maze_and_then_quit() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(0, 0));
    let mut expected_output = vec![
        "Current dimensions: 0 row(s), 0 column(s)",
        "\nDefinition:\n",
        "Maze is empty",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('P', true);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_print_maze_with_content_and_then_quit() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(2, 3));
    let mut expected_output = vec![
        "Current dimensions: 2 row(s), 3 column(s)",
        "\nDefinition:\n",
        "░░░",
        "░░░",
        MockApp::get_press_any_key_text(),
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('P', true);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}
