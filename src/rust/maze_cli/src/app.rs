use maze::LinePrinter;
use maze::Maze;
use maze::Path;
use maze::Point;

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
        P -> Print
        Q -> Quit
        ******************************
        "#;

pub trait App: LinePrinter {
    fn get_maze(&self) -> &Maze;
    fn get_maze_mut(&mut self) -> &mut Maze;
    fn read_key(&mut self) -> Result<Option<char>, io::Error>;
    fn read_line(&mut self) -> Result<Option<String>, io::Error>;

    fn print_lines(&mut self, lines: Vec<&'static str>) -> Result<(), io::Error> {
        for line in lines {
            self.print_line(line)?;
        }
        Ok(())
    }

    fn str_to_lines(s: &'static str) -> Vec<&'static str> {
        s.lines().map(|line| line.trim_start()).collect()
    }

    fn get_welcome_banner_lines() -> Vec<&'static str> {
        Self::str_to_lines(WELCOME_BANNER)
    }

    fn print_welcome_banner(&mut self) -> Result<(), io::Error> {
        self.print_lines(Self::get_welcome_banner_lines())
    }

    fn get_menu_lines() -> Vec<&'static str> {
        Self::str_to_lines(MENU)
    }

    fn print_menu(&mut self) -> Result<(), io::Error> {
        self.print_lines(Self::get_menu_lines())?;
        Ok(())
    }

    fn press_any_key(&mut self) -> Result<(), io::Error> {
        self.print_line("\n[** Press any key **]\n")?;
        self.read_key()?;
        Ok(())
    }

    fn choose_yes_no(&mut self, message: &str) -> Result<bool, io::Error> {
        self.print_line(format!("{} (Y/N)", message).as_str())?;
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
                        self.print_line(format!("Invalid response: '{}'", ch).as_str())?;
                    }
                },
                None => {
                    self.print_line("No action taken")?;
                }
            }
        }
    }

    fn handle_reset(&mut self) -> Result<(), io::Error> {
        let (rows, cols) = {
            let maze = self.get_maze();
            (maze.definition.row_count(), maze.definition.col_count())
        };
        let message = format!(
            "Reset maze to empty? [current dimensions: {} rows, {} columns]",
            rows, cols
        );
        let choice = self.choose_yes_no(message.as_str())?;
        if choice {
            self.get_maze_mut().reset();
            self.print_line("Maze reset to empty")?;
        } else {
            self.print_line("Maze was not changed")?;
        }
        self.press_any_key()?;
        Ok(())
    }

    fn get_line_printer(&mut self) -> &mut dyn LinePrinter;

    fn print_maze(&mut self) -> Result<(), io::Error> {
        let maze = self.get_maze().clone();
        let start = Point { row: 0, col: 0 };
        let end = Point { row: 0, col: 0 };
        let path = Path { points: vec![] };
        let print_target = self.get_line_printer();
        if maze.definition.is_empty() {
            print_target.print_line("Maze is empty")?;
        } else if let Err(error) = maze.print(print_target, start, end, path) {
            print_target.print_line(format!("Failed to print matrix: {}", error).as_str())?;
        }
        Ok(())
    }

    fn handle_print(&mut self) -> Result<(), io::Error> {
        self.print_maze()?;
        self.press_any_key()?;
        Ok(())
    }

    fn process_keys(&mut self) -> Result<(), io::Error> {
        loop {
            match self.read_key()? {
                Some(ch) => match ch.to_ascii_uppercase() {
                    'R' => {
                        self.handle_reset()?;
                        self.print_menu()?;
                    }
                    'P' => {
                        self.handle_print()?;
                        self.print_menu()?;
                    }
                    'Q' => {
                        self.print_line("Exiting...")?;
                        return Ok(());
                    }
                    'E' => {
                        self.print_line("Enter some text: ")?;
                        match self.read_line()? {
                            Some(line) => {
                                self.print_line(format!("You entered: {}", line).as_str())?;
                            }
                            None => {
                                self.print_line("No text entered")?;
                            }
                        }
                    }
                    _ => self.print_line(format!("Unknown option selected: {}", ch).as_str())?,
                },
                None => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }

    fn run(&mut self) -> Result<(), io::Error> {
        self.print_welcome_banner()?;
        self.print_menu()?;
        self.process_keys()?;
        Ok(())
    }
}
