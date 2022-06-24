use std::path::{Path, PathBuf};
use std::io::Write;
use bitreader::BitReader;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "bfbf.rs", about = "An Encoder and decoder for bfbf files.")]
struct Opt {
    ///Set to false by default. If true, it will ignore all formatting and indentation when encoding.
    #[structopt(short = "l", long = "lossy")]
    lossy: bool,

    //Enable debugging print view
    #[structopt(short = "d", long = "debug")]
    debug: bool,

    /// Path to input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let args = Opt::from_args();

    let path: &Path = args.input.as_path();
    let filename = path.file_stem().unwrap();
    let extension = path.extension().unwrap();
    let decode = extension != "bf";
    let mut i = 0;
    let f = std::fs::read(path).unwrap();
    let mut reader = BitReader::new(&f);

    if decode {
        let mut out = "".to_owned();
        while i < (f.len() * 8) {
            if reader.read_bool().unwrap() && args.lossy == false {
                let indent = reader.read_u8(3).unwrap();
                match indent {
                    0b001 => out.push_str("\t"),
                    0b010 => out.push_str("\n"),
                    _ => {}
                }
            } else {
                let cmd = reader.read_u8(3).unwrap();
                match cmd {
                    0b000 => out.push_str(">"),
                    0b001 => out.push_str("<"),
                    0b010 => out.push_str("+"),
                    0b011 => out.push_str("-"),
                    0b100 => out.push_str("."),
                    0b101 => out.push_str(","),
                    0b110 => out.push_str("["),
                    0b111 => out.push_str("]"),
                    _ => {},
                }
            }
            if out.len() != 0 && args.debug {
                println!("Written Char: '{}'", out.chars().last().unwrap().escape_default().to_string());
            }
            i += 4;
        }
        let mut file = std::fs::File::create(format!("{}.d.bf", filename.to_str().unwrap())).expect("Cannot create file!");
        write!(file, "{}", out).expect("Could not write to file!");
        println!("\nSucessfully written to file '{}.d.bf'", filename.to_str().unwrap());
    } else {
        let mut file = std::fs::File::create(format!("{}.bfbf", filename.to_str().unwrap())).expect("Cannot create file!");

        let mut shift: u8 = 4;
        let mut current_byte: u8 = 0;
        let mut byte_aligned = true;

        let mut bytes_written: f32 = 0.0;
        if args.debug {
            println!("PREV CURR | Char | Align | Debug\n----------|------|-------|----->");
        }
        while i < (f.len() * 8) {
            let char = reader.read_u8(8).unwrap() as char;
            if char == '\r' {
                i += 8;
                continue;
            }

            match char {
                '>' => { current_byte += 0b0000 << shift; byte_aligned = !byte_aligned; },
                '<' => { current_byte += 0b0001 << shift; byte_aligned = !byte_aligned; },
                '+' => { current_byte += 0b0010 << shift; byte_aligned = !byte_aligned; },
                '-' => { current_byte += 0b0011 << shift; byte_aligned = !byte_aligned; },
                '.' => { current_byte += 0b0100 << shift; byte_aligned = !byte_aligned; },
                ',' => { current_byte += 0b0101 << shift; byte_aligned = !byte_aligned; },
                '[' => { current_byte += 0b0110 << shift; byte_aligned = !byte_aligned; },
                ']' => { current_byte += 0b0111 << shift; byte_aligned = !byte_aligned; },

                '\t'=> { if args.lossy == false {
                        current_byte += 0b1001 << shift;
                        byte_aligned = !byte_aligned;
                    } else {
                        i += 8; continue;
                    }
                },
                '\n'=> { if args.lossy == false {
                        current_byte += 0b1010 << shift;
                        byte_aligned = !byte_aligned;
                    } else {
                        i += 8; continue;
                    }
                },
                _ => { i += 8; continue; },
            }
            if args.debug {
                print!("{:04b} {:04b} | {:^4} | {:^5} |", current_byte >> 4, current_byte &  0x0F, char.escape_default().to_string(), byte_aligned);
            }
            if byte_aligned {
                if args.debug {
                    print!(" Written Byte");
                }
                bytes_written += 1.0;
                file.write(&[current_byte]).expect("Unable to write byte to file!");
                current_byte = 0;
            }

            if shift == 4 {
                shift = 0;
            } else {
                shift = 4;
            }
            if args.debug {
                print!("\n");
            }
            i += 8;
        }

        if !byte_aligned {
            current_byte += 0b1000;
            bytes_written += 1.0;
            if args.debug {
                println!("{:04b} {:04b} | {:^4} | {:^5} | Byte Alignment", current_byte >> 4, current_byte & 0x0F, "ALGN", byte_aligned);
            }
            file.write(&[current_byte]).expect("Unable to write byte to file!");
        }
        println!("\nSucessfully written to file '{}.bfbf' with a saving space of {:.2}%", filename.to_str().unwrap(), (1.0 - (bytes_written / f.len() as f32)) * 100.0);
    }
}
