use chess::{Board, ChessMove, Color, File, Piece, Rank, Square};
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error as MdBookError;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::{fmt, fs, io};
use tracing::{error, info};

/// A constant X axis offset to apply to all pieces (and pawns).
const X_OFFSET: f32 = 0.6;
/// A constant Y axis offset to apply to all pieces (and pawns).
const Y_OFFSET: f32 = 0.3;
/// The name of the plugin.
pub const PREPROCESSOR_NAME: &'static str = "mdbook-chess";

const fn true_value() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ManyOrOne {
    One(String),
    Many(Vec<String>),
}

impl ManyOrOne {
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a String> + 'a> {
        match self {
            Self::One(s) => Box::new(std::iter::once(s)),
            Self::Many(s) => Box::new(s.iter()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoardBlock {
    /// The name we want to refer to this board by as we mutate it
    load: Option<String>,
    /// This lets us go back to it in future
    save: Option<ManyOrOne>,
    /// If we're creating a new board
    board: Option<String>,
    /// Moves to apply to the board
    #[serde(default)]
    moves: Vec<String>,
    #[serde(default = "true_value")]
    overwrite: bool,
}

/// Returns the SVG for the board for us to embed.
const fn get_board() -> &'static str {
    include_str!("../res/board.svg")
}

/// Gets the templaed SVG path for pieces.
const fn get_piece(p: Piece, c: Color) -> &'static str {
    match (p, c) {
        (Piece::Pawn, Color::White) => include_str!("../res/P.svg"),
        (Piece::Knight, Color::White) => include_str!("../res/N.svg"),
        (Piece::Bishop, Color::White) => include_str!("../res/B.svg"),
        (Piece::Rook, Color::White) => include_str!("../res/R.svg"),
        (Piece::Queen, Color::White) => include_str!("../res/Q.svg"),
        (Piece::King, Color::White) => include_str!("../res/K.svg"),
        (Piece::Pawn, Color::Black) => include_str!("../res/p.svg"),
        (Piece::Knight, Color::Black) => include_str!("../res/n.svg"),
        (Piece::Bishop, Color::Black) => include_str!("../res/b.svg"),
        (Piece::Rook, Color::Black) => include_str!("../res/r.svg"),
        (Piece::Queen, Color::Black) => include_str!("../res/q.svg"),
        (Piece::King, Color::Black) => include_str!("../res/k.svg"),
    }
}

/// For a given rank and file return (x, y) coordinate.
fn coordinate(file: File, rank: Rank) -> (f32, f32) {
    let rank = 70 - (rank.to_index() * 10);
    (
        (file.to_index() * 10) as f32 + X_OFFSET,
        rank as f32 + Y_OFFSET,
    )
}

/// Given a board layout generates an SVG string for the board
pub fn generate_board(board: &Board) -> String {
    let mut pieces = String::new();
    for i in 0..64 {
        let square = unsafe { Square::new(i) };
        if let Some((piece, color)) = board.piece_on(square).zip(board.color_on(square)) {
            let (x, y) = coordinate(square.get_file(), square.get_rank());

            let piece_svg = get_piece(piece, color)
                .replace("$X_POSITION", &x.to_string())
                .replace("$Y_POSITION", &y.to_string());

            pieces.push_str(&piece_svg);
        }
    }

    get_board().replace("<!-- PIECES -->", &pieces)
}

/// Run mdbook-chess on an mdbook replacing all chess blocks with SVGs
pub fn run_preprocessor(ctx: &PreprocessorContext, mut book: Book) -> Result<Book, MdBookError> {
    // No settings so we'll skip
    book.for_each_mut(|item| {
        if let BookItem::Chapter(chapter) = item {
            let _ = process_code_blocks(chapter).map(|s| {
                chapter.content = s;
                info!("chapter '{}' processed", chapter.name);
            });
        }
    });
    Ok(book)
}

/// Generate new markdown for a chapter
fn process_code_blocks(chapter: &mut Chapter) -> Result<String, fmt::Error> {
    use CodeBlockKind::*;
    use CowStr::*;
    use Event::*;
    use Tag::{CodeBlock, Paragraph};

    let mut boards = HashMap::new();

    let mut output = String::with_capacity(chapter.content.len());
    let mut inside_block = false;
    let events = Parser::new(&chapter.content).map(|e| match (&e, inside_block) {
        (Start(CodeBlock(Fenced(Borrowed("chess")))), false) => {
            inside_block = true;
            Start(Paragraph)
        }
        (Text(Borrowed(text)), true) => {
            inside_block = false;
            Html(process_chess_block(text, &mut boards).into())
        }
        (End(CodeBlock(Fenced(Borrowed("chess")))), false) => End(Paragraph),
        _ => e,
    });

    cmark(events, &mut output).map(|_| output)
}

/// Given our c
fn process_chess_block(input: &str, boards: &mut HashMap<String, Board>) -> String {
    match serde_yaml::from_str::<BoardBlock>(input) {
        Ok(block) => {
            let name = block.load.clone();
            let mut board = match block.board.as_deref() {
                Some("start") => Board::default(),
                Some(s) => match Board::from_str(s) {
                    Ok(b) => b,
                    Err(e) => {
                        error!("Invalid FEN String: {}", e);
                        return get_board().to_string();
                    }
                },
                None => {
                    let res = name.as_ref().and_then(|name| boards.get(name)).cloned();
                    match res {
                        Some(b) => b,
                        None => Board::default(),
                    }
                }
            };

            for m in &block.moves {
                match ChessMove::from_san(&board, m.as_str()) {
                    Ok(chess_move) => {
                        let new_board = board.make_move_new(chess_move);
                        board = new_board;
                    }
                    Err(_) => {
                        error!("{} is an invalid SAN move", m);
                        return get_board().to_string();
                    }
                }
            }
            if let Some(s) = block.save.as_ref() {
                for save in s.iter() {
                    boards.insert(save.clone(), board.clone());
                }
            }
            if block.overwrite && !matches!(name.as_deref(), Some("start")) {
                if let Some(name) = name.clone() {
                    boards.insert(name, board.clone());
                }
            }
            generate_board(&board)
        }
        Err(e) => {
            error!("Creating default board invalid YAML: {}", e);
            // We got nothing, lets just pop a default board
            generate_board(&Board::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn ensure_svg_in_output() {
        let mut chapter = Chapter::new(
            "test",
            "```chess\nBoard: default\n```".to_string(),
            ".",
            vec![],
        );
        let s = process_code_blocks(&mut chapter).unwrap();
        assert!(s.contains(&generate_board(&Board::default())));
    }
}
