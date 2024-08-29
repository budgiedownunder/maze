use maze::LinePrinter;
use maze::Maze;
use maze::Path;
use maze::Point;

use std::error::Error;
use std::io::{self};
use std::thread;
use std::time::Duration;

static WELCOME_BANNER: &str = r#"*********************************************
       *           Welcome to the Maze CLI !!      *
       *********************************************
    "#;

static MENU: &str = r#"*********************************************
        Select action:
        
        I -> Insert rows    | D -> Delete rows
        N -> Insert columns | L -> Delete columns
        W -> Set walls      | C -> Clear walls
        R -> Resize         | E -> Empty
        -------------------------------------------
        S -> Solve          | P -> Print
        -------------------------------------------
        Q -> Quit
        *********************************************
        "#;

static PRESS_ANY_KEY_TEXT: &str = "\n[** Press any key **]\n";
pub trait App: LinePrinter {
    fn get_maze(&self) -> &Maze;
    fn get_maze_mut(&mut self) -> &mut Maze;
    fn read_key(&mut self) -> Result<Option<char>, io::Error>;
    fn read_line(&mut self) -> Result<Option<String>, io::Error>;

    fn print_lines(&mut self, lines: Vec<&'static str>) -> Result<(), Box<dyn Error>> {
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

    fn print_welcome_banner(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_lines(Self::get_welcome_banner_lines())
    }

    fn get_menu_lines() -> Vec<&'static str> {
        Self::str_to_lines(MENU)
    }

    fn print_menu(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_lines(Self::get_menu_lines())?;
        Ok(())
    }

    fn get_press_any_key_text() -> &'static str {
        PRESS_ANY_KEY_TEXT
    }
    fn press_any_key(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_line(Self::get_press_any_key_text())?;
        self.read_key()?;
        Ok(())
    }

    fn prompt_yes_no(&mut self, message: &str) -> Result<bool, Box<dyn Error>> {
        self.print_line(&format!("{} (Y/N)", message))?;
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
                        self.print_line(&format!("Invalid response: '{}'", ch))?;
                    }
                },
                None => {
                    self.print_line("No action taken")?;
                }
            }
        }
    }

    fn prompt_text(&mut self, message: &str) -> Result<String, Box<dyn Error>> {
        self.print_line(message)?;
        loop {
            match self.read_line()? {
                Some(line) => {
                    return Ok(line);
                }
                None => {
                    self.print_line("Please enter a value")?;
                }
            }
        }
    }

    fn prompt_number(&mut self, message: &str) -> Result<usize, Box<dyn Error>> {
        loop {
            let text = self.prompt_text(message)?;
            match text.trim().parse::<usize>() {
                Ok(num) => {
                    return Ok(num);
                }
                Err(_) => {
                    self.print_line(
                        "Invalid number, please enter an integer value greater or equal to zero",
                    )?;
                }
            }
        }
    }

    fn print_maze_dimensions(&mut self, prefix: &str) -> Result<(), Box<dyn Error>> {
        let (rows, cols) = {
            let maze = self.get_maze();
            (maze.definition.row_count(), maze.definition.col_count())
        };
        let message = format!("{} dimensions: {} row(s), {} column(s)", prefix, rows, cols);
        self.print_line(&message)?;
        Ok(())
    }

    fn do_insert_rows(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_line("Insert rows")?;
        Ok(())
    }

    fn do_delete_rows(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_line("Delete rows")?;
        Ok(())
    }

    fn do_insert_cols(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_line("Insert columns")?;
        Ok(())
    }

    fn do_delete_cols(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_line("Delete columns")?;
        Ok(())
    }

    fn do_set_walls(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_line("Set walls")?;
        Ok(())
    }

    fn do_clear_walls(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_line("Clear walls")?;
        Ok(())
    }

    fn do_resize(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_maze_dimensions("Current")?;
        let new_row_count = self.prompt_number("Enter new row count: ")?;
        let new_col_count = self.prompt_number("Enter new column count: ")?;
        self.get_maze_mut()
            .definition
            .resize(new_row_count, new_col_count);
        self.print_maze_dimensions("New")?;
        Ok(())
    }

    fn do_empty(&mut self) -> Result<(), Box<dyn Error>> {
        let (rows, cols) = {
            let maze = self.get_maze();
            (maze.definition.row_count(), maze.definition.col_count())
        };
        let message = format!(
            "Set maze to empty? [current dimensions: {} row(s), {} column(s)]",
            rows, cols
        );
        let choice = self.prompt_yes_no(&message)?;
        if choice {
            self.get_maze_mut().reset();
            self.print_line("Maze set to empty")?;
        } else {
            self.print_line("Maze was not changed")?;
        }
        Ok(())
    }

    fn get_line_printer(&mut self) -> &mut dyn LinePrinter;

    fn print_maze(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_maze_dimensions("Current")?;
        self.print_line("\nDefinition:\n")?;
        let maze = self.get_maze().clone();
        let start = Point { row: 0, col: 0 };
        let end = Point { row: 0, col: 0 };
        let path = Path { points: vec![] };
        let print_target = self.get_line_printer();
        if maze.definition.is_empty() {
            print_target.print_line("Maze is empty")?;
        } else if let Err(error) = maze.print(print_target, start, end, path) {
            print_target.print_line(&format!("Failed to print matrix: {}", error))?;
        }
        Ok(())
    }

    fn do_print(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_maze()?;
        Ok(())
    }

    fn do_solve(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_line("Do solve")?;
        Ok(())
    }

    fn process_keys(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            if let Some(ch) = self.read_key()? {
                match ch.to_ascii_uppercase() {
                    'I' => self.do_insert_rows()?,
                    'D' => self.do_delete_rows()?,
                    'N' => self.do_insert_cols()?,
                    'L' => self.do_delete_cols()?,
                    'W' => self.do_set_walls()?,
                    'C' => self.do_clear_walls()?,
                    'R' => self.do_resize()?,
                    'E' => self.do_empty()?,
                    'S' => self.do_solve()?,
                    'P' => self.do_print()?,
                    'Q' => {
                        self.print_line("Exiting...")?;
                        return Ok(());
                    }
                    _ => {
                        self.print_line(&format!("Unknown option selected: {}", ch))?;
                        continue;
                    }
                }
                self.press_any_key()?;
                self.print_menu()?;
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_welcome_banner()?;
        self.print_menu()?;
        self.process_keys()?;
        Ok(())
    }
}
