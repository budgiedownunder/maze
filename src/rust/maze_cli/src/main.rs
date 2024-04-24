use maze::Definition;
use maze::Maze;

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

fn main() {
    print_welcome_banner();
    print_menu();
    let _m = Maze::new(Definition::new(0, 0));
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;

    // use predicates::str::contains;

    #[test]
    fn should_display_welcome_and_menu_on_start() {
        let output = Command::cargo_bin(app_name()).unwrap().output().unwrap();
        let mut expected_lines = welcome_banner_lines();
        expected_lines.extend(menu_lines());

        let std_output = String::from_utf8_lossy(&output.stdout);
        assert_lines_eq(std_output.lines().collect(), expected_lines);
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
}
