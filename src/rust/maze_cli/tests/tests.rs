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
fn should_be_able_to_resize_maze_and_then_quit() -> Result<(), Box<dyn Error>> {
    let mut mock_app = MockApp::new();
    let mut expected_output = vec![
        "Current dimensions: 0 row(s), 0 column(s)",
        "Enter new row count: ",
        "Enter new column count: ",
        "New dimensions: 5 row(s), 10 column(s)",
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
        "F░░",
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
