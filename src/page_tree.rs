use std::fs::File;

use crate::cross_reference::XRef;
use crate::object;
use crate::page::Page;

#[derive(Debug)]
pub enum Error {
    PageNotFound(usize),
    Object(object::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::PageNotFound(page_number) => write!(f, "page `{}` is not found", page_number),
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
pub struct Pages {
    pages: Vec<Page>,
}

impl Pages {
    pub fn new(
        file: &mut File,
        xref: &XRef,
        root_page_ref: &object::PdfIndirectRef,
    ) -> Result<Self, Error> {
        let parsed_page_list = Self::parse_page_list(file, xref, root_page_ref)?;

        for (i, page) in parsed_page_list.iter().enumerate() {
            assert_eq!(i + 1, page.get_page_number());
        }

        Ok(Self {
            pages: parsed_page_list,
        })
    }

    pub fn get_page(&self, page_number: usize) -> Result<&Page, Error> {
        if page_number == 0 || self.get_page_number() < page_number {
            return Err(Error::PageNotFound(page_number));
        }

        let page = &self.pages[page_number - 1];

        assert_eq!(page_number, page.get_page_number());

        Ok(page)
    }

    fn parse_page_list(
        file: &mut File,
        xref: &XRef,
        root_page_ref: &object::PdfIndirectRef,
    ) -> Result<Vec<Page>, Error> {
        let root_node_obj = root_page_ref.get_indirect_obj(file, xref)?;
        let root_node_obj = object::PdfIndirectObj::ensure(&root_node_obj)?;
        let root_node_obj = root_node_obj.get_object();

        let root_node_dict =
            object::PdfDict::ensure_with_key(root_node_obj, vec!["Type", "Kids", "Count"])?;
        root_node_dict.ensure_type("Pages")?;

        let kids = root_node_dict.get("Kids").unwrap();
        let kids = object::PdfArray::ensure(kids)?;

        let mut page_list = Vec::<Page>::new();
        let mut detected_pages = 0;
        for kid in kids {
            let kid_ref = object::PdfIndirectRef::ensure(kid)?;

            let mut page_list_in_kid =
                Self::parse_page_tree_node(file, xref, kid_ref, detected_pages + 1)?;
            detected_pages += page_list_in_kid.len();

            page_list.append(&mut page_list_in_kid);
        }

        Ok(page_list)
    }

    fn parse_page_tree_node(
        file: &mut File,
        xref: &XRef,
        node_ref: &object::PdfIndirectRef,
        start_page_number: usize,
    ) -> Result<Vec<Page>, Error> {
        let node_obj = node_ref.get_indirect_obj(file, xref)?;
        let node_obj = object::PdfIndirectObj::ensure(&node_obj)?.get_object();

        // ページリストのノードには中間ノードかページノードがある
        let node_dict = object::PdfDict::ensure_with_key(node_obj, vec!["Type"])?;

        let mut page_list = Vec::<Page>::new();
        if node_dict.ensure_type("Page").is_ok() {
            let page = Self::parse_page_node(node_dict, start_page_number)?;

            page_list.push(page);
        } else if object::PdfDict::ensure_type(node_dict, "Pages").is_ok() {
            let node_dict = object::PdfDict::ensure_with_key(node_obj, vec!["Kids", "Count"])?;

            let kids = node_dict.get("Kids").unwrap();
            let kids = object::PdfArray::ensure(kids)?;

            let mut detected_pages = 0;
            for kid in kids {
                let kid_ref = object::PdfIndirectRef::ensure(kid)?;

                let mut page_list_in_kid = Self::parse_page_tree_node(
                    file,
                    xref,
                    kid_ref,
                    start_page_number + detected_pages,
                )?;
                detected_pages += page_list_in_kid.len();

                page_list.append(&mut page_list_in_kid)
            }
        } else {
            panic!("page nor pages");
        }

        Ok(page_list)
    }

    fn parse_page_node(node_dict: &object::PdfDict, page_number: usize) -> Result<Page, Error> {
        let external_objects = Self::extract_external_objects(node_dict)?;
        let may_thumbnail_ref = Self::extract_thumbnail_ref(node_dict)?;

        Ok(Page::new(page_number, may_thumbnail_ref, external_objects))
    }

    fn extract_external_objects(
        node_dict: &object::PdfDict,
    ) -> Result<Vec<object::PdfIndirectRef>, Error> {
        let mut external_objects = Vec::<object::PdfIndirectRef>::new();

        // XObjectはResource Dictionaryの更に配下にある
        if let Some(resource_obj) = node_dict.get("Resources") {
            let resource_dict = object::PdfDict::ensure_with_key(resource_obj, vec![])?;

            if let Some(xobj_obj) = resource_dict.get("XObject") {
                let xobj_dict = object::PdfDict::ensure_with_key(xobj_obj, vec![])?;

                xobj_dict
                    .iter()
                    .filter_map(|kv| object::PdfIndirectRef::ensure(kv.1).ok())
                    .for_each(|indirect_ref| external_objects.push(indirect_ref.clone()))
            }
        }

        Ok(external_objects)
    }

    fn extract_thumbnail_ref(
        node_dict: &object::PdfDict,
    ) -> Result<Option<object::PdfIndirectRef>, Error> {
        let may_thumbnail_ref = match node_dict.get("Thumb") {
            Some(thumbnail_ref) => Some(object::PdfIndirectRef::ensure(thumbnail_ref)?.clone()),
            _ => None,
        };

        Ok(may_thumbnail_ref)
    }

    fn get_page_number(&self) -> usize {
        self.pages.len()
    }
}
