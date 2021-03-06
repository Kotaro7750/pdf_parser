use ::image as image_lib;
use std::fs::File;

mod cross_reference;
mod error;
mod header;
mod image;
mod lexer;
mod object;
mod page;
mod page_tree;
mod parser;
mod raw_byte;
mod trailer;
mod util;

pub struct PDF<'a> {
    file: &'a mut File,
    size: u64,
    trailer: trailer::Trailer,
    xref: cross_reference::XRef,
    pages: page_tree::Pages,
}

impl<'a> PDF<'a> {
    pub fn new(file: &'a mut File) -> Result<PDF<'a>, error::Error> {
        let size = PDF::get_file_size(file)?;

        header::validate_pdf_header(file)?;

        let trailer = trailer::parse_trailer(file, size)?;
        let xref = cross_reference::XRef::new(file, trailer.xref_start_offset)?;

        // ドキュメントカタログ
        let root_ref = trailer.get_root_catalog_ref();
        let root_obj = root_ref.get_indirect_obj(file, &xref)?;
        let root_obj = object::PdfIndirectObj::ensure(&root_obj)?.get_object();

        let root_dict = object::PdfDict::ensure_with_key(root_obj, vec!["Type", "Pages"])?;

        root_dict.ensure_type("Catalog")?;

        let pages_ref = object::PdfIndirectRef::ensure(root_dict.get("Pages").unwrap())?;

        let pages = page_tree::Pages::new(file, &xref, pages_ref)?;

        Ok(PDF {
            file,
            size,
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
            let page = self.pages.get_page(*page_number)?;

            images_of_pages.push(page.extract_images(self.file, &self.xref).unwrap());
        }

        Ok(images_of_pages)
    }
}
