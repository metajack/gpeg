extern crate gpeg;

use gpeg::{make_zigzag_table, pack_coeffs, read_data};

fn unpack(p: u16) -> (u16, i16) {
    let zeros: u16 = p >> 12;
    let coeff: i16 = if (p & 0x800) == 0x800 {
        ((p & 0xfff) | 0xf000) as i16
    } else {
        (p & 0xfff) as i16
    };
    (zeros, coeff)
}

fn unpack_coeffs(width: u32, height: u32, data: &[u16]) -> Vec<i16> {
    let mut coeffs: Vec<i16> = vec![0; (width * height) as usize];
    let zigzag = make_zigzag_table(width);
    let block_width = width >> 3;
    let block_height = height >> 3;
    let mut data_idx = 0;
    for by in 0..block_height {
        for bx in 0..block_width {
            let block_offset = (by << 3) * width + (bx << 3);
            let mut zz_idx: usize = 0;
            while zz_idx < 64 {
                let (zeros, coeff) = unpack(data[data_idx]);
                // println!("read ({},{}) at index {} of block ({},{})",
                //          zeros, coeff, zz_idx, bx, by);
                data_idx += 1;
                if zeros == 0 && coeff == 0 {
                    break;
                }
                zz_idx += zeros as usize;
                coeffs[block_offset as usize + zigzag[zz_idx >> 3][zz_idx % 8]] = coeff;
                zz_idx += 1;
            }
        }
    }
    coeffs
}

fn print_block(data: &[i16], bx: u32, by: u32, stride: u32) {
    let block_offset = ((by << 3) * stride + (bx << 3)) as usize;
    for y in 0..8 {
        for x in 0..8 {
            print!("{:5}", data[block_offset + y * (stride as usize) + x]);
        }
        println!("");
    }
}

fn main() {
    let width = 1024;
    let height = 576;
    let planes = vec![
        (width, height, "f1.Y"),
        (width >> 1, height >> 1, "f1.Cb"),
        (width >> 1, height >> 1, "f1.Cr"),
        ];
    for (w, h, f) in planes {
        let data = read_data(f);
        let (packed, indices) = pack_coeffs(w, h, &data);
        let unpacked = unpack_coeffs(w, h, &packed);
        assert!(data.len() == unpacked.len());

        if w == width {
            println!("BLOCKS");
            print_block(&data, 0, 0, width);
            println!("");
            print_block(&unpacked, 0, 0, width);
            println!("");
            print_block(&data, 96, 25, width);
            println!("");
            print_block(&unpacked, 96, 25, width);

            println!("\nINDICES");
            println!("index={} is {}", 0*128+0, indices[0 * 128 + 0]);
            println!("index={} is {}", 1*128+0, indices[1 * 128 + 0]);
            println!("index={} is {}", 70*128+96, indices[70 * 128 + 96]);
                     
        }
        
        for i in 0..data.len() {
            if data[i] != unpacked[i] {
                println!("mismatch at ({},{}) in block ({}, {}): {} != {}",
                         i % (w as usize), i / (w as usize),
                         (i % (w as usize)) >> 3, (i / (w as usize)) >> 3,
                         data[i], unpacked[i]);
            }
        }
    }
}
