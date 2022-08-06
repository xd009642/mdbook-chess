use mdbook_chess::save_board;

fn main() {
    save_board(&Default::default(), "output.svg").unwrap();
}
