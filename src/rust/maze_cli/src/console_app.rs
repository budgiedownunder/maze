use maze_cli::app::App;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use std::io::{self};
use std::thread;
use std::time::Duration;

pub struct ConsoleApp {}

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
