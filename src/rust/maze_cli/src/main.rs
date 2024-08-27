use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use std::io::{self /*, stdout, Stdout*/};
use std::thread;
use std::time::Duration;

use maze::Definition;
use maze::Maze;

static WELCOME_BANNER: &'static str = r#"******************************
       * Welcome to the Maze CLI !! *
       ******************************
    "#;

static MENU: &'static str = r#"******************************
        Select action:

        E -> Enter text
        Q -> Quit
        ******************************
        "#;

trait App {
    fn read_key(&mut self) -> Result<Option<char>, io::Error>;
    fn read_line(&mut self) -> Result<Option<String>, io::Error>;
    fn write_line(&mut self, line: &str) -> Result<(), io::Error>;

    fn write_lines(&mut self, lines: Vec<&'static str>) -> Result<(), io::Error> {
        for line in lines {
            self.write_line(line)?;
        }
        Ok(())
    }

    fn str_lines(&mut self, s: &'static str) -> Vec<&'static str> {
        s.lines().map(|line| line.trim_start()).collect()
    }

    fn welcome_banner_lines(&mut self) -> Vec<&'static str> {
        self.str_lines(WELCOME_BANNER)
    }

    fn write_welcome_banner(&mut self) -> Result<(), io::Error> {
        let lines = self.welcome_banner_lines();
        self.write_lines(lines)
    }

    fn menu_lines(&mut self) -> Vec<&'static str> {
        self.str_lines(MENU)
    }

    fn write_menu(&mut self) -> Result<(), io::Error> {
        let lines = self.menu_lines();
        self.write_lines(lines)?;
        Ok(())
    }

    fn process_keys(&mut self) -> Result<(), io::Error> {
        let mut _m: Maze = Maze::new(Definition::new(0, 0));
        loop {
            match self.read_key()? {
                Some(ch) => match ch.to_ascii_uppercase() {
                    'Q' => {
                        self.write_line("Exiting...")?;
                        return Ok(());
                    }
                    'E' => {
                        self.write_line("Enter some text: ")?;
                        match self.read_line()? {
                            Some(line) => {
                                self.write_line(format!("You entered: {}", line).as_str())?;
                            }
                            None => {
                                self.write_line("No text entered")?;
                            }
                        }
                    }
                    _ => self.write_line(format!("Unknown option selected: {}", ch).as_str())?,
                },
                None => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }

    fn run(&mut self) -> Result<(), io::Error> {
        self.write_welcome_banner()?;
        self.write_menu()?;
        self.process_keys()?;
        Ok(())
    }
}

struct ConsoleApp {}

impl ConsoleApp {}

impl App for ConsoleApp {
    fn read_key(&mut self) -> Result<Option<char>, io::Error> {
        loop {
            if poll(Duration::from_secs(0))? {
                if let Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    ..
                }) = read()?
                {
                    if modifiers.is_empty() && kind == crossterm::event::KeyEventKind::Press {
                        if let KeyCode::Char(ch) = code {
                            return Ok(Some(ch));
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn read_line(&mut self) -> Result<Option<String>, io::Error> {
        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;
        if bytes_read == 0 {
            Ok(None)
        } else {
            Ok(Some(input))
        }
    }

    fn write_line(&mut self, line: &str) -> Result<(), io::Error> {
        println!("{}", line);
        Ok(())
    }
}

// Helper functions
//fn app_name() -> &'static str {
//  env!("CARGO_PKG_NAME")
//}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = ConsoleApp {};
    app.run()?;
    Ok(())
}

#[cfg(test)]
mod maze_cli_tests {
    use crate::App;
    use std::collections::VecDeque;
    use std::io::{self};

    struct MockInputKey {
        key: char,
        reset_output: bool,
    }

    struct MockInputLine {
        text: String,
        reset_output: bool,
    }

    struct MockApp {
        input_keys: VecDeque<MockInputKey>,
        input_lines: VecDeque<MockInputLine>,
        output: Vec<String>,
    }

    impl MockApp {
        fn new() -> MockApp {
            MockApp {
                input_keys: VecDeque::new(),
                input_lines: VecDeque::new(),
                output: Vec::new(),
            }
        }

        fn add_input_key(&mut self, key: char, reset_output: bool) {
            self.input_keys.push_back(MockInputKey {
                key: key,
                reset_output: reset_output,
            });
        }

        fn add_input_line(&mut self, text: &str, reset_output: bool) {
            self.input_lines.push_back(MockInputLine {
                text: text.to_string(),
                reset_output: reset_output,
            });
        }

        fn io_error(message: String) -> io::Error {
            io::Error::new(io::ErrorKind::Other, message)
        }

        fn verify_output(&self, expected: &[&str]) -> Result<(), io::Error> {
            if expected.len() != self.output.len() {
                return Err(Self::io_error(
                    format!("The output and expected lines differ in length. Expected has length {}, while output has length {}.",
                    expected.len(),
                    self.output.len() )                    
                ));
            }
            for i in 0..expected.len() {
                if expected[i] != self.output[i] {
                    return Err(Self::io_error(
                        format!("Difference found in output at index {}: expected[{}] = '{}', output[{}] = '{}'", i, i, expected[i], i, self.output[i])
                    ));
                }
            }
            Ok(())
        }

        fn print_output(&self) {
            for line in &self.output {
                println!("{}", line);
            }
        }
    }

    impl App for MockApp {
        fn read_key(&mut self) -> Result<Option<char>, io::Error> {
            match self.input_keys.pop_front() {
                Some(input_key) => {
                    if input_key.reset_output {
                        self.output.clear();
                    }
                    Ok(Some(input_key.key))
                }
                None => return Err(Self::io_error("No key presses found in input_keys buffer".to_string())),
            }
        }

        fn read_line(&mut self) -> Result<Option<String>, io::Error> {
            match self.input_lines.pop_front() {
                Some(input_line) => {
                    if input_line.reset_output {
                        self.output.clear();
                    }
                    Ok(Some(input_line.text))
                }
                None => return Err(Self::io_error("No lines found in input_lines buffer".to_string())),
            }
        }

        fn write_line(&mut self, line: &str) -> Result<(), io::Error> {
            self.output.push(line.to_string());
            Ok(())
        }
    }

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
        let expected_output = ["Enter some text: ", "You entered: Some test text", "Exiting..."];
        mock_app.add_input_key('E', true);
        mock_app.add_input_line("Some test text", false);
        mock_app.add_input_key('Q', false);
        mock_app.run()?;
        mock_app.print_output();
        mock_app.verify_output(&expected_output)?;
        Ok(())
    }
}
