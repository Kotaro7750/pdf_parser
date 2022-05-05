use std::fs::File;

use crate::cross_reference::XRef;
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
}

pub fn parse_page_list(
    file: &mut File,
    xref: &mut XRef,
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
    xref: &mut XRef,
    node_ref: (u64, u64),
    page_list: &mut Vec<Page>,
) -> Result<(), Error> {
    let node_obj = object::get_indirect_obj(file, xref, node_ref)?;
    let node_obj = object::ensure_indirect_obj(&node_obj)?;

    // ページリストのノードには中間ノードかページノードがある
    let node_map = object::ensure_dict_with_key(&node_obj, vec!["Type"])?;

    if let Ok(_) = object::ensure_dict_type(node_map, "Page") {
        println!("page detected");

        let may_thumnail_ref = if let Some(thumbnail_ref) = node_map.get(&"Thumb".to_string()) {
            Some(object::ensure_indirect_ref(thumbnail_ref)?)
        } else {
            None
        };

        page_list.push(Page {
            thumbnail: may_thumnail_ref,
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
