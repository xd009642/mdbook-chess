use chess::{Board, Color, File, Piece, Rank, Square};
use std::path::Path;
use std::str::FromStr;
use std::{fs, io};

const X_OFFSET: f32 = -0.6;
const Y_OFFSET: f32 = 0.0;

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
            pieces.push('\n');
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
