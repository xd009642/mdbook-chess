use clap::{App, Arg, ArgMatches};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook_chess::*;
use semver::{Version, VersionReq};
use std::{io, process};

pub fn make_app() -> App<'static> {
    App::new(PREPROCESSOR_NAME)
        .about("A chess preprocessor which puts images of boards into your book")
        .subcommand(
            App::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(sub_args);
    } else if let Err(e) = handle_preprocessing() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing() -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            PREPROCESSOR_NAME,
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }
    eprintln!("Processing!");

    let processed_book = run_preprocessor(&ctx, book).expect("FUCK1");
    let s = serde_json::to_string(&processed_book).expect("FUCK");
    println!("{}", s);
    eprintln!("{}", s);
    Ok(())
}

fn handle_supports(sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    if renderer != "not-supported" {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
