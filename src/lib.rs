use std::fs::File;
use std::io::Read;

pub struct Plane {
    pub width: u32,
    pub height: u32,
    pub packed_coeffs: Vec<u16>,
    pub packed_indices: Vec<u32>,
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
            let base_row = BASE_TABLE[j * 8 + i] >> 3;
            let base_col = BASE_TABLE[j * 8 + i] % 8;
            table[j][i] = (base_row * stride + base_col) as usize;
        }
    }
    table
}

pub fn make_dezigzag_table(stride: u32) -> Vec<usize> {
    static BASE_TABLE: [u32; 64] = [
         0,  1,  5,  6, 14, 15, 27, 28,
         2,  4,  7, 13, 16, 26, 29, 42,
         3,  8, 12, 17, 25, 30, 41, 43,
         9, 11, 18, 24, 31, 40, 44, 53,
        10, 19, 23, 32, 39, 45, 52, 54,
        20, 22, 33, 38, 46, 51, 55, 60,
        21, 34, 37, 47, 50, 56, 59, 61,
        35, 36, 48, 49, 57, 58, 62, 63,
    ];
    let mut table: Vec<usize> = vec![0; 64];
    for j in 0..8 {
        for i in 0..8 {
            let base_row = BASE_TABLE[j * 8 + i] >> 3;
            let base_col = BASE_TABLE[j * 8 + i] % 8;
            table[j * 8 + i] = (base_row * stride + base_col) as usize;
        }
    }
    table
}

fn pack(zeros: u16, value: i16) -> u16 {
    (zeros << 12) | ((value as u16) & 0x0fff)
}

// returns a vec of packed coefficients and a vec of block indices
pub fn pack_coeffs(width: u32, height: u32, data: &[i16]) -> (Vec<u16>, Vec<u32>) {
    let block_width = (width >> 3) as usize;
    let block_height = (height >> 3) as usize;
    let zigzag = make_zigzag_table(width);

    let mut packed: Vec<u16> = vec![];
    let mut indices: Vec<u32> = vec![];

    for by in 0..block_height {
        for bx in 0..block_width {
            indices.push(packed.len() as u32);

            let block_offset = (by << 3) * (width as usize) + (bx << 3);
            let mut coeffs = vec![];
            for j in 0..8 {
                for i in 0..8 {
                    coeffs.push(data[block_offset + zigzag[j][i]]);
                }
            }

            let mut zeros = 0;
            for coeff in coeffs {
                if coeff == 0 {
                    zeros += 1;
                    continue
                }
                while zeros > 15 {
                    // we must store 15 zeros with a zero, followed by our
                    // packed coefficient
                    packed.push(pack(15, 0));
                    zeros -= 16;
                }
                packed.push(pack(zeros, coeff));
                zeros = 0;
            }
            // remaining zeros (if any) have a special symbol of 0, no
            // matter how many there were
            if zeros > 0 {
                packed.push(0);
            }
        }
    }

    // need to pad packed coefficients to a multiple of 512
    let overage = packed.len() % 512;
    if overage > 0 {
        let extra = 512 - overage;
        packed.reserve(extra);
        for _ in 0..extra {
            packed.push(0);
        }
    }

    (packed, indices)
}
