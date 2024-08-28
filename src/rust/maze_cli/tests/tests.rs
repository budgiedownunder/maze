mod mock_app;
use crate::mock_app::MockApp;
use maze_cli::app::App;
use maze::Maze;
use maze::Definition;
use std::io::{self};

#[test]
fn should_be_able_to_quit_on_start() -> Result<(), io::Error> {
    let mut mock_app = MockApp::new();
    let expected_output = vec!["Exiting..."];
    mock_app.add_input_key('Q', true);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_enter_text_and_then_quit() -> Result<(), io::Error> {
    let mut mock_app = MockApp::new();
    let expected_output = vec![
        "Enter some text: ",
        "You entered: Some test text",
        "Exiting...",
    ];
    mock_app.add_input_key('E', true);
    mock_app.add_input_line("Some test text", false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_reset_maze_and_quit() -> Result<(), io::Error> {
    let mut mock_app = MockApp::new();
    mock_app.current_maze = Maze::new(Definition::new(10, 5));
    let mut expected_output = vec![
        "Reset maze to empty? [current dimensions: 10 rows, 5 columns] (Y/N)",
        "Maze reset to empty",
        "[** Press any key **]",
    ];
    expected_output.extend(MockApp::get_menu_lines());
    expected_output.push("Exiting...");

    mock_app.add_input_key('R', true);
    mock_app.add_input_key('Y', false);
    mock_app.add_input_key(' ', false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.verify_output(expected_output)?;
    Ok(())
}
