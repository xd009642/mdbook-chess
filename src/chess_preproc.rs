use crate::animation::{Animation, Frame};
use crate::arrows::Line;
use crate::options::{parse_square, Appearance};
use crate::output::{Diagram, Exporter, Output};
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
    output: Option<Output>,
    animation: Option<Animation>,
}

impl Default for BoardBlock {
    fn default() -> Self {
        Self {
            load: None,
            save: None,
            board: None,
            moves: vec![],
            overwrite: true,
            highlights: vec![],
            lines: vec![],
            knight_lines: vec![],
            appearance: Appearance::default(),
            output: None,
            animation: None,
        }
    }
}

pub struct ChessPreprocessor;

impl Preprocessor for ChessPreprocessor {
    fn name(&self) -> &str {
        PREPROCESSOR_NAME
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let format = ctx
            .config
            .get::<Output>("preprocessor.chess.output")?
            .unwrap_or_default();
        let mut exporter =
            Exporter::new(ctx.root.join(&ctx.config.book.src), &ctx.renderer, format);
        let mut result = Ok(());
        book.for_each_mut(|item| {
            if result.is_ok() {
                if let BookItem::Chapter(chapter) = item {
                    result = process_code_blocks(chapter, &mut exporter)
                        .map(|content| chapter.content = content);
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
fn process_code_blocks(chapter: &Chapter, exporter: &mut Exporter) -> Result<String, Error> {
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
                let svg = process_chess_block(&input, &mut boards)
                    .and_then(|diagram| exporter.render(&diagram, chapter.path.as_deref()))
                    .with_context(|| {
                        format!("chapter '{}', chess block {}", chapter.name, block_number)
                    })?;
                output.push_str(&chapter.content[copied..start]);
                output.push_str(&svg);
                output.push('\n');
                // HTML needs a blank separator. Retain quote markers and list indentation
                // on that separator so it does not terminate the enclosing container.
                let line_start = chapter.content[..start].rfind('\n').map_or(0, |i| i + 1);
                for ch in chapter.content[line_start..start].chars() {
                    output.push(if ch == '>' || ch.is_whitespace() {
                        ch
                    } else {
                        ' '
                    });
                }
                copied = range.end;
            }
            _ => {}
        }
    }
    output.push_str(&chapter.content[copied..]);
    Ok(output)
}

fn process_chess_block(input: &str, boards: &mut HashMap<String, Board>) -> Result<Diagram, Error> {
    let mut block = serde_yaml::from_str::<Option<BoardBlock>>(input)?.unwrap_or_default();
    block.appearance.validate()?;
    if let Some(animation) = &block.animation {
        animation.validate(block.moves.len())?;
    }
    let initial = match block.board.as_deref() {
        Some("start") => Board::default(),
        Some(fen) => Board::from_str(fen).map_err(|e| anyhow!("invalid FEN in 'board': {e}"))?,
        None => match block.load.as_deref() {
            None | Some("start") => Board::default(),
            Some(name) => *boards
                .get(name)
                .with_context(|| format!("unknown saved board '{name}'"))?,
        },
    };
    let positions = play_moves(initial, &block.moves)?;
    let (board, last_move) = *positions
        .last()
        .expect("sequence includes initial position");
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
    let mut frames = Vec::new();
    if block.animation.is_some() {
        for (i, (position, last_move)) in positions.iter().enumerate() {
            let final_frame = i == positions.len() - 1;
            let mut appearance = block.appearance.clone();
            let mut frame_highlights = if final_frame {
                block
                    .highlights
                    .iter()
                    .map(|s| parse_square(s))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                vec![]
            };
            if !final_frame {
                appearance.annotations.clear();
            }
            if appearance.last_move {
                if let Some(m) = last_move {
                    frame_highlights.extend([m.get_source(), m.get_dest()]);
                }
            }
            frames.push(Frame {
                svg: render::board(
                    position,
                    &frame_highlights,
                    if final_frame { &block.lines } else { &[] },
                    if final_frame {
                        &block.knight_lines
                    } else {
                        &[]
                    },
                    &appearance,
                ),
                label: if i == 0 {
                    "Initial position".into()
                } else {
                    format!("{} / {} · {}", i, block.moves.len(), block.moves[i - 1])
                },
            });
        }
    }
    block
        .appearance
        .alt
        .get_or_insert_with(|| render::description(&board));
    let svg = render::board(
        &board,
        &highlights,
        &block.lines,
        &block.knight_lines,
        &block.appearance,
    );
    Ok(Diagram {
        svg,
        appearance: block.appearance,
        output: block.output,
        animation: block.animation,
        frames,
    })
}

/// Every replay frame is a legal position, including captures, castling and promotions.
fn play_moves(initial: Board, moves: &[String]) -> Result<Vec<(Board, Option<ChessMove>)>, Error> {
    let mut board = initial;
    let mut positions = vec![(board, None)];
    for san in moves {
        // chess 3.2 requires " e.p." although PGN 8.2.3.3 uses ordinary capture SAN.
        // Normalize only pawn captures onto this position's en-passant target;
        // the dependency still generates and checks the legal move.
        let basic = san
            .strip_suffix('+')
            .or_else(|| san.strip_suffix('#'))
            .unwrap_or(san);
        let ep_target = board.en_passant().map(|pawn| {
            pawn.forward(board.side_to_move())
                .expect("en-passant pawn is on rank 4 or 5")
        });
        let is_ep = ep_target.is_some_and(|target| {
            basic.len() == 4
                && (b'a'..=b'h').contains(&basic.as_bytes()[0])
                && basic.as_bytes()[1] == b'x'
                && basic.get(2..) == Some(target.to_string().as_str())
        });
        let mut notation = if is_ep {
            format!("{san} e.p.")
        } else {
            san.clone()
        };
        // The same parser expects a8Q rather than PGN's a8=Q (8.2.3.8).
        if let Some((prefix, suffix)) = notation.rsplit_once('=') {
            if matches!(suffix.trim_end_matches(['+', '#']), "Q" | "R" | "B" | "N") {
                notation = format!("{prefix}{suffix}");
            }
        }
        let chess_move = ChessMove::from_san(&board, &notation)
            .map_err(|e| anyhow!("invalid SAN move '{san}': {e}"))?;
        board = board.make_move_new(chess_move);
        positions.push((board, Some(chess_move)));
    }
    Ok(positions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::Square;

    /// The README's empty/comment-only fences produce the same starting position as board: start.
    #[test]
    fn empty_and_comment_only_blocks() {
        for yaml in ["", "# Starting position\n"] {
            let diagram = process_chess_block(yaml, &mut HashMap::new()).expect("empty board");
            assert!(diagram
                .appearance
                .alt
                .expect("description")
                .contains(&Board::default().to_string()));
        }
    }

    /// Replay includes the initial state and each legal result; special moves alter all affected squares.
    #[test]
    fn replay_special_moves() {
        let cases = [
            (
                "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
                "O-O",
                Square::G1,
                chess::Piece::King,
                Square::H1,
            ),
            (
                "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
                "exd6",
                Square::D6,
                chess::Piece::Pawn,
                Square::D5,
            ),
            (
                "4k3/P7/8/8/8/8/8/4K3 w - - 0 1",
                "a8=Q+",
                Square::A8,
                chess::Piece::Queen,
                Square::A7,
            ),
        ];
        for (fen, san, destination, piece, vacated) in cases {
            let initial = Board::from_str(fen).expect("legal fixture");
            let positions = play_moves(initial, &[san.into()]).expect("legal move");
            assert_eq!(positions.len(), 2);
            assert_eq!(positions[0].0, initial);
            assert_eq!(positions[1].0.piece_on(destination), Some(piece));
            assert_eq!(positions[1].0.piece_on(vacated), None);
            if san == "O-O" {
                assert_eq!(
                    positions[1].0.piece_on(Square::F1),
                    Some(chess::Piece::Rook)
                );
            }
        }
    }

    /// Inserting the HTML separator must not split a quoted diagram from its explanation.
    #[test]
    fn quoted_diagram_keeps_its_explanation() {
        let chapter = Chapter::new(
            "test",
            "> ```chess\n> ```\n> **explanation**\n".into(),
            ".",
            vec![],
        );
        let content = process_code_blocks(
            &chapter,
            &mut Exporter::new(".".into(), "html", Output::Auto),
        )
        .expect("quoted diagram");
        let events: Vec<_> = Parser::new(&content).collect();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Start(Tag::BlockQuote(_))))
                .count(),
            1
        );
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Strong))));
    }

    /// En-passant normalization must still reject a capture that exposes the player's king.
    #[test]
    fn en_passant_cannot_expose_check() {
        let initial =
            Board::from_str("k3r3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").expect("legal position");
        assert!(play_moves(initial, &["exd6".into()]).is_err());
    }

    /// Tables, emphasis and other code fences must survive a nearby chess diagram byte-for-byte.
    #[test]
    fn preserves_surrounding_markdown() {
        let before =
            "| A | B |\n|---|---|\n| **bold** | _italic_ |\n\n```rust\nlet x = 1;\n```\n\n";
        let after = "\n[link](https://example.com) &amp; <em>text</em>\n";
        for fence in ["```", "~~~~"] {
            let content = format!("{before}{fence}chess\n{fence}\n{after}");
            let chapter = Chapter::new("test", content, ".", vec![]);
            let output = process_code_blocks(
                &chapter,
                &mut Exporter::new(".".into(), "html", Output::Auto),
            )
            .expect("valid chapter");
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
            process_code_blocks(
                &chapter,
                &mut Exporter::new(".".into(), "html", Output::Auto)
            )
            .expect("valid chapter"),
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
            process_code_blocks(
                &chapter,
                &mut Exporter::new(".".into(), "html", Output::Auto)
            )
            .expect_err("unknown saved board")
        );
        assert!(
            error.contains("Endgames") && error.contains("block 1") && error.contains("missing")
        );
    }
}
