use chess_preproc::*;
use clap::{App, Arg, ArgMatches};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use semver::{Version, VersionReq};
use std::{env, io, process};
use tracing::{info, trace, warn};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{Layer, Registry};

pub mod arrows;
mod chess_preproc;

pub fn make_app() -> App<'static> {
    App::new(PREPROCESSOR_NAME)
        .about("A chess preprocessor which puts images of boards into your book")
        .subcommand(
            App::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    let filter = match env::var("RUST_LOG") {
        Ok(_) => EnvFilter::from_default_env(),
        _ => EnvFilter::new("mdbook_chess=info"),
    };

    let fmt = tracing_subscriber::fmt::Layer::default().with_writer(io::stderr);

    let subscriber = filter.and_then(fmt).with_subscriber(Registry::default());

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

fn main() {
    if let Err(e) = setup_logging() {
        eprintln!("Failed to setup tracing logging: {}", e);
    }
    let matches = make_app().get_matches();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(sub_args);
    } else if let Err(e) = handle_preprocessing() {
        info!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing() -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        warn!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            PREPROCESSOR_NAME,
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }
    info!("Processing!");

    let preproc = ChessPreprocessor;
    let processed_book = preproc.run(&ctx, book).expect("Failed to preprocess book");
    let s =
        serde_json::to_string(&processed_book).expect("Failed to convert processed book to json");
    println!("{}", s);
    Ok(())
}

fn handle_supports(sub_args: &ArgMatches) -> ! {
    let preproc = ChessPreprocessor;
    let renderer = sub_args.value_of("renderer").expect("Required argument");
    if preproc.supports_renderer(renderer) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
