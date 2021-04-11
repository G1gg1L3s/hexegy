use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings::ColoredHelp, Arg,
};

pub fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .settings(&[ColoredHelp])
        .arg(
            Arg::with_name("FILE")
                .help("Encode/decode data from a file.")
                .long("file")
                .short("f")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("DECODE")
                .help("Decode data")
                .long("decode")
                .short("d"),
        )
        .arg(
            Arg::with_name("WS")
                .help("Ignore whitespaces. By default only newlines ('\\n') are ignored.")
                .long("ignore-whitespaces")
                .short("i"),
        )
        .arg(
            Arg::with_name("WRAP")
            .help("wrap encoded lines after number of bytes (2 characters). Default is 0, which indicates no wrapping.")
            .long("wrap")
            .short("w")
            .takes_value(true)
            .validator(|x| {
                x.parse::<usize>().map(|_| ()).map_err(|err| err.to_string())
            })
        )
}
