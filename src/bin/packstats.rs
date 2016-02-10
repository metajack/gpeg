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
            width: width,
            height: height,
            data: read_data("f1.Cb"),
        },
        Plane {
            width: width,
            height: height,
            data: read_data("f1.Cr"),
        },
    ];

    let mut packed_coeffs = 0;
    for (plane_i, plane) in planes.iter().enumerate() {
        let (packed, _) = pack_coeffs(plane.width, plane.height, &plane.data);
        packed_coeffs += packed.len();
        let num_blocks = (plane.width >> 3) * (plane.height >> 3);
        let unpacked_size = num_blocks * 64 * 2;

        println!("total blocks: {}", num_blocks);
        println!("raw size: {}", unpacked_size);
        println!("packed size: {}", packed_coeffs * 2);
        println!("% of original size: {:.2}%", ((packed_coeffs * 2) as f32) / (unpacked_size as f32) * 100f32);
    }
}
