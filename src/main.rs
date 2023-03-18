use crate::fs::File;
use core::panic;
use image::GenericImage;
use image::{imageops::resize, DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb, Rgba};
use serde_json;
use serde_json::Result;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::u8;
use std::env;
use prog_rs::prelude::*;


fn main() {
    let args: Vec<String> = env::args().collect();
    let seeds_images_dir = "./src/seed_images";
    if args.len() == 2{
        if args[1]=="update"{
            println!("update started");
            crop_images((&seeds_images_dir).to_string());
            save_img_colors((&seeds_images_dir).to_string());
            println!("update was finished");
        }
    }
    make_mosaic_art();
}

fn crop_images(dir_name: String) {
    let files = match fs::read_dir(dir_name) {
        Ok(f) => f,
        Err(_) => panic!("Error reading directory"),
    };

    for (index, file) in files.enumerate() {
        let file = file.unwrap();
        let file_path = file.path();
        let img = match image::open(file_path.clone()) {
            Ok(f) => f,
            Err(_) => {
                println!("can't open {:?}", file_path);
                continue;
            }
        };

        save_img(
            image::DynamicImage::ImageRgba8(resize_img(crop_img(img), 50)),
            format!("./src/crop/{}.png", index.to_string()),
        );
    }
}

fn save_img_colors(dir_name: String) {
    let files = match fs::read_dir(dir_name) {
        Ok(f) => f,
        Err(e) => panic!("can't read directory on save_img_colors {}",e),
    };
    let mut color_code_dict = HashMap::new();
    for (index, file) in files.enumerate() {
        let file_dir = file.unwrap();
        let file_path = file_dir.path();
        let img = match image::open(file_path.clone()) {
            Ok(f) => f,
            Err(_) => {
                println!("can't open {:?}", file_path);
                continue;
            }
        };
        color_code_dict.insert(
            index.to_string(),
            get_color_code(image::DynamicImage::ImageRgba8(resize_img(img, 1))),
        );
    }
    match save_color_code(&color_code_dict) {
        Err(_) => panic!("error was occurred while saving color code"),
        Ok(_) => {}
    };
}

fn crop_img(mut img: DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    let size = std::cmp::min(width, height);

    let cropped_img = img.crop(width / 2 - size / 2, 0, size, size);
    return cropped_img;
}

fn resize_img(img: DynamicImage, resize_num: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let resized_img = resize(
        &img,
        resize_num,
        resize_num,
        image::imageops::FilterType::Triangle,
    );
    resized_img
}

fn save_img(img: DynamicImage, save_path: String) {
    match img.save(save_path.clone()) {
        Ok(_) => {}
        Err(err) => println!("cannot save {}: {}", save_path, err),
    }
}

fn get_color_code(img: DynamicImage) -> String {
    let pixel: Rgb<u8> = img.get_pixel(0, 0).to_rgb();
    return rgb_to_hex_string(&pixel);
}

fn save_color_code(color_codes: &HashMap<String, String>) -> Result<()> {
    let serialized = serde_json::to_string(color_codes)?;
    let mut file = match File::create("color_code.json") {
        Ok(f) => f,
        Err(_) => panic!("Error creating color"),
    };
    match file.write_all(serialized.as_bytes()) {
        Ok(_) => {}
        Err(_) => panic!("Error writing color code"),
    }
    Ok(())
}

fn rgb_to_hex_string(rgb: &Rgb<u8>) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

fn hex_to_rgb(hex: &str) -> [u8; 3] {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();

    [r, g, b]
}

fn calculate_color_distance(color1: [u8; 3], color2: Rgb<u8>) -> i32 {
    (color1[0] as i32 - color2[0] as i32).abs().pow(2)
        + (color1[1] as i32 - color2[1] as i32).abs().pow(2)
        + (color1[2] as i32 - color2[2] as i32).abs().pow(2)
}

fn make_mosaic_image_row(
    mosaic_img: DynamicImage,
    width: u32,
    height: u32,
    index: u32,
    color_code_json: HashMap<String, String>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut result_mosaic_img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::new(width * 50, height / 2 * 50);
    for y in (0..height / 2).progress().with_prefix("make image row...") {
        for x in 0..width {
            let pixel = mosaic_img.get_pixel(x, y + height / 2 * (index)).to_rgb();
            let file_path = format!(
                "./src/crop/{}.png",
                calculate_min_color_distance_code(pixel, &color_code_json)
            );
            let combine_image = match image::open(file_path.clone()) {
                Ok(f) => f,
                Err(_) => {
                    println!("cannot open image {}", file_path);
                    continue;
                }
            };
            match result_mosaic_img.copy_from(&combine_image, x * 50, y * 50) {
                Ok(file) => file,
                Err(err) => panic!("{}", err),
            }
        }
    }
    result_mosaic_img
}

fn make_mosaic_art() {
    let mosaic_img = match image::open("./src/source/seed.jpg") {
        Ok(f) => f,
        Err(e) => panic!("cannot open mosaic_image {}", e),
    };
    let (width, height) = mosaic_img.dimensions();
    let color_code_json = load_color_code_json();

    let mut handles = vec![];
    let results = Arc::new(Mutex::new(Vec::new()));
    for i in 0..2 {
        let results = results.clone();
        let mosaic_img = mosaic_img.clone();
        let color_code_json = color_code_json.clone();
        let handle = thread::spawn(move || {
            let result = make_mosaic_image_row(
                mosaic_img,
                width.clone(),
                height.clone(),
                i,
                color_code_json,
            );
            let mut results = match results.lock() {
                Ok(result) => result,
                Err(e) => {
                    panic!("some process error {}",e);
                }
            };
            results.push((i, result));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut sorted_results = results.lock().unwrap();
    sorted_results.sort_by(|a, b| a.0.cmp(&b.0));
    let mut result_mosaic_img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::new(width * 50, height * 50);
    for (_, result) in sorted_results.iter() {
        match result_mosaic_img.copy_from(result, 0, height / 2 * 50) {
            Ok(file) => file,
            Err(err) => panic!("{}", err),
        }
    }
    println!("Saving...");
    match result_mosaic_img.save("./result.png") {
        Ok(_) => {}
        Err(e) => {
            panic!("cannot save result mosaic{}", e)
        }
    }
}
fn calculate_min_color_distance_code(
    color: Rgb<u8>,
    image_colors: &HashMap<String, String>,
) -> &str {
    let mut min_color_distance = 196608;
    let mut min_image_key = "0";
    for (key, image_color) in image_colors.iter() {
        let image_color_number = hex_to_rgb(image_color);
        let color_distance = calculate_color_distance(image_color_number, color);
        if color_distance < min_color_distance {
            min_color_distance = color_distance;
            min_image_key = key;
        }
    }
    min_image_key
}
fn load_color_code_json() -> HashMap<String, String> {
    let mut file = match File::open("./color_code.json") {
        Ok(file) => file,
        Err(e) => panic!("cannot open color_code.json {}", e),
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {}
        Err(e) => {
            panic!("cannot load color_code.json {}", e)
        }
    }

    let json_data: HashMap<String, String> = match serde_json::from_str(&contents) {
        Ok(f) => f,
        Err(e) => {
            panic!("cannot deserialize color_code.json {}", e);
        }
    };

    json_data
}
