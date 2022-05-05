use bmp;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use flate2::read::ZlibDecoder;

mod cross_reference;
mod error;
mod header;
mod lexer;
mod object;
mod page;
mod parser;
mod raw_byte;
mod trailer;

pub struct PDF<'a> {
    file: &'a mut File,
    size: u64,
    trailer: trailer::Trailer,
    xref: cross_reference::XRef,
}

impl<'a> PDF<'a> {
    pub fn new(file: &'a mut File) -> Result<PDF<'a>, error::Error> {
        let size = PDF::get_file_size(file)?;

        header::expect_pdf(file)?;

        let trailer = trailer::parse_trailer(file, size)?;
        let mut xref = cross_reference::XRef::new(file, &trailer);

        // ドキュメントカタログ
        let root_ref = trailer.get_root_catalog_ref();
        let root_obj = object::get_indirect_obj(file, &mut xref, root_ref)?;
        let root_obj = object::ensure_indirect_obj(&root_obj)?;

        let root_hm = object::ensure_dict_with_key(root_obj, vec!["Type", "Pages"])?;

        object::ensure_dict_type(root_hm, "Catalog")?;

        let page_list_ref =
            object::ensure_indirect_ref(root_hm.get(&String::from("Pages")).unwrap())?;

        let page_list = page::parse_page_list(file, &mut xref, page_list_ref)?;

        for page in page_list {
            if let Some(thumbnail_ref) = page.thumbnail {
                let obj = object::get_indirect_obj(file, &mut xref, thumbnail_ref)?;
                let (map, byte_vec) = object::get_stream(file, &mut xref, &obj)?;

                println!("{:?}", map);

                let width = object::ensure_integer(&map.get(&"Width".to_string()).unwrap())?;
                let height = object::ensure_integer(&map.get(&"Height".to_string()).unwrap())?;

                let mut deflater = ZlibDecoder::new(&byte_vec[..]);

                let decoded: Result<Vec<u8>, _> = deflater.bytes().collect();
                let decoded = decoded.unwrap();

                let mut img = bmp::Image::new(width as u32, height as u32);

                for x in 0..=(width - 1) {
                    for y in 0..=(height - 1) {
                        let offset = 3 * (width * y + x);
                        img.set_pixel(
                            x as u32,
                            y as u32,
                            bmp::Pixel::new(
                                decoded[offset as usize],
                                decoded[(offset + 1) as usize],
                                decoded[(offset + 2) as usize],
                            ),
                        );
                    }
                }

                img.save(format!("./{}", thumbnail_ref.0));
            }
        }

        Ok(PDF {
            file: file,
            size: size,
            trailer,
            xref,
        })
    }

    fn get_file_size(file: &File) -> Result<u64, std::io::Error> {
        Ok(file.metadata()?.len())
    }
}
