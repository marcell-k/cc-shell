#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();
    println!("{}: command not found", input.trim());
    print!("$ ");
    io::stdout().flush().unwrap();
}
