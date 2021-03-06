use flate2::read::ZlibDecoder;
use image::{DynamicImage, ImageBuffer, RgbImage};
use jpeg_decoder;
use std::fmt;
use std::fs::File;
use std::io::Read;

use crate::cross_reference;
use crate::object;

#[derive(Debug)]
pub enum Error {
    Object(object::Error),
    UnsupporttedColorSpace,
    UnsupporttedFilter,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::Object(e) => write!(f, "object: {}", e),
            Error::UnsupporttedColorSpace => write!(f, "colorspace is not supportted"),
            Error::UnsupporttedFilter => write!(f, "filter is not supportted"),
        }
    }
}
impl From<object::Error> for Error {
    fn from(e: object::Error) -> Self {
        Self::Object(e)
    }
}

enum Filter {
    Flate,
    DCT,
}

enum ColorSpace {
    DeviceGray,
    DeviceRGB,
}

pub struct ImageDecodeParam {
    width: u32,
    height: u32,
    colorspace: ColorSpace,
    filter: Filter,
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

        image_dict.assert_with_key(vec!["Width", "Height", "Filter"])?;

        let width = object::PdfInteger::ensure(image_dict.get("Width").unwrap())?;
        let height = object::PdfInteger::ensure(image_dict.get("Height").unwrap())?;

        width.assert_natural()?;
        let width = width.unpack() as u32;

        height.assert_natural()?;
        let height = height.unpack() as u32;

        let colorspace = get_colorspace(image_dict, file, xref)?;
        let filter = get_filter(image_dict)?;

        Ok(ImageDecodeParam {
            width,
            height,
            colorspace,
            filter,
        })
    }
}

fn get_colorspace(
    image_dict: &object::PdfDict,
    file: &mut File,
    xref: &cross_reference::XRef,
) -> Result<ColorSpace, Error> {
    image_dict.assert_with_key(vec!["ColorSpace"])?;
    let colorspace_obj = image_dict.get("ColorSpace").unwrap();

    let colorspace = if let Ok(name) = object::PdfName::ensure(colorspace_obj) {
        name.clone()
    } else {
        let indirect_ref = object::PdfIndirectRef::ensure(colorspace_obj)?;
        let indirect_obj = indirect_ref.get_indirect_obj(file, xref)?;

        let may_name = object::PdfIndirectObj::ensure(&indirect_obj)?.get_object();
        let name = object::PdfName::ensure(may_name)?;

        name.clone()
    };

    Ok(match colorspace.as_str() {
        "DeviceRGB" => ColorSpace::DeviceRGB,
        "DeviceGray" => ColorSpace::DeviceGray,
        _ => return Err(Error::UnsupporttedColorSpace),
    })
}

fn get_filter(image_dict: &object::PdfDict) -> Result<Filter, Error> {
    image_dict.assert_with_key(vec!["Filter"])?;
    let filter_obj = image_dict.get("Filter").unwrap();

    let filter = if let Ok(array_obj) = object::PdfArray::ensure(filter_obj) {
        match array_obj.get(0) {
            Some(name) => object::PdfName::ensure(name)?,
            None => return Err(Error::UnsupporttedFilter),
        }
    } else {
        object::PdfName::ensure(filter_obj)?
    };

    Ok(match filter.as_str() {
        "FlateDecode" => Filter::Flate,
        "DCTDecode" => Filter::DCT,
        _ => return Err(Error::UnsupporttedFilter),
    })
}

pub fn decode_image(image: &ImageDecodeParam, image_bytes: &[u8]) -> Result<RgbImage, Error> {
    let decoded = match image.filter {
        Filter::Flate => {
            let deflater = ZlibDecoder::new(image_bytes);

            let decoded: Result<Vec<u8>, _> = deflater.bytes().collect();
            decoded.unwrap()
        }
        Filter::DCT => {
            let mut decoder = jpeg_decoder::Decoder::new(image_bytes);
            decoder.decode().unwrap()
        }
    };

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
