extern crate gpeg;

use gpeg::{make_zigzag_table, read_data, Plane};

fn pack(zeros: u16, value: i16) -> u16 {
    (zeros << 12) | ((value as u16) & 0x0fff)
}

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
        let block_width = (plane.width / 8) as usize;
        let block_height = (plane.height / 8) as usize;

        let num_blocks = block_width * block_height;
        let unpacked_size = num_blocks * 64 * 2;
        let zigzag = make_zigzag_table(plane.width);

        for by in 0..block_width {
            for bx in 0..block_height {
                let mut coeffs = vec![];
                let block_offset = by * 1024 + bx as usize;
                for j in 0..8 {
                    for i in 0..8 {
                        coeffs.push(plane.data[block_offset + zigzag[j][i]]);
                    }
                }

                let mut packed: Vec<u16> = vec![];
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
                        zeros -= 15;
                    }
                    packed.push(pack(zeros, coeff));
                    zeros = 0;
                }
                // remaining zeros (if any) have a special symbol of 0, no
                // matter how many there were
                if zeros > 0 {
                    packed.push(0);
                }

                println!("plane {} block ({},{}): {:?}", plane_i, bx, by, packed);
                packed_coeffs += packed.len();
            }
        }
        println!("total blocks: {}", num_blocks);
        println!("raw size: {}", unpacked_size);
        println!("packed size: {}", packed_coeffs * 2);
        println!("% of original size: {:.2}%", ((packed_coeffs * 2) as f32) / (unpacked_size as f32) * 100f32);
    }
}
