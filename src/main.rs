extern crate pdf_parser;

use image::codecs::jpeg::JpegEncoder;
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

    let mut pdf = PDF::new(&mut file).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1)
    });

    for (page_number, images) in pdf
        .extract_image(&(1..=5).collect())
        .unwrap()
        .iter()
        .enumerate()
    {
        for (image_number, image) in images.iter().enumerate() {
            let mut file = File::create(format!("{}-{}.jpg", page_number, image_number)).unwrap();

            let mut encoder = JpegEncoder::new(file);

            encoder.encode_image(image).unwrap();

            //image
            //    .save(format!("{}-{}.png", page_number, image_number))
            //    .unwrap();
        }
    }
}
