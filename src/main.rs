use image::GenericImage;
use image::{DynamicImage, GenericImageView,ImageBuffer, imageops::resize,Rgb,Rgba, Pixel};
use std::u8;
use std::{fs};
use crate::fs::File;
use serde_json;
use std::collections::HashMap;
use serde_json::Result;
use std::io::Write;
use std::io::Read;
use std::io::BufReader;
use std::sync::Mutex;
use std::thread;
use std::sync::{ Arc};

fn main() {
    let seeds_images_dir = "./src/seed_images";
    // crop_images((&seeds_images_dir).to_string());
    // save_img_colors((&seeds_images_dir).to_string());
    make_mosaic_art();
}


fn crop_images(dir_name:String){
    let files = fs::read_dir(dir_name).unwrap();

    for (index,file) in files.enumerate(){
        let file = file.unwrap();
        let file_path = file.path();
        let img = image::open(file_path).unwrap();

        save_img(image::DynamicImage::ImageRgba8(resize_img(crop_img(img),50)),format!("./src/crop/{}.png",index.to_string()));
    }
}


fn save_img_colors(dir_name:String){
    let files = fs::read_dir(dir_name).unwrap();
    let mut color_code_dict = HashMap::new();
    for (index,file) in files.enumerate(){
        let file_dir = file.unwrap();
        let file_path = file_dir.path();
        let file = File::open(file_path).unwrap();
        let mut buf_reader = BufReader::new(file);
        let img =  image::load(&mut buf_reader, image::ImageFormat::Jpeg).unwrap();
        // let img = image::open(file_path).unwrap();
        color_code_dict.insert( index.to_string(),get_color_code( image::DynamicImage::ImageRgba8(resize_img(img,1))));

    }
    save_color_code(&color_code_dict).unwrap();
}

fn crop_img(mut img:DynamicImage)->DynamicImage{

    // 画像の幅と高さを取得
    let (width, height) = img.dimensions();

    // 切り取るサイズを計算
    let size = std::cmp::min(width, height);

    // 正方形に切り取る
    let cropped_img = img.crop(width/2-size/2, 0, size, size);
    return cropped_img
}

fn resize_img(img:DynamicImage,resize_num:u32)->ImageBuffer<Rgba<u8>, Vec<u8>>{
    let resized_img = resize(&img, resize_num, resize_num, image::imageops::FilterType::Triangle);
    resized_img
}

fn save_img(img:DynamicImage, save_path:String){
    img.save(save_path).unwrap();
}

fn get_color_code(img:DynamicImage)-> String {
    let pixel: Rgb<u8> = img.get_pixel(0, 0).to_rgb();
    return rgb_to_hex_string(&pixel)
}

fn save_color_code(color_codes: &HashMap<String,String>)->Result<()>{
    let serialized = serde_json::to_string(color_codes)?;
    let mut file = match File::create("color_code.json") {
        Ok(f) => f,
        Err(e) => panic!("Error creating color")
    };
    file.write_all(serialized.as_bytes()).unwrap();
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

fn calculate_color_distance(color1: [u8;3], color2: Rgb<u8>) -> i32{
    (color1[0]as i32 - color2[0] as i32).abs().pow(2)+(color1[1]as i32 - color2[1] as i32).abs().pow(2) + (color1[2]as i32 - color2[2] as i32).abs().pow(2)
}

fn make_mosaic_image_row(mosaic_img:DynamicImage,width:u32,height:u32,index:u32,color_code_json:HashMap<String, String>)->ImageBuffer<Rgba<u8>, Vec<u8>>{
    let mut result_mosaic_img:ImageBuffer<Rgba<u8>, Vec<u8>>=ImageBuffer::new(width*50, height/2*50);
for y in 0..height/2{
    for x in 0..width{
            let pixel = mosaic_img.get_pixel(x, y+height/2*(index)).to_rgb();
            let combine_image= image::open(format!("./src/crop/{}.png", calculate_min_color_distance_code(pixel,&color_code_json))).unwrap();
            match result_mosaic_img.copy_from(&combine_image, x*50, y*50){
                Ok(file)=>file,
                Err(err)=>panic!("{}",err)
            }
        }
    }
    result_mosaic_img
}

fn make_mosaic_art(){
    let mosaic_img = image::open("./src/source/seed.jpg").unwrap();
    let (width, height) = mosaic_img.dimensions();
    let color_code_json = load_color_code_json();

    let mut handles = vec![];
    let results = Arc::new(Mutex::new(Vec::new())); // 結果を格納する可変のVecをMutexで保護するArcを作成する
    for i in 0..2{
        let results = results.clone(); // Arcのクローンを作成し、スレッドに渡す
        let mosaic_img = mosaic_img.clone();
        let color_code_json = color_code_json.clone();
        let handle = thread::spawn(move || {
            let result = make_mosaic_image_row(mosaic_img,width.clone(),height.clone(),i,color_code_json);

            let mut results = results.lock().unwrap();
            results.push((i, result)); // 結果をMutexで保護された可変のVecに追加する
        });
        handles.push(handle);
    }


    for handle in handles {
        handle.join().unwrap();
    }

    let mut sorted_results = results.lock().unwrap(); // 結果のVecのロックを取得する
    sorted_results.sort_by(|a, b| a.0.cmp(&b.0)); // インデックス順に並べる
    let mut result_mosaic_img:ImageBuffer<Rgba<u8>, Vec<u8>>=ImageBuffer::new(width*50, height*50);
    for ((y,result)) in sorted_results.iter(){
        match result_mosaic_img.copy_from(result, 0, height/2*50) {
            Ok(file)=>file,
            Err(err)=>panic!("{}", err)
        }
    }
    result_mosaic_img.save("./result.png").unwrap();

}
fn calculate_min_color_distance_code(color: Rgb<u8>,image_colors:&HashMap<String,String>)->&str{
    let mut min_color_distance=196608;
    let mut min_image_key = "0";
    for (key,image_color) in  image_colors.iter(){
        let image_color_number = hex_to_rgb(image_color);
        let color_distance = calculate_color_distance(image_color_number, color);
        if color_distance < min_color_distance{
            min_color_distance = color_distance;
            min_image_key =key;
        }
    }
    min_image_key

}
fn load_color_code_json()->HashMap<String, String>{
    let mut file = File::open("./color_code.json").unwrap();
    // ファイルを文字列として読み込む
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // JSONをデシリアライズする
    let json_data:HashMap<String,String> = serde_json::from_str(&contents).unwrap();

    // HashMapを返す
    json_data
}