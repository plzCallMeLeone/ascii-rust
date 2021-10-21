/*
 * ascii-rust :  Simple, fast ascii art generator written in Rust
 * version : 0.0.0.1
 * author : plzCallMeLeone
 *
 */
extern crate image;
extern crate fontdue;
extern crate itertools;

use image::{
    imageops::{ colorops, crop_imm, overlay},
    ImageBuffer,
    Luma,
};
use std::{
    fs::File, 
    io::{BufReader, Read}, 
    collections::BTreeMap,
    iter,
    env,
};
use fontdue::{Font, FontSettings};
use itertools::Itertools;


pub struct GlyphBmpData {
    pub name : String,
    pub width : u32,
    pub height : u32,
    pub data : BTreeMap<char, Vec<u8>>,
}

pub fn load_bytes(path: &str) -> <Vec<u8> {
    let mut bytes = Vec::<u8>::new();
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut bytes).unwrap();
    bytes
}

impl GlyphBmpData {
    fn extract_glyph_size<I>(font : &Font, px : f32, iter : I) -> (u32, u32) 
        where I: Iterator<Item = char>
    {
        let min_height = iter.map(|x| font.metrics(x, px).height).max().unwrap() as u32;

        let font_metrics = font.metrics(' ', px);
        let width = font_metrics.advance_width as u32;
        let height = if min_height > (px * 1.0) as u32 {
            min_height
        } else {
            (px * 1.0) as u32
        };
        (width, height)
    }
    pub fn new<I>(font : &Font, ch_range_iter : I, px : f32) -> Result<GlyphBmpData, &str> 
        where I: Iterator<Item = char> + Clone
    {
        // check all of the character in range is able to render.
        if !GlyphBmpData::is_valid_range(ch_range_iter.clone(), &font) {
            return Err("UnRenderable Character");
        }

        let (width, height) = GlyphBmpData::extract_glyph_size(&font, px, ch_range_iter.clone());
        let mut data = BTreeMap::<char, Vec<u8>>::new();


        for ch in ch_range_iter {
            let (_,mut bitmap) = font.rasterize(ch, px);

            let margin = (width * height) as usize - bitmap.len();
            bitmap.iter_mut().for_each(|x| {
                *x = 255 - *x;
            });
            bitmap.extend(iter::repeat(255u8).take(margin));
            data.insert(ch, bitmap);
        }
        Ok(GlyphBmpData {
            name : String::from("d2coding"),
            width,
            height,
            data,
        })
    }

    fn is_valid_range<I>(ch_range_iter : I, font : &Font) -> bool 
        where I: Iterator<Item = char> + Clone
    {
        let iter = ch_range_iter.clone()
                                .take_while( |ch| font.lookup_glyph_index(*ch) == 0)
                                .next();
        iter == None
    }

}


fn convert_grayimg_to_string(img_dat :&ImageBuffer<Luma<u8>, Vec<u8>>, gdata : &GlyphBmpData)->String {
    let c_width = gdata.width;
    let c_height = gdata.height;

    let num_of_tile = (img_dat.height() * img_dat.width()) / (c_width * c_height);
    let mut ret = String::new();
    ret.reserve(num_of_tile as usize);

    for y in (0..img_dat.height()).step_by(c_height as usize) {
        for x in (0..img_dat.width()).step_by(c_width as usize) {

            //crop image to font size;
            let piece = crop_imm(&*img_dat, x, y, c_width  , c_height);

            //find similarist font
            let (_, best_match) : (u64, &char) = gdata.data.iter().map(|(ch, dat)|{
                let diff = dat.iter() .zip(piece.to_image().as_raw().iter()) .map(|(a,b)|{
                    if a > b { (a - b) as u64 }
                    else { (b - a) as u64 }
                }).sum(); // get difference of each font and pieces
                (diff, ch)
            }).min().unwrap(); // find min difference.
            ret.push(*best_match);
        }
        ret.push('\n');
    }
    ret
}

pub fn string_to_grayimg(string : &str, gdata : &GlyphBmpData, size: (u32, u32)) -> ImageBuffer<Luma<u8>, Vec<u8>>{
    let (iwidth, iheight) = size;
    let fwidth = gdata.width;
    let fheight = gdata.height;
    
    let mut ret_img = ImageBuffer::<Luma<u8>, Vec<u8>>::new(iwidth, iheight);

    for ((y, x), ch) in (0..iheight).step_by(fheight as usize)
        .cartesian_product((0..iwidth).step_by(fwidth as usize))
        .zip(string.chars().filter(|&c| c != '\n')) {
            let dat = gdata.data.get(&ch).unwrap();
            let img_dat = ImageBuffer::from_vec(fwidth, fheight, dat.to_vec()).unwrap();
            overlay(&mut ret_img, &img_dat, x, y);
        }
    ret_img
}

fn main() {
    let bytes = load_bytes("d2coding.ttf");
    let font = Font::from_bytes(bytes, FontSettings::default()).unwrap();
    let font_data = GlyphBmpData::new(&font, ' '..'~', 8.0).unwrap_or_else(|err|{
        eprintln!("Error : {}", err);
        std::process::exit(1);
    });


    let image = image::open("test.jpg").unwrap();
    let grayimg = colorops::grayscale(&image);

    let ans = convert_grayimg_to_string(&grayimg, &font_data);

    let ascii_img = string_to_grayimg(&ans, &font_data, (grayimg.width(), grayimg.height()));
    ascii_img.save("a.jpg").unwrap();
}
