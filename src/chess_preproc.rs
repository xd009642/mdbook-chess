use crate::arrows::Line;
use crate::options::{parse_square, Appearance};
use crate::render;
use anyhow::{anyhow, Context};
use chess::{Board, ChessMove};
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::errors::Error;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

pub const PREPROCESSOR_NAME: &str = "mdbook-chess";

const fn true_value() -> bool {
    true
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
    /// Whether to replace saved board with this update to it.
    #[serde(default = "true_value")]
    overwrite: bool,
    /// Squares to highlight
    #[serde(default)]
    highlights: Vec<String>,
    /// Arrows. Should be a list of string commands in the form ["a1->a2", "e2<->e4", "a3-g3"]
    #[serde(default)]
    lines: Vec<Line>,
    #[serde(default, rename = "knight-lines")]
    knight_lines: Vec<Line>,
    #[serde(flatten)]
    appearance: Appearance,
}

pub struct ChessPreprocessor;

impl Preprocessor for ChessPreprocessor {
    fn name(&self) -> &str {
        PREPROCESSOR_NAME
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let mut result = Ok(());
        book.for_each_mut(|item| {
            if result.is_ok() {
                if let BookItem::Chapter(chapter) = item {
                    result = process_code_blocks(chapter).map(|content| chapter.content = content);
                }
            }
        });
        result?;
        Ok(book)
    }
}

/// To allow for `field = "hello"` or `field = ["hello", "world"]` in YAML
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

/// Replace only fenced chess blocks; all other source bytes belong to the author.
fn process_code_blocks(chapter: &Chapter) -> Result<String, Error> {
    let mut boards = HashMap::new();
    let mut output = String::with_capacity(chapter.content.len());
    let mut copied = 0;
    let mut block_start = None;
    let mut input = String::new();
    let mut block_number = 0;
    for (event, range) in Parser::new(&chapter.content).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) if info.trim() == "chess" => {
                block_start = Some(range.start);
                input.clear();
            }
            Event::Text(text) if block_start.is_some() => input.push_str(&text),
            Event::End(TagEnd::CodeBlock) if block_start.is_some() => {
                let start = block_start.take().expect("inside chess block");
                block_number += 1;
                let svg = process_chess_block(&input, &mut boards).with_context(|| {
                    format!("chapter '{}', chess block {}", chapter.name, block_number)
                })?;
                output.push_str(&chapter.content[copied..start]);
                output.push_str(&svg);
                output.push('\n');
                copied = range.end;
            }
            _ => {}
        }
    }
    output.push_str(&chapter.content[copied..]);
    Ok(output)
}

fn process_chess_block(input: &str, boards: &mut HashMap<String, Board>) -> Result<String, Error> {
    let block: BoardBlock =
        serde_yaml::from_str(if input.trim().is_empty() { "{}" } else { input })?;
    block.appearance.validate()?;
    let mut last_move = None;
    let mut board = match block.board.as_deref() {
        Some("start") => Board::default(),
        Some(fen) => Board::from_str(fen).map_err(|e| anyhow!("invalid FEN in 'board': {e}"))?,
        None => match block.load.as_deref() {
            None | Some("start") => Board::default(),
            Some(name) => *boards
                .get(name)
                .with_context(|| format!("unknown saved board '{name}'"))?,
        },
    };
    for san in &block.moves {
        let chess_move = ChessMove::from_san(&board, san)
            .map_err(|e| anyhow!("invalid SAN move '{san}': {e}"))?;
        board = board.make_move_new(chess_move);
        last_move = Some(chess_move);
    }
    let mut highlights = block
        .highlights
        .iter()
        .map(|s| parse_square(s))
        .collect::<Result<Vec<_>, _>>()?;
    if block.appearance.last_move {
        if let Some(m) = last_move {
            highlights.extend([m.get_source(), m.get_dest()]);
        }
    }
    for line in &block.knight_lines {
        let dx = line
            .start
            .get_file()
            .to_index()
            .abs_diff(line.end.get_file().to_index());
        let dy = line
            .start
            .get_rank()
            .to_index()
            .abs_diff(line.end.get_rank().to_index());
        if !matches!((dx, dy), (1, 2) | (2, 1)) {
            anyhow::bail!("knight-lines must join squares a knight's move apart");
        }
    }
    if let Some(names) = &block.save {
        for name in names.iter() {
            boards.insert(name.clone(), board);
        }
    }
    if block.overwrite {
        if let Some(name) = block.load.as_ref().filter(|name| name.as_str() != "start") {
            boards.insert(name.clone(), board);
        }
    }
    let svg = render::board(
        &board,
        &highlights,
        &block.lines,
        &block.knight_lines,
        &block.appearance,
    );
    Ok(render::html(&svg, &block.appearance))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::Square;

    /// Tables, emphasis and other code fences must survive a nearby chess diagram byte-for-byte.
    #[test]
    fn preserves_surrounding_markdown() {
        let before =
            "| A | B |\n|---|---|\n| **bold** | _italic_ |\n\n```rust\nlet x = 1;\n```\n\n";
        let after = "\n[link](https://example.com) &amp; <em>text</em>\n";
        for fence in ["```", "~~~~"] {
            let content = format!("{before}{fence}chess\n{fence}\n{after}");
            let chapter = Chapter::new("test", content, ".", vec![]);
            let output = process_code_blocks(&chapter).expect("valid chapter");
            assert!(output.starts_with(before));
            assert!(output.ends_with(after));
            assert_eq!(output.matches("<svg").count(), 1);
        }
    }

    /// Ordinary chapters are untouched, including mdBook extensions unknown to this parser.
    #[test]
    fn dont_break_emphasis_in_tables() {
        let content = include_str!("../demo-book/src/no_chess.md");
        let chapter = Chapter::new("test", content.into(), ".", vec![]);
        assert_eq!(
            process_code_blocks(&chapter).expect("valid chapter"),
            content
        );
    }

    /// A variation must not replace the saved main line when overwrite is disabled.
    #[test]
    fn variations_preserve_saved_position() {
        let mut boards = HashMap::new();
        process_chess_block("moves: [e4]\nsave: main", &mut boards).expect("main line");
        let original = boards["main"];
        process_chess_block(
            "load: main\nmoves: [c5]\noverwrite: false\nsave: sicilian",
            &mut boards,
        )
        .expect("variation");
        assert_eq!(boards["main"], original);
        assert_eq!(
            boards["sicilian"].piece_on(Square::C5),
            Some(chess::Piece::Pawn)
        );
        process_chess_block("load: main\nmoves: [e5]", &mut boards).expect("main continuation");
        assert_eq!(
            boards["main"].piece_on(Square::E5),
            Some(chess::Piece::Pawn)
        );
    }

    /// Invalid input must fail the build with location context rather than inventing a position.
    #[test]
    fn invalid_position_reports_location() {
        let chapter = Chapter::new(
            "Endgames",
            "```chess\nload: missing\n```".into(),
            ".",
            vec![],
        );
        let error = format!(
            "{:#}",
            process_code_blocks(&chapter).expect_err("unknown saved board")
        );
        assert!(
            error.contains("Endgames") && error.contains("block 1") && error.contains("missing")
        );
    }
}
