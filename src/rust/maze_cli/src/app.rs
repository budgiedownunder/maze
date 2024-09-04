use maze::LinePrinter;
use maze::Maze;
use maze::Path;
use maze::Point;

use std::error::Error;
use std::io::{self};
use std::path::{Path as stdPath, PathBuf};
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
        -------------------------------------------
        A -> Set start      | F -> Set finish
        W -> Set walls      | C -> Clear walls
        -------------------------------------------
        R -> Resize         | E -> Empty
        -------------------------------------------
        S -> Solve          | P -> Print
        -------------------------------------------
        O -> Open
        V -> Save           | Z -> Save As
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
    fn get_maze_name(&self) -> String;
    fn set_maze_name(&mut self, name: &str) -> Result<(), Box<dyn Error>>;

    fn get_maze_storage_id(name: &str) -> String {
        format!("{}.json", name.trim())
    }

    fn maze_name_exists(name: &str) -> bool {
        let path = PathBuf::from(Self::get_maze_storage_id(name));
        !name.is_empty() && stdPath::new(&path).exists()
    }

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
            if let Some(line) = self.read_line()? {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
                self.print_line("Please enter a value")?;
            }
        }
    }

    fn prompt_number(
        &mut self,
        message: &str,
        min_limit: Option<usize>,
        max_limit: Option<usize>,
    ) -> Result<usize, Box<dyn Error>> {
        fn range_summary(min_limit: Option<usize>, max_limit: Option<usize>) -> String {
            let (have_min, have_max) = (min_limit.is_some(), max_limit.is_some());
            if have_min && have_max {
                if min_limit.unwrap() != max_limit.unwrap() {
                    format!(
                        "between {} and {} (inclusive)",
                        min_limit.unwrap(),
                        max_limit.unwrap()
                    )
                } else {
                    format!("equal to {}", min_limit.unwrap())
                }
            } else if have_min {
                format!("greater or equal to {}", min_limit.unwrap())
            } else if have_max {
                format!("between zero and {} (inclusive)", max_limit.unwrap())
            } else {
                "greater or equal to 0".to_string()
            }
        }

        fn error_message<T: std::fmt::Display>(
            value: T,
            min_limit: Option<usize>,
            max_limit: Option<usize>,
        ) -> String {
            format!(
                "Invalid value '{}' (out of bounds), please enter an integer value {}",
                value,
                range_summary(min_limit, max_limit)
            )
        }

        fn value_error<T: std::fmt::Display>(
            value: T,
            min_limit: Option<usize>,
            max_limit: Option<usize>,
        ) -> Result<(), String> {
            Err(error_message(value, min_limit, max_limit))
        }

        fn validate_number(
            num: usize,
            min_limit: Option<usize>,
            max_limit: Option<usize>,
        ) -> Result<(), String> {
            if let Some(min_allowed) = min_limit {
                if num < min_allowed {
                    return value_error(num, min_limit, max_limit);
                }
            }
            if let Some(max_allowed) = max_limit {
                if num > max_allowed {
                    return value_error(num, min_limit, max_limit);
                }
            }
            Ok(())
        }

        loop {
            let text = self.prompt_text(message)?;
            match text.trim().parse::<usize>() {
                Ok(num) => match validate_number(num, min_limit, max_limit) {
                    Ok(_) => return Ok(num),
                    Err(err_msg) => self.print_line(&err_msg)?,
                },
                Err(_) => self.print_line(&error_message(text.trim(), min_limit, max_limit))?,
            }
        }
    }

    fn print_maze_dimensions(&mut self, prefix: &str) -> Result<(), Box<dyn Error>> {
        let (num_rows, num_cols) = {
            let maze = self.get_maze();
            (maze.definition.row_count(), maze.definition.col_count())
        };
        let message = format!(
            "{} dimensions: {} row(s), {} column(s)",
            prefix, num_rows, num_cols
        );
        self.print_line(&message)?;
        Ok(())
    }

    fn do_insert_rows(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_maze_dimensions("Current")?;
        let current_rows = self.get_maze().definition.row_count();
        let mut start_row = 1;
        if current_rows > 1 {
            start_row = self.prompt_number("Insert at row: ", Some(1), Some(1 + current_rows))?;
        }
        let num_rows = self.prompt_number("Number rows to insert: ", None, None)?;
        self.get_maze_mut()
            .definition
            .insert_rows(start_row - 1, num_rows)?;
        self.print_maze_dimensions("Success - new")?;
        Ok(())
    }

    fn do_delete_rows(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_maze_dimensions("Current")?;
        let current_rows = self.get_maze().definition.row_count();
        if current_rows == 0 {
            self.print_line("Definition is empty - no rows to delete")?;
            return Ok(());
        }
        let start_row = self.prompt_number("Delete rows from: ", Some(1), Some(current_rows))?;
        let num_rows = self.prompt_number(
            "Number rows to delete: ",
            Some(1),
            Some(current_rows - start_row + 1),
        )?;
        self.get_maze_mut()
            .definition
            .delete_rows(start_row - 1, num_rows)?;
        self.print_maze_dimensions("Success - new")?;
        Ok(())
    }

    fn do_insert_cols(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_maze_dimensions("Current")?;
        if self.get_maze().definition.row_count() == 0 {
            self.print_line("Definition is empty - insert some rows before adding columns")?;
            return Ok(());
        }
        let current_cols = self.get_maze().definition.col_count();
        let mut start_col = 1;
        if current_cols > 1 {
            start_col =
                self.prompt_number("Insert at column: ", Some(1), Some(1 + current_cols))?;
        }
        let num_cols = self.prompt_number("Number columns to insert: ", None, None)?;
        self.get_maze_mut()
            .definition
            .insert_cols(start_col - 1, num_cols)?;
        self.print_maze_dimensions("Success - new")?;
        Ok(())
    }

    fn do_delete_cols(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_maze_dimensions("Current")?;
        let current_cols = self.get_maze().definition.col_count();
        if current_cols == 0 {
            self.print_line("Definition has no columns to delete")?;
            return Ok(());
        }
        let start_col = self.prompt_number("Delete columns from: ", Some(1), Some(current_cols))?;
        let num_cols = self.prompt_number(
            "Number columns to delete: ",
            Some(1),
            Some(current_cols - start_col + 1),
        )?;
        self.get_maze_mut()
            .definition
            .delete_cols(start_col - 1, num_cols)?;
        self.print_maze_dimensions("Success - new")?;
        Ok(())
    }

    fn get_maze_dims(&self) -> (usize, usize) {
        let (num_rows, num_cols) = {
            let maze = self.get_maze();
            (maze.definition.row_count(), maze.definition.col_count())
        };
        (num_rows, num_cols)
    }

    fn maze_has_cells(&mut self) -> bool {
        let (num_rows, num_cols) = self.get_maze_dims();
        num_rows > 0 && num_cols > 0
    }

    fn process_set_endpoint(&mut self, start: bool) -> Result<(), Box<dyn Error>> {
        let name = if start { "start" } else { "finish" };
        self.print_line(&format!("Set {}", name))?;
        self.print_maze_dimensions("Current")?;
        if !self.maze_has_cells() {
            self.print_line(&format!(
                "Maze has no cells - add some rows and columns first before setting the {} cell",
                name
            ))?;
            return Ok(());
        }
        let (num_rows, num_cols) = self.get_maze_dims();
        let row = self.prompt_number("Row:", Some(1), Some(num_rows))? - 1;
        let col = self.prompt_number("Column:", Some(1), Some(num_cols))? - 1;
        if start {
            self.get_maze_mut()
                .definition
                .set_start(Some(Point { row, col }))?;
        } else {
            self.get_maze_mut()
                .definition
                .set_finish(Some(Point { row, col }))?;
        }
        Ok(())
    }

    fn do_set_start(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_set_endpoint(true)?;
        Ok(())
    }

    fn do_set_finish(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_set_endpoint(false)?;
        Ok(())
    }

    fn process_walls(&mut self, title: &str, modify_char: char) -> Result<(), Box<dyn Error>> {
        self.print_line(title)?;
        self.print_maze_dimensions("Current")?;
        if !self.maze_has_cells() {
            self.print_line(
                "Maze has no cells - add some rows and columns first before modifying walls",
            )?;
            return Ok(());
        }
        let (num_rows, num_cols) = self.get_maze_dims();
        let start_row = self.prompt_number("Start row:", Some(1), Some(num_rows))?;
        let start_col = self.prompt_number("Start column:", Some(1), Some(num_cols))?;
        let end_row = self.prompt_number("End row:", Some(1), Some(num_rows))?;
        let end_col = self.prompt_number("End column:", Some(1), Some(num_cols))?;
        self.get_maze_mut().definition.set_value(
            Point {
                row: start_row - 1,
                col: start_col - 1,
            },
            Point {
                row: end_row - 1,
                col: end_col - 1,
            },
            modify_char,
        )?;
        Ok(())
    }

    fn do_set_walls(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_walls("Set walls", 'W')?;
        Ok(())
    }

    fn do_clear_walls(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_walls("Clear walls", ' ')?;
        Ok(())
    }

    fn do_resize(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_maze_dimensions("Current")?;
        let new_row_count = self.prompt_number("Enter new row count: ", None, None)?;
        let new_col_count = self.prompt_number("Enter new column count: ", None, None)?;
        self.get_maze_mut()
            .definition
            .resize(new_row_count, new_col_count);
        self.print_maze_dimensions("Success - new")?;
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
        let path = Path { points: vec![] };
        let print_target = self.get_line_printer();
        if maze.definition.is_empty() {
            print_target.print_line("Maze is empty")?;
        } else if let Err(error) = maze.print(print_target, path) {
            print_target.print_line(&format!("Failed to print maze: {}", error))?;
        }
        Ok(())
    }

    fn do_solve(&mut self) -> Result<(), Box<dyn Error>> {
        match self.get_maze_mut().solve() {
            Ok(solution) => {
                self.print_line("\nSuccessfully solved maze:\n")?;
                let maze = self.get_maze().clone();
                let print_target = self.get_line_printer();
                if let Err(error) = maze.print(print_target, solution.path) {
                    print_target.print_line(&format!("Failed to print solved maze: {}", error))?;
                }
            }
            Err(error) => {
                self.print_line(&format!("Failed to solve maze: {}", error))?;
            }
        }
        Ok(())
    }

    fn do_print(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_maze()?;
        Ok(())
    }

    fn do_open(&mut self) -> Result<(), Box<dyn Error>> {
        let name = self.prompt_text("Enter name of maze to open: ")?;
        let file_name = &Self::get_maze_storage_id(&name);
        self.print_line(&format!("Loading from: {}", file_name))?;
        self.get_maze_mut().load_from_file(file_name)?;
        self.set_maze_name(&name)?;
        self.print_line(&format!(
            "Maze '{}' successfully loaded from file '{}'",
            name, file_name
        ))?;
        Ok(())
    }

    fn save_maze(&mut self, name: &str, prompt_overwrite: bool) -> Result<(), Box<dyn Error>> {
        if prompt_overwrite && Self::maze_name_exists(name) {
            let yes = self.prompt_yes_no(&format!(
                "A maze with the name '{}' already exists. Overwrite it?",
                name
            ))?;
            if !yes {
                self.print_line("Maze not saved")?;
                return Ok(());
            }
        }
        self.set_maze_name(name)?;
        let file_name = &Self::get_maze_storage_id(name);
        self.get_maze().save_to_file(file_name, true)?;
        self.print_line(&format!("Saved '{}' to file: '{}'", name, file_name))?;
        Ok(())
    }

    fn do_save(&mut self) -> Result<(), Box<dyn Error>> {
        let name = self.get_maze_name();
        if name.is_empty() {
            self.do_save_as()?
        } else {
            self.save_maze(&name, false)?
        }
        Ok(())
    }

    fn do_save_as(&mut self) -> Result<(), Box<dyn Error>> {
        self.print_line(&format!("Current name is '{}'", self.get_maze_name()))?;
        let name = self.prompt_text("Enter name of maze to save as: ")?;
        self.save_maze(name.trim(), true)?;
        Ok(())
    }

    fn process_keys(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            if let Some(ch) = self.read_key()? {
                let result = match ch.to_ascii_uppercase() {
                    'I' => self.do_insert_rows(),
                    'D' => self.do_delete_rows(),
                    'N' => self.do_insert_cols(),
                    'L' => self.do_delete_cols(),
                    'A' => self.do_set_start(),
                    'F' => self.do_set_finish(),
                    'W' => self.do_set_walls(),
                    'C' => self.do_clear_walls(),
                    'R' => self.do_resize(),
                    'E' => self.do_empty(),
                    'S' => self.do_solve(),
                    'P' => self.do_print(),
                    'O' => self.do_open(),
                    'V' => self.do_save(),
                    'Z' => self.do_save_as(),
                    'Q' => {
                        self.print_line("Exiting...")?;
                        return Ok(());
                    }
                    _ => {
                        self.print_line(&format!("Unknown option selected: {}", ch))?;
                        continue;
                    }
                };
                if let Err(error) = result {
                    self.print_line(&format!("Failed: {}", error))?;
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
