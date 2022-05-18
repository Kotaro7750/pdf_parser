use flate2::read::ZlibDecoder;
use image::{DynamicImage, ImageBuffer, RgbImage};
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
        image_dict: &object::PdfDict,
        file: &mut File,
        xref: &cross_reference::XRef,
    ) -> Result<ImageDecodeParam, Error> {
        image_dict.assert_with_key(vec!["Subtype"])?;

        let subtype = object::PdfName::ensure(image_dict.get("Subtype").unwrap())?;

        if subtype != "Image" {
            panic!("subtype is not image");
        }

        image_dict.assert_with_key(vec!["Width", "Height"])?;

        let width = object::PdfInteger::ensure(&image_dict.get("Width").unwrap())?;
        let height = object::PdfInteger::ensure(&image_dict.get("Height").unwrap())?;

        let width = *width.as_ref() as u32;
        let height = *height.as_ref() as u32;

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
    image_dict: &object::PdfDict,
    file: &mut File,
    xref: &cross_reference::XRef,
) -> Result<ColorSpace, Error> {
    let colorspace = match image_dict.get("ColorSpace").unwrap() {
        Object::Name(name) => name.clone(),
        Object::IndirectRef(indirect_ref) => {
            let hoge = indirect_ref.get_indirect_obj(file, xref)?;

            match object::PdfIndirectObj::ensure(&hoge)?.get_object() {
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
