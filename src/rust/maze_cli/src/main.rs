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

        P -> Print maze
        Q -> Quit
        ******************************
        "#;

trait App {
    fn read_key(&mut self) -> Result<Option<char>, io::Error>;
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
                        return Ok(())
                    }
                    'P' => {
                        self.write_line("Would print")?;
                    }
                    _ => self.write_line(format!("Unknown option selected: {}", ch).as_str())?,
                },
                None => {
                    thread::sleep(Duration::from_millis(10));
                },
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
mod tests {
    use crate::App;
    use std::io::{self};
    use std::collections::VecDeque;

    struct MockApp {
        input_keys: VecDeque<char>,
        output: Vec<String>,
    }

    impl MockApp {
        fn new() -> MockApp {
            MockApp {
                input_keys: VecDeque::new(),
                output: Vec::new(),
            }
        }

        fn add_input_key(&mut self, key: char) {
            self.input_keys.push_back(key);
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
                Some(ch) => Ok(Some(ch)),
                None => {
                    self.write_line("No key presses found in input_keys buffer")?;
                    Err(io::Error::new(io::ErrorKind::Other, "No key presses found in input_keys buffer"))
                },    
            }
        }
        fn write_line(&mut self, line: &str) -> Result<(), io::Error> {
            self.output.push(line.to_string());
            Ok(())
        }
    }

    #[test]
    fn should_be_able_quit_on_start() -> Result<(), io::Error> {
        let mut mock_app = MockApp::new();
        mock_app.add_input_key('Q');
        mock_app.run()?;
        mock_app.print_output();
        Ok(())
    }
}
