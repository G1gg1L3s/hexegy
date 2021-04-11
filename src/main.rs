use std::{error::Error, fs::File, io};
use std::{
    io::{BufReader, BufWriter, ErrorKind, Read, Stdout, Write},
    usize,
};

use anyhow::anyhow;

mod cli;

// A trait for better error handling.
// I want to use app with unix pipes, so I want to filter errors with broken ones
// to create better user experience.
trait FilterBrokenPipe {
    fn filter_broken_pipe(self) -> Result<(), anyhow::Error>;
}

impl FilterBrokenPipe for anyhow::Error {
    fn filter_broken_pipe(self) -> Result<(), anyhow::Error> {
        let err = self.downcast::<io::Error>()?;
        if err.kind() == ErrorKind::BrokenPipe {
            Ok(())
        } else {
            Err(err.into())
        }
    }
}

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
    let prefix = matches.value_of("PREFIX").unwrap_or("").to_string();
    let mut app = App::new(ignore_ws, wrap_size, prefix);

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

        if let Err(err) = err {
            err.filter_broken_pipe()?;
        }
    }
    // for better ux we should append a newline to the end of our stream if we were encoding
    // so we check the last byte if the newline is already output.
    if let Some(b) = app.out.buffer().last() {
        if !decode && *b != b'\n' {
            if let Err(err) = writeln!(app.out) {
                anyhow::Error::from(err).filter_broken_pipe()?;
            }
        }
    }
    Ok(())
}

// State of the app
struct App {
    ignore_ws: bool, // flag which is used during the encoding
    wrap_size: usize,
    column: usize, // current column in a line, if we want to wrap
    prefix: String,
    out: BufWriter<Stdout>,
}

/// Abstraction which helps us turn byte stream into stream, which buffers first char
/// and decodes byte if the second comes up
struct HexDecoder {
    last: Option<u8>,
}

impl HexDecoder {
    fn new() -> Self {
        Self { last: None }
    }

    fn write(&mut self, out: &mut impl Write, digit: u8) -> Result<(), io::Error> {
        match self.last.take() {
            Some(hi) => {
                // we can safely unwrap because we already check if this is valid digit
                let hi = from_hex_digit(hi).unwrap();
                let lo = from_hex_digit(digit).unwrap();
                let byte = hi * 16 + lo;
                out.write(&[byte])?;
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
    fn new(ignore_ws: bool, wrap_size: usize, prefix: String) -> Self {
        let out = BufWriter::new(io::stdout());
        Self {
            ignore_ws,
            wrap_size,
            column: 0,
            out,
            prefix,
        }
    }

    /// Writes byte to stdout and wraps line if needed
    fn write(&mut self, c: u8) -> Result<(), io::Error> {
        write!(self.out, "{}{:02x}", self.prefix, c)?;
        self.column += 1;
        if self.column == self.wrap_size {
            self.column = 0;
            writeln!(self.out)?;
        }
        Ok(())
    }

    /// Main encoding function
    fn encode_src(&mut self, src: &mut dyn io::Read) -> anyhow::Result<()> {
        for byte in src.bytes() {
            let byte = byte?;
            self.write(byte)?;
        }
        Ok(())
    }

    /// Main decoding function
    fn decode_src(&mut self, src: &mut dyn io::Read) -> anyhow::Result<()> {
        let mut decoder = HexDecoder::new();
        for digit in src.bytes() {
            let digit = digit?;

            if digit == b'\n' || (self.ignore_ws && digit.is_ascii_whitespace()) {
                continue;
            }
            if !digit.is_ascii_hexdigit() {
                return Err(anyhow!("not ascii hexdigit: {:?}", digit as char));
            }
            decoder.write(&mut self.out, digit)?;
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
