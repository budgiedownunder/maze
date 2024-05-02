use crossterm::event::{poll, read, Event, KeyCode, KeyEvent/* , KeyEventKind*/};
use std::io::{self};
use std::time::Duration;
use std::thread;

use maze::Definition;
use maze::Maze;

trait InputSource {
    fn getch(&mut self) -> io::Result<char>;
} 

// Implementation for reading from crossterm
struct Crossterm;
impl InputSource for Crossterm {
    fn getch(&mut self) -> io::Result<char> {
        loop {
            if poll(Duration::from_secs(0))? {
                if let Event::Key(KeyEvent { code, modifiers, kind, .. }) = read()? {
                    if modifiers.is_empty() && kind == crossterm::event::KeyEventKind::Press {
                        if let KeyCode::Char(ch) = code {
                            return Ok(ch);
                        }
                    }    
                } 
            }
            thread::sleep(Duration::from_millis(10));                
        }
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

fn process_keys<T: InputSource>(input: &mut T) {
    let mut _m: Maze = Maze::new(Definition::new(0, 0));
    loop {
        match input.getch() {
            Ok(ch) => {
                match ch.to_ascii_uppercase() {
                    'Q' => {
                        println!("Exiting...");
                        break;
                    }
                    'P' => {
                        println!("Would print");
                    }
                    _ => println!("Unknown option selected: {}", ch)
                } 
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
    process_keys(&mut Crossterm {});
}

fn main() {
    run();
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use assert_cmd::Command;

    #[test]
    fn should_be_able_quit_on_start() {
        let output = Command::cargo_bin(app_name()).unwrap().output().unwrap();
        let mut expected_lines = welcome_banner_lines();
        expected_lines.extend(menu_lines());

        let std_output = String::from_utf8_lossy(&output.stdout);
        assert_lines_eq(std_output.lines().collect(), expected_lines);

        //feed_character('Q');
    }

    // Helper functions
    fn app_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn assert_lines_eq(actual_lines: Vec<&str>, expected_lines: Vec<&str>) {
        assert_eq!(actual_lines.len(), expected_lines.len());
        for (actual_line, expected_line) in actual_lines.iter().zip(expected_lines.iter()) {
            assert_eq!(*actual_line, *expected_line);
        }
    }

    fn feed_character(ch: char) -> io::Result<()> {
        let mut stdin = io::stdout();
        let mut handle = stdin.lock();
        handle.write_all(ch.to_string().as_bytes())?;
        Ok(())
    }
}
*/