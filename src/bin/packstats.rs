extern crate gpeg;

use gpeg::{make_zigzag_table, read_data, Plane};

fn main() {
    let width = 1024;
    let height = 576;
    let y_plane = Plane {
        width: width,
        height: height,
        data: read_data("f1.Y"),
    };
    let cb_plane = Plane {
        width: width,
        height: height,
        data: read_data("f1.Cb"),
    };
    let cr_plane = Plane {
        width: width,
        height: height,
        data: read_data("f1.Cr"),
    };

    let block_width = (width / 8) as usize;
    let block_height = (height / 8) as usize;

    let num_blocks = block_width * block_height;
    let unpacked_size = num_blocks * 64 * 2;
    let zigzag = make_zigzag_table(1024);
    let mut total_nonzero_coeffs = 0;
    for by in 0..block_width {
        for bx in 0..block_height {
            let mut coeffs = vec![];
            let block_offset = by * 1024 + bx as usize;
            for j in 0..8 {
                for i in 0..8 {
                    coeffs.push(y_plane.data[block_offset + zigzag[j][i]]);
                }
            }
            let nonzero_coeffs = coeffs.iter().enumerate()
                .filter(|&(_, &x)| x != 0)
                // .map(|(i, _)| i + 1)
                // .last()
                // .unwrap_or(0);
                .count();
            total_nonzero_coeffs += nonzero_coeffs;
        }
    }
    println!("total blocks: {}", num_blocks);
    println!("total ceoffs: {}", total_nonzero_coeffs);
    println!("raw size: {}", unpacked_size);
    let packed_size = (num_blocks + total_nonzero_coeffs) * 2;
    println!("packed size: {}", packed_size);
    println!("size %: {:.2}", (packed_size as f32) / (unpacked_size as f32) * 100.);
}
