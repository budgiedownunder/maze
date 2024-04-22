use maze::Definition;
use maze::Maze;

static WELCOME_BANNER: &'static str = r#"
    ******************************
    * Welcome to the Maze CLI !! *
    ******************************
    "#;

fn print_welcome_banner() {
    for line in WELCOME_BANNER.lines() {
        println!("{}", line.trim_start());
    }
}

fn main() {
    print_welcome_banner();
    let _m = Maze::new(Definition::new(0, 0));
}
