use mdbook_chess::*;

fn main() {
    save_board(&Default::default(), "output.svg").unwrap();
    save_board_from_fen(
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        "position.svg",
    )
    .unwrap();
}
