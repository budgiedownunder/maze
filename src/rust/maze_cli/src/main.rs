use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use std::io::{self};
use std::thread;
use std::time::Duration;

use maze::Definition;
use maze::Maze;

fn getch() -> io::Result<Option<char>> {
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

fn str_lines(s: &'static str) -> Vec<&'static str> {
    s.lines().map(|line| line.trim_start()).collect()
}

fn print_lines(lines: Vec<&'static str>) {
    for line in lines {
        println!("{}", line);
    }
}

static WELCOME_BANNER: &'static str = r#"******************************
       * Welcome to the Maze CLI !! *
       ******************************
    "#;

fn welcome_banner_lines() -> Vec<&'static str> {
    str_lines(WELCOME_BANNER)
}

fn print_welcome_banner() {
    print_lines(welcome_banner_lines());
}

static MENU: &'static str = r#"******************************
        Select action:

        P -> Print maze
        Q -> Quit
        ******************************
        "#;

fn menu_lines() -> Vec<&'static str> {
    str_lines(MENU)
}

fn print_menu() {
    print_lines(menu_lines());
}

fn process_keys() {
    let mut _m: Maze = Maze::new(Definition::new(0, 0));
    loop {
        match getch() {
            Ok(Some(ch)) => match ch.to_ascii_uppercase() {
                'Q' => {
                    println!("Exiting...");
                    break;
                }
                'P' => {
                    println!("Would print");
                }
                _ => println!("Unknown option selected: {}", ch),
            },
            Ok(None) => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err)
            }
        }
    }
}

fn run() {
    print_welcome_banner();
    print_menu();
    process_keys();
}

fn main() {
    run();
}

#[cfg(test)]
mod tests {
    // use super::*;
    use env_logger::Env;
    use expectrl::{spawn, Eof, Regex /*, Session */};
    use log::LevelFilter;
    use std::str;
    use std::sync::Once;
    use std::time::Duration;
    use strip_ansi_escapes::strip;

    static INIT: Once = Once::new();

    fn init_logging() {
        INIT.call_once(|| {
            // Set up logging level from environment or fallback to debug
            env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
                .filter_level(LevelFilter::Debug)
                .init();
            log::debug!("Debug logging is now enabled.");
        });
    }

    #[test]
    fn test_logging_output() {
        init_logging();
    }
    #[test]
    fn should_be_able_quit_on_start() -> Result<(), expectrl::Error> {
        // Initialize the logger for debugging
        init_logging();

        // Spawn the CLI application
        println!("Spawning CLI...");
        let mut p = spawn("cargo run --quiet").expect("Failed to spawn process");
        println!("Spawning complete.");

        // Set a timeout for expect calls
        p.set_expect_timeout(Some(Duration::from_secs(5)));

        // Capture and print any initial output
        let output = p.expect(Regex(".+"))?;
        // Convert the output to a readable string
        println!("Captured Output ==> {:?}", output);
        let stripped_output = strip(output.before()).unwrap();
        if let Ok(output_str) = str::from_utf8(&stripped_output) {
            println!("Captured Output: {}", output_str);
        } else {
            println!("Captured Output contains invalid UTF-8");
        }

        // Send 'Q' key press without newline
        p.send("Q")?;

        // Ensure the process exits
        p.expect(Eof)?;

        Ok(())
    }

    // Helper functions
    //fn app_name() -> &'static str {
    //    env!("CARGO_PKG_NAME")
    //}
}
