extern crate clap;
extern crate gpeg;

use clap::{Arg, App};
use gpeg::make_zigzag_table;
use std::fs::File;
use std::io::{self, BufReader, Read};

#[derive(Debug, PartialEq, Eq)]
enum FrameType {
    BaselineDct,
}

#[derive(Debug, PartialEq, Eq)]
enum SegmentType {
    StartOfImage,
    App(u8),
    DefineQuantizationTable,
    StartOfFrame(FrameType),
    DefineHuffmanTables,
    StartOfScan,
    EndOfImage
}

fn skip(input: &mut Iterator<Item=io::Result<u8>>, bytes: usize) {
    for _ in 0..bytes {
        match input.next() {
            Some(_) => (),
            None => panic!("ran out of data trying to skip"),
        }
    }
}

fn read_word(input: &mut Iterator<Item=io::Result<u8>>) -> u16 {
    let hi = input.next().unwrap().unwrap();
    let lo = input.next().unwrap().unwrap();
    ((hi as u16) << 8) | (lo as u16)
}

fn parse_header(input: &mut Iterator<Item=io::Result<u8>>, hi: u8, lo: u8) -> (SegmentType, usize) {
    assert!(hi == 0xff);
    let ty = match lo {
        0xd8 => SegmentType::StartOfImage,
        code @ 0xe0...0xef => SegmentType::App(code & 0xf),
        0xdb => SegmentType::DefineQuantizationTable,
        0xc0 => SegmentType::StartOfFrame(FrameType::BaselineDct),
        0xc4 => SegmentType::DefineHuffmanTables,
        0xda => SegmentType::StartOfScan,
        0xd9 => SegmentType::EndOfImage,
        code => panic!("unknown segment {:x}", code),
    };
    let len = match ty {
        SegmentType::StartOfImage => 0,
        SegmentType::EndOfImage => 0,
        _ => read_word(input) - 2,
    };
    (ty, len as usize)
}

fn read_header(input: &mut Iterator<Item=io::Result<u8>>) -> (SegmentType, usize) {
    let hi = input.next().unwrap().unwrap();
    let lo = input.next().unwrap().unwrap();
    parse_header(input, hi, lo)
}

fn find_marker(input: &mut Iterator<Item=io::Result<u8>>) -> (SegmentType, usize) {
    loop {
        let hi = input.next().unwrap().unwrap();
        if hi == 0xff {
            let lo = input.next().unwrap().unwrap();
            if lo != 0x00 {
                return parse_header(input, hi, lo);
            }
        }
    }
}

fn read_quantization_table(input: &mut Iterator<Item=io::Result<u8>>) -> (usize, Vec<u16>) {
    let pqtq = input.next().unwrap().unwrap();
    // only support baseline which has pq=0 always
    assert!((pqtq & 0xf0) == 0);
    let tq = (pqtq & 0x0f) as usize;
    let zigzag = make_zigzag_table(8);
    let mut table = vec![0; 64];
    for j in 0..8 {
        for i in 0..8 {
            table[zigzag[j][i]] = input.next().unwrap().unwrap() as u16;
        }
    }
    (tq, table)
}

fn main() {
    let matches = App::new("dumpsegments")
        .about("Dumps marker segments from a JPEG file")
        .arg(Arg::with_name("INPUT")
             .help("Input JPEG file")
             .required(true)
             .index(1))
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();

    println!("parsing {}...", input);

    let f = File::open(input).unwrap();
    let r = BufReader::new(f);
    let mut input = r.bytes();

    let mut head = read_header(&mut input);
    while head.0 != SegmentType::EndOfImage {
        println!("segment {:?}", head);
        match head.0 {
            SegmentType::StartOfScan => {
                println!("skipping to next marker");
                head = find_marker(&mut input);
            },
            SegmentType::DefineQuantizationTable => {
                let (tq, table) = read_quantization_table(&mut input);
                println!("QUANT TABLE {}", tq);
                for j in 0..8 {
                    for i in 0..8 {
                        print!("{:4} ", table[j * 8 + i]);
                    }
                    println!("");
                }
                
                head = read_header(&mut input);
            },
            _ => {
                println!("skipping {} bytes", head.1);
                skip(&mut input, head.1);
                head = read_header(&mut input);
            }
        }
    }
    println!("segment {:?}", head);
}
