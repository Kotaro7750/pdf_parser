use image as image_lib;
use std::fs::File;

use crate::cross_reference::XRef;
use crate::image as image_localmod;
use crate::object;

#[derive(Debug)]
pub enum Error {
    Object(object::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Object(e) => write!(f, "object: {}", e),
        }
    }
}

impl From<object::Error> for Error {
    fn from(e: object::Error) -> Error {
        Self::Object(e)
    }
}

#[derive(Debug)]
pub struct Page {
    page_number: usize,
    thumbnail: Option<object::PdfIndirectRef>,
    external_objects: Vec<object::PdfIndirectRef>,
}

impl Page {
    pub fn new(
        page_number: usize,
        thumbnail_ref: Option<object::PdfIndirectRef>,
        external_objects: Vec<object::PdfIndirectRef>,
    ) -> Self {
        Self {
            page_number,
            thumbnail: thumbnail_ref,
            external_objects,
        }
    }

    pub fn get_page_number(&self) -> usize {
        self.page_number
    }

    pub fn extract_images(
        &self,
        file: &mut File,
        xref: &XRef,
    ) -> Result<Vec<image_lib::RgbImage>, Error> {
        let mut images: Vec<image_lib::RgbImage> = vec![];

        for xobj_ref in &(self.external_objects) {
            let image = construct_image_from_xobj(xobj_ref, file, xref)?;
            images.push(image);
        }

        Ok(images)
    }
}

fn construct_image_from_xobj(
    xobj_ref: &object::PdfIndirectRef,
    file: &mut File,
    xref: &XRef,
) -> Result<image_lib::RgbImage, Error> {
    let xobj = xobj_ref.get_indirect_obj(file, &xref)?;
    let xobj = object::PdfStreamObj::ensure_stream(&xobj)?;

    let xobj_dict = &xobj.dict;
    xobj_dict.assert_with_key(vec!["Subtype"])?;

    let subtype = object::PdfName::ensure(xobj_dict.get("Subtype").unwrap())?;
    if subtype != "Image" {
        panic!("subtype is not image");
    }

    let stream_content = xobj.get_stream(file, xref)?;

    let image_param = image_localmod::ImageDecodeParam::new(&xobj_dict, file, xref).unwrap();
    let image = image_localmod::decode_image(&image_param, &stream_content).unwrap();

    Ok(image)
}
