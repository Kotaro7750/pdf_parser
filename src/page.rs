use image as image_lib;
use std::fs::File;

use crate::cross_reference::XRef;
use crate::image as image_localmod;
use crate::object;

#[derive(Debug)]
pub enum Error {
    Object(object::Error),
}

impl From<object::Error> for Error {
    fn from(e: object::Error) -> Error {
        Self::Object(e)
    }
}

pub struct Page {
    pub thumbnail: Option<object::PdfIndirectRef>,
    external_objects: Vec<object::PdfIndirectRef>,
}

pub fn parse_page_list(
    file: &mut File,
    xref: &XRef,
    root_page_ref: &object::PdfIndirectRef,
) -> Result<Vec<Page>, Error> {
    let root_node_obj = root_page_ref.get_indirect_obj(file, xref)?;
    let root_node_obj = object::PdfIndirectObj::ensure(&root_node_obj)?;
    let root_node_obj = root_node_obj.get_object();

    let root_node_dict =
        object::PdfDict::ensure_with_key(&root_node_obj, vec!["Type", "Kids", "Count"])?;

    root_node_dict.ensure_type("Pages")?;

    let mut page_list = Vec::<Page>::new();

    let kids = root_node_dict.get("Kids").unwrap();
    let kids = object::PdfArray::ensure(kids)?;

    for kid in kids {
        let kid_ref = object::PdfIndirectRef::ensure(&kid)?;
        parse_page_tree_node(file, xref, kid_ref, &mut page_list)?;
    }

    Ok(page_list)
}

fn parse_page_tree_node(
    file: &mut File,
    xref: &XRef,
    node_ref: &object::PdfIndirectRef,
    page_list: &mut Vec<Page>,
) -> Result<(), Error> {
    let node_obj = node_ref.get_indirect_obj(file, xref)?;
    let node_obj = object::PdfIndirectObj::ensure(&node_obj)?.get_object();

    // ページリストのノードには中間ノードかページノードがある
    let node_dict = object::PdfDict::ensure_with_key(&node_obj, vec!["Type"])?;

    if let Ok(_) = node_dict.ensure_type("Page") {
        let mut xobj_vec = Vec::<object::PdfIndirectRef>::new();

        if let Some(resource_obj) = node_dict.get("Resources") {
            let resource_dict = object::PdfDict::ensure_with_key(resource_obj, vec![])?;

            if let Some(xobj) = resource_dict.get("XObject") {
                let xobj = object::PdfDict::ensure_with_key(xobj, vec![])?;
                for indirect_ref in xobj
                    .iter()
                    .filter(|kv| object::PdfIndirectRef::ensure(kv.1).is_ok())
                    .map(|kv| object::PdfIndirectRef::ensure(kv.1).unwrap())
                {
                    xobj_vec.push(indirect_ref.clone());
                }
            }
        }

        let may_thumnail_ref = if let Some(thumbnail_ref) = node_dict.get("Thumb") {
            Some(object::PdfIndirectRef::ensure(thumbnail_ref)?.clone())
        } else {
            None
        };

        page_list.push(Page {
            thumbnail: may_thumnail_ref,
            external_objects: xobj_vec,
        });
    } else if let Ok(_) = object::PdfDict::ensure_type(node_dict, "Pages") {
        let node_map = object::PdfDict::ensure_with_key(&node_obj, vec!["Kids", "Count"])?;

        let kids = node_map.get("Kids").unwrap();
        let kids = object::PdfArray::ensure(kids)?;

        for kid in kids {
            let kid_ref = object::PdfIndirectRef::ensure(&kid)?;
            parse_page_tree_node(file, xref, kid_ref, page_list)?;
        }
    } else {
        panic!("page nor pages");
    }

    Ok(())
}

impl Page {
    pub fn extract_image(
        &self,
        file: &mut File,
        xref: &XRef,
    ) -> Result<Vec<image_lib::RgbImage>, Error> {
        let mut images: Vec<image_lib::RgbImage> = vec![];

        for xobj_ref in &(self.external_objects) {
            let obj = xobj_ref.get_indirect_obj(file, &xref)?;
            let stream_obj = object::PdfStreamObj::ensure_stream(&obj)?;
            let byte_vec = stream_obj.get_stream(file, xref)?;

            let dict = &stream_obj.dict;
            dict.assert_with_key(vec!["Subtype"])?;

            let subtype = object::PdfName::ensure(dict.get("Subtype").unwrap())?;

            if subtype != "Image" {
                panic!("subtype is not image");
            }

            let image_param = image_localmod::ImageDecodeParam::new(&dict, file, xref).unwrap();

            images.push(image_localmod::decode_image(&image_param, &byte_vec).unwrap());
        }

        Ok(images)
    }
}
