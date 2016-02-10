extern crate gpeg;

use gpeg::{make_zigzag_table, pack_coeffs, read_data, Plane};

fn main() {
    let width = 1024;
    let height = 576;
    let planes = vec![
        Plane {
            width: width,
            height: height,
            data: read_data("f1.Y"),
        },
        Plane {
            width: width / 2,
            height: height / 2,
            data: read_data("f1.Cb"),
        },
        Plane {
            width: width / 2,
            height: height / 2,
            data: read_data("f1.Cr"),
        },
    ];

    let mut packed_coeffs = 0;
    for (plane_i, plane) in planes.iter().enumerate() {
        let (packed, _) = pack_coeffs(plane.width, plane.height, &plane.data);
        packed_coeffs += packed.len();
        let num_blocks = (plane.width >> 3) * (plane.height >> 3);
        let unpacked_size = num_blocks * 64 * 2;

        println!("plane {}: total blocks: {}", plane_i, num_blocks);
        println!("plane {}: raw size: {}", plane_i, unpacked_size);
        println!("plane {}: packed size: {}", plane_i, packed_coeffs * 2);
        println!("plane {}: % of original size: {:.2}%", plane_i, ((packed_coeffs * 2) as f32) / (unpacked_size as f32) * 100f32);
    }
}
