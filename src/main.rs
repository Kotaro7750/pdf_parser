extern crate pdf_parser;

use std::fs::File;
use std::process;

fn main() {
    let mut file = File::open("sample.pdf").unwrap_or_else(|err| {
        println!("File cannot open: {}", err);
        process::exit(1);
    });

    let pdf = pdf_parser::PDF::new(&mut file).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1)
    });
}
