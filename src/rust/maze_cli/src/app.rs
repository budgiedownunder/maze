use maze::Maze;
use std::io::{self};
use std::thread;
use std::time::Duration;

static WELCOME_BANNER: &str = r#"******************************
       * Welcome to the Maze CLI !! *
       ******************************
    "#;

static MENU: &str = r#"******************************
        Select action:

        E -> Enter text
        R -> Reset to empty
        Q -> Quit
        ******************************
        "#;

pub trait App {
    fn get_current_maze(&mut self) -> &mut Maze;
    fn read_key(&mut self) -> Result<Option<char>, io::Error>;
    fn read_line(&mut self) -> Result<Option<String>, io::Error>;
    fn write_line(&mut self, line: &str) -> Result<(), io::Error>;

    fn write_lines(&mut self, lines: Vec<&'static str>) -> Result<(), io::Error> {
        for line in lines {
            self.write_line(line)?;
        }
        Ok(())
    }

    fn str_lines(s: &'static str) -> Vec<&'static str> {
        s.lines().map(|line| line.trim_start()).collect()
    }

    fn get_welcome_banner_lines() -> Vec<&'static str> {
        Self::str_lines(WELCOME_BANNER)
    }

    fn write_welcome_banner(&mut self) -> Result<(), io::Error> {
        self.write_lines(Self::get_welcome_banner_lines())
    }

    fn get_menu_lines() -> Vec<&'static str> {
        Self::str_lines(MENU)
    }

    fn write_menu(&mut self) -> Result<(), io::Error> {
        self.write_lines(Self::get_menu_lines())?;
        Ok(())
    }

    fn press_any_key(&mut self) -> Result<(), io::Error> {
        self.write_line("[** Press any key **]")?;
        self.read_key()?;
        Ok(())
    }

    fn choose_yes_no(&mut self, message: &str) -> Result<bool, io::Error> {
        self.write_line(format!("{} (Y/N)", message).as_str())?;
        loop {
            match self.read_key()? {
                Some(ch) => match ch.to_ascii_uppercase() {
                    'Y' => {
                        return Ok(true);
                    }
                    'N' => {
                        return Ok(false);
                    }
                    _ => {
                        self.write_line(format!("Invalid response: '{}'", ch).as_str())?;
                    }
                },
                None => {
                    self.write_line("No action taken")?;
                }
            }
        }
    }

    fn handle_reset(&mut self) -> Result<(), io::Error> {
        let (rows, cols) = {
            let maze = self.get_current_maze();
            (maze.definition.row_count(), maze.definition.col_count())
        };
        let message = format!(
            "Reset maze to empty? [current dimensions: {} rows, {} columns]",
            rows, cols
        );
        let choice = self.choose_yes_no(message.as_str())?;
        if choice {
            self.get_current_maze().reset();
            self.write_line("Maze reset to empty")?;
        } else {
            self.write_line("Maze was not changed")?;
        }
        self.press_any_key()?;
        Ok(())
    }

    fn process_keys(&mut self) -> Result<(), io::Error> {
        loop {
            match self.read_key()? {
                Some(ch) => match ch.to_ascii_uppercase() {
                    'R' => {
                        self.handle_reset()?;
                        self.write_menu()?;
                    }
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
