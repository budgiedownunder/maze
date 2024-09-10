use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use std::io::{self};
use std::thread;
use std::time::Duration;

use maze::Definition;
use maze::LinePrinter;
use maze::Maze;
use maze_console::app::App;

use storage::Store;

pub struct ConsoleApp {
    store: Box<dyn Store>,
    current_maze: Maze,
}

impl ConsoleApp {
    pub fn new(store: Box<dyn Store>) -> ConsoleApp {
        ConsoleApp {
            store,
            current_maze: Maze::new(Definition::new(0, 0)),
        }
    }
}

impl App for ConsoleApp {

    fn get_store(&mut self) -> &mut Box<dyn Store> {
        &mut self.store
    }

    fn get_maze(&self) -> &Maze {
        &self.current_maze
    }

    fn get_maze_mut(&mut self) -> &mut Maze {
        &mut self.current_maze
    }

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

    fn get_line_printer(&mut self) -> &mut dyn LinePrinter {
        self
    }
}

impl LinePrinter for ConsoleApp {
    fn print_line(&mut self, line: &str) -> Result<(), io::Error> {
        println!("{}", line);
        Ok(())
    }
}
