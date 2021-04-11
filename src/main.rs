use std::{error::Error, fs::File, io};
use std::{
    io::{BufReader, BufWriter, ErrorKind, Read, Stdout, Write},
    usize,
};

use anyhow::anyhow;

mod cli;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::create_app().get_matches();

    let sources = if let Some(values) = matches.values_of("FILE") {
        values.collect()
    } else {
        vec!["-"]
    };

    let decode = matches.is_present("DECODE");
    let ignore_ws = matches.is_present("WS");
    let wrap_size = matches
        .value_of("WRAP")
        .map(str::parse)
        .map(Result::unwrap)
        .unwrap_or(0);
    let mut app = App::new(ignore_ws, wrap_size);

    for src in sources {
        let src: Box<dyn Read> = if src == "-" {
            Box::new(io::stdin())
        } else {
            Box::new(File::open(src)?)
        };
        let mut src = BufReader::new(src);

        let err = if decode {
            app.decode_src(&mut src)
        } else {
            app.encode_src(&mut src)
        };

        // I want to use this app with pipes, so if we encourted broken one,
        // then return with Ok
        if let Err(err) = err {
            if let Some(err) = err.downcast_ref::<io::Error>() {
                if err.kind() == ErrorKind::BrokenPipe {
                    return Ok(());
                }
            }
            return Err(err.into());
        }
    }
    println!();
    Ok(())
}

// State of the app
struct App {
    ignore_ws: bool, // flag which is used during the encoding
    wrap_size: usize,
    column: usize, // current column in a line, if we want to wrap
}

/// Abstraction which helps us turn byte stream into stream, which buffers first char
/// and decodes byte if the second comes up
struct HexDecoder {
    out: BufWriter<Stdout>,
    last: Option<u8>,
}

impl HexDecoder {
    fn new() -> Self {
        let out = BufWriter::new(io::stdout());
        Self { out, last: None }
    }

    fn write(&mut self, digit: u8) -> Result<(), io::Error> {
        match self.last.take() {
            Some(hi) => {
                // we can safely unwrap because we already check if this is valid digit
                let hi = from_hex_digit(hi).unwrap();
                let lo = from_hex_digit(digit).unwrap();
                let byte = hi * 16 + lo;
                self.out.write(&[byte])?;
            }
            None => {
                self.last.replace(digit);
            }
        }
        Ok(())
    }

    /// We should check at the end if we have odd length
    fn finish(&self) -> anyhow::Result<()> {
        if self.last.is_some() {
            Err(anyhow!("Odd length"))
        } else {
            Ok(())
        }
    }
}

impl App {
    fn new(ignore_ws: bool, wrap_size: usize) -> Self {
        Self {
            ignore_ws,
            wrap_size,
            column: 0,
        }
    }

    /// Writes byte to stdout and wraps line if needed
    fn write(&mut self, out: &mut Stdout, c: u8) -> Result<(), io::Error> {
        write!(out, "{}", c as char)?;
        self.column += 1;
        if self.column == self.wrap_size {
            self.column = 0;
            println!();
        }
        Ok(())
    }

    /// Main encoding function
    fn encode_src(&mut self, src: &mut dyn io::Read) -> anyhow::Result<()> {
        let mut out = io::stdout();
        for byte in src.bytes() {
            let byte = byte?;
            let lo = to_hex_digit(byte & 0b111).unwrap();
            let hi = to_hex_digit(byte >> 4).unwrap();
            self.write(&mut out, hi)?;
            self.write(&mut out, lo)?;
        }
        Ok(())
    }

    /// Main decoding function
    fn decode_src(&self, src: &mut dyn io::Read) -> anyhow::Result<()> {
        let mut decoder = HexDecoder::new();
        for digit in src.bytes() {
            let digit = digit?;

            if digit == b'\n' || (self.ignore_ws && digit.is_ascii_whitespace()) {
                continue;
            }
            if !digit.is_ascii_hexdigit() {
                return Err(anyhow!("not ascii hexdigit: {:?}", digit as char));
            }
            decoder.write(digit)?;
        }
        decoder.finish()
    }
}

fn from_hex_digit(x: u8) -> Option<u8> {
    match x {
        b'0'..=b'9' => Some(x - b'0'),
        b'a'..=b'f' => Some(x - b'a' + 10),
        b'A'..=b'F' => Some(x - b'A' + 10),
        _ => None,
    }
}

const DIGITS: &[u8] = b"0123456789abcdef";

fn to_hex_digit(x: u8) -> Option<u8> {
    if x <= 0xf {
        Some(DIGITS[x as usize])
    } else {
        None
    }
}
