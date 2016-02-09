use std::fs::File;
use std::io::Read;

pub struct Plane {
    pub width: u32,
    pub height: u32,
    pub data: Vec<i16>,
}

pub fn read_data(file: &str) -> Vec<i16> {
        let mut f = File::open(file).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s.split_whitespace()
            .map(|coeff_str| i16::from_str_radix(coeff_str, 10).unwrap())
            .collect()
}

pub fn make_zigzag_table(stride: u32) -> Vec<Vec<usize>> {
    static BASE_TABLE: [u32; 64] = [
        0,  1,  8, 16,  9,  2,  3, 10,
        17, 24, 32, 25, 18, 11,  4,  5,
        12, 19, 26, 33, 40, 48, 41, 34,
        27, 20, 13,  6,  7, 14, 21, 28,
        35, 42, 49, 56, 57, 50, 43, 36,
        29, 22, 15, 23, 30, 37, 44, 51,
        58, 59, 52, 45, 38, 31, 39, 46,
        53, 60, 61, 54, 47, 55, 62, 63,
    ];
    let mut table: Vec<Vec<usize>> = vec![vec![0; 8]; 8];
    for j in 0..8 {
        for i in 0..8 {
            let base_row = BASE_TABLE[j * 8 + i] / 8;
            let base_col = BASE_TABLE[j * 8 + i] % 8;
            table[j][i] = (base_row * stride + base_col) as usize;
        }
    }
    table
}
