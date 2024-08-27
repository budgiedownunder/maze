mod mock_app;
use crate::mock_app::MockApp;
use maze_cli::app::App;
use std::io::{self};

#[test]
fn should_be_able_to_quit_on_start() -> Result<(), io::Error> {
    let mut mock_app = MockApp::new();
    let expected_output = ["Exiting..."];
    mock_app.add_input_key('Q', true);
    mock_app.run()?;
    mock_app.print_output();
    mock_app.verify_output(&expected_output)?;
    Ok(())
}

#[test]
fn should_be_able_to_enter_text_and_then_quit() -> Result<(), io::Error> {
    let mut mock_app = MockApp::new();
    let expected_output = [
        "Enter some text: ",
        "You entered: Some test text",
        "Exiting...",
    ];
    mock_app.add_input_key('E', true);
    mock_app.add_input_line("Some test text", false);
    mock_app.add_input_key('Q', false);
    mock_app.run()?;
    mock_app.print_output();
    mock_app.verify_output(&expected_output)?;
    Ok(())
}
