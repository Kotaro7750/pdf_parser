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
    pub thumbnail: Option<(u64, u64)>,
    external_objects: Vec<(u64, u64)>,
}

pub fn parse_page_list(
    file: &mut File,
    xref: &XRef,
    root_page_ref: (u64, u64),
) -> Result<Vec<Page>, Error> {
    let root_node_obj = object::get_indirect_obj(file, xref, root_page_ref)?;

    let root_node_obj = object::ensure_indirect_obj(&root_node_obj)?;

    let root_node_map =
        object::ensure_dict_with_key(&root_node_obj, vec!["Type", "Kids", "Count"])?;

    object::ensure_dict_type(root_node_map, "Pages")?;

    let mut page_list = Vec::<Page>::new();

    let kids = root_node_map.get(&"Kids".to_string()).unwrap();
    let kids = object::ensure_array(kids)?;

    for kid in kids {
        let kid_ref = object::ensure_indirect_ref(kid)?;
        parse_page_tree_node(file, xref, kid_ref, &mut page_list)?;
    }

    Ok(page_list)
}

fn parse_page_tree_node(
    file: &mut File,
    xref: &XRef,
    node_ref: (u64, u64),
    page_list: &mut Vec<Page>,
) -> Result<(), Error> {
    let node_obj = object::get_indirect_obj(file, xref, node_ref)?;
    let node_obj = object::ensure_indirect_obj(&node_obj)?;

    // ページリストのノードには中間ノードかページノードがある
    let node_map = object::ensure_dict_with_key(&node_obj, vec!["Type"])?;

    if let Ok(_) = object::ensure_dict_type(node_map, "Page") {
        let mut xobj_vec = Vec::<(u64, u64)>::new();
        if let Some(resource_obj) = node_map.get(&"Resources".to_string()) {
            let resource_obj = object::ensure_dict_with_key(resource_obj, vec![])?;

            if let Some(xobj) = resource_obj.get(&"XObject".to_string()) {
                let xobj = object::ensure_dict_with_key(xobj, vec![])?;
                for indirect_ref in xobj
                    .iter()
                    .filter(|kv| object::ensure_indirect_ref(kv.1).is_ok())
                    .map(|kv| object::ensure_indirect_ref(kv.1).unwrap())
                {
                    xobj_vec.push(indirect_ref);
                }
            }
        }

        let may_thumnail_ref = if let Some(thumbnail_ref) = node_map.get(&"Thumb".to_string()) {
            Some(object::ensure_indirect_ref(thumbnail_ref)?)
        } else {
            None
        };

        page_list.push(Page {
            thumbnail: may_thumnail_ref,
            external_objects: xobj_vec,
        });
    } else if let Ok(_) = object::ensure_dict_type(node_map, "Pages") {
        let node_map = object::ensure_dict_with_key(&node_obj, vec!["Kids", "Count"])?;

        let kids = node_map.get(&"Kids".to_string()).unwrap();
        let kids = object::ensure_array(kids)?;

        for kid in kids {
            let kid_ref = object::ensure_indirect_ref(kid)?;
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
            let obj = object::get_indirect_obj(file, &xref, *xobj_ref)?;
            let (dict_obj, byte_vec) = object::get_stream(file, &xref, &obj)?;

            let map = object::ensure_dict_with_key(dict_obj, vec!["Subtype"])?;

            let subtype = object::ensure_name(map.get(&"Subtype".to_string()).unwrap())?;

            if subtype != "Image" {
                panic!("subtype is not image");
            }

            let image_param = image_localmod::ImageDecodeParam::new(dict_obj, file, xref).unwrap();

            images.push(image_localmod::decode_image(&image_param, &byte_vec).unwrap());
        }

        Ok(images)
    }
}
