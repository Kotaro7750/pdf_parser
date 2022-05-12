use flate2::read::ZlibDecoder;
use image::{DynamicImage, ImageBuffer, RgbImage};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;

use crate::cross_reference;
use crate::object;
use crate::parser::Object;

#[derive(Debug)]
pub enum Error {
    Object(object::Error),
    UnsupporttedColorSpace,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            // TODO object::ErrorにDisplayトレイトを実装したら適切なエラー文にする
            Error::Object(e) => write!(f, "{:?}", e),
            Error::UnsupporttedColorSpace => write!(f, "colorspace is not supportted"),
        }
    }
}

impl From<object::Error> for Error {
    fn from(e: object::Error) -> Self {
        Self::Object(e)
    }
}

enum ColorSpace {
    DeviceGray,
    DeviceRGB,
}

pub struct ImageDecodeParam {
    width: u32,
    height: u32,
    colorspace: ColorSpace,
}

impl ImageDecodeParam {
    pub fn new(
        dict_obj: &Object,
        file: &mut File,
        xref: &cross_reference::XRef,
    ) -> Result<ImageDecodeParam, Error> {
        let image_dict = object::ensure_dict_with_key(dict_obj, vec!["Subtype"])?;

        let subtype = object::ensure_name(image_dict.get(&"Subtype".to_string()).unwrap())?;

        if subtype != "Image" {
            panic!("subtype is not image");
        }

        let image_dict = object::ensure_dict_with_key(dict_obj, vec!["Width", "Height"])?;

        let width = object::ensure_integer(&image_dict.get(&"Width".to_string()).unwrap())?;
        let height = object::ensure_integer(&image_dict.get(&"Height".to_string()).unwrap())?;

        let width = width as u32;
        let height = height as u32;

        let colorspace = get_colorspace(image_dict, file, xref)?;

        Ok(ImageDecodeParam {
            width,
            height,
            colorspace,
        })
    }
}

pub fn decode_image(image: &ImageDecodeParam, byte_vec: &Vec<u8>) -> Result<RgbImage, Error> {
    let deflater = ZlibDecoder::new(&byte_vec[..]);

    let decoded: Result<Vec<u8>, _> = deflater.bytes().collect();
    let decoded = decoded.unwrap();

    let width = image.width;
    let height = image.height;

    let image_result = match image.colorspace {
        ColorSpace::DeviceRGB => DynamicImage::ImageRgb8(
            ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(width, height, decoded).unwrap(),
        ),
        ColorSpace::DeviceGray => DynamicImage::ImageLuma8(
            ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(width, height, decoded).unwrap(),
        ),
    };

    let image_result = image_result.into_rgb8();

    Ok(image_result)
}

fn get_colorspace(
    image_dict: &HashMap<String, Object>,
    file: &mut File,
    xref: &cross_reference::XRef,
) -> Result<ColorSpace, Error> {
    let colorspace = match image_dict.get(&"ColorSpace".to_string()).unwrap() {
        Object::Name(name) => name.clone(),
        Object::IndirectRef(obj_num, gen_num) => {
            match object::ensure_indirect_obj(&object::get_indirect_obj(
                file,
                xref,
                (*obj_num, *gen_num),
            )?)? {
                Object::Name(name) => name.clone(),
                _ => {
                    return Err(Error::UnsupporttedColorSpace);
                }
            }
        }
        _ => return Err(Error::UnsupporttedColorSpace),
    };

    Ok(match colorspace.as_str() {
        "DeviceRGB" => ColorSpace::DeviceRGB,
        "DeviceGray" => ColorSpace::DeviceGray,
        _ => return Err(Error::UnsupporttedColorSpace),
    })
}
