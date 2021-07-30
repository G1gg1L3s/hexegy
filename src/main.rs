use std::{fs::File, io};
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
    fn filter_broken_pipe(self) -> anyhow::Result<()>;
}

impl<E> FilterBrokenPipe for anyhow::Result<(), E>
where
    E: Into<anyhow::Error>,
{
    fn filter_broken_pipe(self) -> anyhow::Result<()> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => {
                let err = err.into().downcast::<io::Error>()?;
                if err.kind() == ErrorKind::BrokenPipe {
                    Ok(())
                } else {
                    Err(err.into())
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let matches = cli::create_app().get_matches();

    let sources = if let Some(values) = matches.values_of("FILE") {
        values.collect()
    } else {
        // if we have string from argument, we don't need to read from stdin
        if matches.is_present("STRING") {
            vec![]
        } else {
            vec!["-"]
        }
    };

    let decode = matches.is_present("DECODE");
    let ignore_ws = matches.is_present("WS");

    // can unwrap, because it is already checked
    let wrap_size = matches.value_of("WRAP").map_or(0, |x| x.parse().unwrap());
    let prefix = matches.value_of("PREFIX").unwrap_or("").to_string();
    let mut app = App::new(ignore_ws, wrap_size, prefix);

    // decodes or encodes source
    let mut process_src = |src: &mut dyn io::Read| -> Result<(), anyhow::Error> {
        let err = if decode {
            app.decode_src(src)
        } else {
            app.encode_src(src)
        };

        err.filter_broken_pipe()
    };

    // first check if we have argument string to encode/decode
    if let Some(string) = matches.value_of("STRING") {
        let mut string = io::Cursor::new(string);
        process_src(&mut string)?;
    }

    for src in sources {
        let src: Box<dyn Read> = if src == "-" {
            Box::new(io::stdin())
        } else {
            Box::new(File::open(src)?)
        };
        let mut src = BufReader::new(src);
        process_src(&mut src)?;
    }
    // for better ux we should append a newline to the end of our stream if we were encoding
    // so we check the last byte if the newline is already output.
    if let Some(b) = app.out.buffer().last() {
        if !decode && *b != b'\n' {
            writeln!(app.out).filter_broken_pipe()?;
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

/// An abstraction which helps us turn a byte stream into a stream, which buffers
/// first character and if the second comes up, decodes the byte and writes to the output
struct HexDecoder {
    last: Option<u8>,
}

impl HexDecoder {
    fn new() -> Self {
        Self { last: None }
    }

    fn write(&mut self, mut out: impl Write, digit: u8) -> Result<(), io::Error> {
        match self.last.take() {
            Some(hi) => {
                // we can safely unwrap because we already check if this is valid digit
                let hi = from_hex_digit(hi).unwrap();
                let lo = from_hex_digit(digit).unwrap();
                let byte = hi * 16 + lo;
                out.write_all(&[byte])?;
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
