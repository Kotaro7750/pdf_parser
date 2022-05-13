use ::image as image_lib;
use std::fs::File;

mod cross_reference;
mod error;
mod header;
mod image;
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
    pages: Vec<page::Page>,
}

impl<'a> PDF<'a> {
    pub fn new(file: &'a mut File) -> Result<PDF<'a>, error::Error> {
        let size = PDF::get_file_size(file)?;

        header::validate_pdf_header(file)?;

        let trailer = trailer::parse_trailer(file, size)?;
        let mut xref = cross_reference::XRef::new(file, &trailer);

        // ドキュメントカタログ
        let root_ref = trailer.get_root_catalog_ref();
        let root_obj = object::get_indirect_obj(file, &mut xref, root_ref)?;
        let root_obj = object::ensure_indirect_obj(&root_obj)?;

        let root_hm = object::ensure_dict_with_key(root_obj, vec!["Type", "Pages"])?;

        object::ensure_dict_type(root_hm, "Catalog")?;

        let pages_ref = object::ensure_indirect_ref(root_hm.get(&String::from("Pages")).unwrap())?;

        let pages = page::parse_page_list(file, &mut xref, pages_ref)?;

        Ok(PDF {
            file: file,
            size: size,
            trailer,
            xref,
            pages,
        })
    }

    fn get_file_size(file: &File) -> Result<u64, std::io::Error> {
        Ok(file.metadata()?.len())
    }

    pub fn extract_image(
        &mut self,
        request_pages: &Vec<usize>,
    ) -> Result<Vec<Vec<image_lib::RgbImage>>, error::Error> {
        let mut images_of_pages: Vec<Vec<image_lib::RgbImage>> = vec![];
        for page_number in request_pages {
            let page = &self.pages[*page_number - 1];

            images_of_pages.push(page.extract_image(&mut self.file, &self.xref).unwrap());
        }

        Ok(images_of_pages)
    }
}
