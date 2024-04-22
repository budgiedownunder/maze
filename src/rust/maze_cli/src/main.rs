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


#[cfg(test)]
mod tests {
    #[test]
    fn dummy_test() {
        assert_eq!(1, 1);
    }
}    