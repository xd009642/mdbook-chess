use chess::{Board, Color, File, Piece, Rank, Square};
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error as MdBookError;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::{fmt, fs, io};

const X_OFFSET: f32 = 0.6;
const Y_OFFSET: f32 = 0.3;
pub const PREPROCESSOR_NAME: &'static str = "mdbook-chess";

const fn get_board() -> &'static str {
    include_str!("../res/board.svg")
}

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

fn coordinate(file: File, rank: Rank) -> (f32, f32) {
    let rank = 70 - (rank.to_index() * 10);
    (
        (file.to_index() * 10) as f32 + X_OFFSET,
        rank as f32 + Y_OFFSET,
    )
}

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

pub fn save_board_from_fen(fen: &str, path: impl AsRef<Path>) -> io::Result<()> {
    let board = Board::from_str(fen)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    save_board(&board, path)
}

/// Takes
pub fn save_board(board: &Board, path: impl AsRef<Path>) -> io::Result<()> {
    let svg = generate_board(&board);
    fs::write(path, svg)
}

pub fn run_preprocessor(ctx: &PreprocessorContext, mut book: Book) -> Result<Book, MdBookError> {
    // No settings so we'll skip
    book.for_each_mut(|item| {
        if let BookItem::Chapter(chapter) = item {
            let _ = process_code_blocks(chapter).map(|s| {
                chapter.content = s;
                eprintln!("chapter '{}' processed", chapter.name);
            });
        }
    });
    Ok(book)
}

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
            Html(generate_board_svg(text, &mut boards).into())
        }
        (End(CodeBlock(Fenced(Borrowed("chess")))), false) => End(Paragraph),
        _ => e,
    });

    cmark(events, &mut output).map(|_| output)
}

fn generate_board_svg(input: &str, boards: &mut HashMap<String, Board>) -> String {
    let mut active_board = None;
    for line in input.lines() {
        if let Some(board) = line.strip_prefix("Board:").map(|x| x.trim()) {
            if board == "default" {
                return generate_board(&Board::default());
            } else {
                active_board = boards.get_mut(board);
            }
        }
    }
    // We got nothing, lets just pop an empty board there:
    get_board().to_string()
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
