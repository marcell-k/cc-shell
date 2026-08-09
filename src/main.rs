#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();
    print!("$ ");
    println!("{}: command not found", input.trim());
    io::stdout().flush().unwrap();
}
