use crate::arrows::{ArrowHead, Line};
use chess::{Board, Color, Piece, Square, ALL_SQUARES};
use svg::node::element::{Group, Polygon, Polyline, Rectangle, Title};
use svg::node::Blob;
use svg::{Document, Node};

fn piece(p: Piece, c: Color) -> &'static str {
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

pub(crate) fn coordinate(square: Square) -> (f32, f32) {
    (
        square.get_file().to_index() as f32 * 10.0,
        (7 - square.get_rank().to_index()) as f32 * 10.0,
    )
}

pub(crate) fn board(board: &Board, highlights: &[Square], lines: &[Line]) -> String {
    let mut document = Document::new()
        .set("viewBox", (0, 0, 80, 80))
        .set("width", 302)
        .set("height", 302)
        .set("role", "img")
        .set("aria-label", "Chess position")
        .add(Title::new("Chess position"));
    for square in ALL_SQUARES {
        let (x, y) = coordinate(square);
        let light = (square.get_file().to_index() + square.get_rank().to_index()) % 2 != 0;
        document.append(
            Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", 10)
                .set("height", 10)
                .set("fill", if light { "#ffce9e" } else { "#d18b47" }),
        );
    }
    for &square in highlights {
        let (x, y) = coordinate(square);
        document.append(
            Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", 10)
                .set("height", 10)
                .set("fill", "#ff0000")
                .set("fill-opacity", 0.5),
        );
    }
    for square in ALL_SQUARES {
        if let Some((p, c)) = board.piece_on(square).zip(board.color_on(square)) {
            let (x, y) = coordinate(square);
            // The bundled artwork uses a 45-unit canvas. Only this outer group positions it.
            document.append(
                Group::new()
                    .set(
                        "transform",
                        format!("translate({},{}) scale(0.2)", x + 0.5, y + 0.5),
                    )
                    .add(Blob::new(piece(p, c))),
            );
        }
    }
    for line in lines {
        document.append(arrow(line));
    }
    // A single line keeps a Markdown HTML block intact, including inside lists and quotes.
    document.to_string().replace(['\n', '\r'], " ")
}

fn arrow(line: &Line) -> Group {
    let mut group = Group::new().set("fill", "#008a00").set("stroke", "#008a00");
    if line.start == line.end {
        return group;
    }
    let (x1, y1) = coordinate(line.start);
    let (x2, y2) = coordinate(line.end);
    let start = (x1 + 5.0, y1 + 5.0);
    let end = (x2 + 5.0, y2 + 5.0);
    group.append(
        Polyline::new()
            .set(
                "points",
                format!("{},{} {},{}", start.0, start.1, end.0, end.1),
            )
            .set("fill", "none")
            .set("stroke-width", 1.5),
    );
    // Explicit arrow heads avoid document-global marker IDs when many boards share a page.
    if line.head.is_some() {
        group.append(arrow_head(start, end));
    }
    if line.head == Some(ArrowHead::Double) {
        group.append(arrow_head(end, start));
    }
    group
}

fn arrow_head(from: (f32, f32), to: (f32, f32)) -> Polygon {
    let length = (to.0 - from.0).hypot(to.1 - from.1);
    let (dx, dy) = ((to.0 - from.0) / length, (to.1 - from.1) / length);
    let base = (to.0 - dx * 3.0, to.1 - dy * 3.0);
    Polygon::new().set("stroke", "none").set(
        "points",
        format!(
            "{},{} {},{} {},{}",
            to.0,
            to.1,
            base.0 - dy * 1.8,
            base.1 + dx * 1.8,
            base.0 + dy * 1.8,
            base.1 - dx * 1.8
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Boards must be valid standalone SVG without IDs that collide on a multi-board page.
    #[test]
    fn standalone_svg() {
        let line = Line {
            start: Square::E2,
            end: Square::E4,
            head: Some(ArrowHead::Double),
        };
        let svg = board(&Board::default(), &[Square::E4], &[line]);
        let doc = roxmltree::Document::parse(&svg).expect("valid SVG");
        assert_eq!(
            doc.root_element().tag_name().namespace(),
            Some("http://www.w3.org/2000/svg")
        );
        assert!(doc.descendants().all(|n| n.attribute("id").is_none()));
        assert_eq!(
            doc.descendants().filter(|n| n.has_tag_name("rect")).count(),
            65
        );
        assert_eq!(
            doc.descendants()
                .filter(|n| n.has_tag_name("polygon"))
                .count(),
            2
        );
    }
}
