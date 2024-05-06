use std::{str, fs::File, io::{BufReader, Read}};
use image::GenericImageView;
use image::io::Reader as ImageReader;
use inflate;

const DEFAULT_NEXT_POS: i128 = 0x0b400;

fn main() {
    let png_path = "test.png";
    let out_path = "test_out.xml";
    let decoded_data = read_png(png_path);
    let _ = std::fs::write(out_path,decoded_data);
}

const BLUE: usize = 2;
const GREEN: usize = 1;
const RED: usize = 0;
const ALPHA: usize = 3;

/// Pulls out the colors of the png into a byte vector, feeds it into the decoder,
/// then undoes the inflation algorithm and returns the raw bytes.
pub fn read_png(png_path: &str) -> Vec<u8> {
    let extra_data = get_spore_section(png_path);

    let img = ImageReader::open(png_path).unwrap().decode().unwrap();
    let mut img_data = vec![];
    for pixel in img.pixels() {
        img_data.push(pixel.2[BLUE]);
        img_data.push(pixel.2[GREEN]);
        img_data.push(pixel.2[RED]);
        img_data.push(pixel.2[ALPHA]);
    }
    let mut dec_dat = decode(img_data);
    
    if let Some(mut more_dat) = extra_data {
        more_dat.drain(..8);
        more_dat.append(&mut dec_dat);
        return inflate::inflate_bytes_zlib_no_checksum(&more_dat).unwrap();
    }
    return inflate::inflate_bytes_zlib_no_checksum(&dec_dat).unwrap();
}

/// Gets extra data from the png if it exists
fn get_spore_section(path: &str) -> Option<Vec<u8>> {
    let mut stream = BufReader::new(File::open(path).unwrap());

    let _ = stream.seek_relative(8);
    let mut type_code = 0;
    while type_code != 0x49454E44 {
        let length = read_i32(&mut stream);
        type_code = read_i32(&mut stream);

        let _ = stream.seek_relative(length.into());
        read_i32(&mut stream);   // crc, we don't care
    }

    if stream.buffer().is_empty() {return None;}

    let length = read_i32(&mut stream) as usize;

    if read_i32(&mut stream) != 0x73704F72 {return None;} // Invalid extra data

    let mut data = vec![0; length];
    let _ = stream.read(&mut data);
    return Some(data);
}
/// Reads in 4 bytes from a bufreader and returns them as an int
fn read_i32(stream: &mut BufReader<File>) -> i32 {
    let mut data: [u8; 4] = [0,0,0,0];
    let _ = stream.read(&mut data);
    arr_to_i(&data)
}
/// This is where the magic happens.
/// By magic I mean legit magic what the fuck is happening here
fn decode(img_data: Vec<u8>) -> Vec<u8> {
    let mut hash: u128 = 0x811c9dc5;
    let mut next_pos: u128 = 0x0b400;
    let mut len_dat = vec![];
    let mut dec_data = vec![];

    'outer: for _counter in 0..8 {
        let mut byte: u8 = 0;

        for _ in 0..8 {
            let data: u128 = img_data[next_pos as usize] as u128;
            let mut magic_number = (hash * 0x1000193) & 0xffffffff;

            hash = magic_number ^ ((next_pos & 7) | (data & 0xf8));
            magic_number = ((data & 1) << 7) ^ ((hash & 0x8000) >> 8);
            next_pos = (next_pos >> 1) ^ ((DEFAULT_NEXT_POS & -(next_pos as i128 & 1)) as u128);

            if next_pos == DEFAULT_NEXT_POS as u128 {break 'outer;}
            
            byte = (byte >> 1) | magic_number as u8;
        }
        len_dat.push(byte);
    };
    let ln = arr_to_i_revendian(&len_dat[4..8]); // Gets the length of the next section

    // PART 2--------------------------------------------------------
    'outer: for _counter in 0..ln {
        let mut byte: u8 = 0;

        for _ in 0..8 {
            let data: u128 = img_data[next_pos as usize] as u128;
            let mut magic_number = (hash * 0x1000193) & 0xffffffff;

            hash = magic_number ^ ((next_pos & 7) | (data & 0xf8));
            magic_number = ((data & 1) << 7) ^ ((hash & 0x8000) >> 8);
            next_pos = (next_pos >> 1) ^ ((DEFAULT_NEXT_POS & -(next_pos as i128 & 1)) as u128);

            if next_pos == DEFAULT_NEXT_POS as u128 {break 'outer;}
            
            byte = (byte >> 1) | magic_number as u8;
        }
        dec_data.push(byte);
    }
    return dec_data;
}
/// Converts 4 bytes into an int
fn arr_to_i(array: &[u8; 4]) -> i32 {
    ((array[0] as i32) << 24) + ((array[1] as i32) << 16) +
    ((array[2] as i32) <<  8) + ((array[3] as i32) <<  0)
}
// WHY ARE BOTH KINDS OF ENDIANNESS USED IN THIS WHAT THE FUCK BRUH
fn arr_to_i_revendian(array: &[u8]) -> i32 {
    ((array[3] as i32) << 24) + ((array[2] as i32) << 16) +
    ((array[1] as i32) <<  8) + ((array[0] as i32) <<  0)
}