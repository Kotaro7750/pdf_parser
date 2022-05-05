extern crate pdf_parser;

use std::env;
use std::fs::File;
use std::process;

use pdf_parser::PDF;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Specify file name");
        process::exit(1);
    }

    let filename = &args[1];

    let mut file = File::open(filename).unwrap_or_else(|err| {
        println!("File cannot open: {}", err);
        process::exit(1);
    });

    let pdf = pdf_parser::PDF::new(&mut file).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1)
    });
}
