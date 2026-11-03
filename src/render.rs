use crate::arrows::{ArrowHead, Line};
use crate::options::{parse_square, Align, Appearance, Orientation};
use chess::{Board, Color, Piece, Square, ALL_SQUARES};
use svg::node::element::{Circle, Element, Group, Polygon, Polyline, Rectangle, Text, Title};
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

pub(crate) fn coordinate(square: Square, orientation: &Orientation) -> (f32, f32) {
    let file = square.get_file().to_index() as f32;
    let rank = square.get_rank().to_index() as f32;
    match orientation {
        Orientation::White => (file * 10.0, (7.0 - rank) * 10.0),
        Orientation::Black => ((7.0 - file) * 10.0, rank * 10.0),
    }
}

pub(crate) fn board(
    board: &Board,
    highlights: &[Square],
    lines: &[Line],
    knight_lines: &[Line],
    options: &Appearance,
) -> String {
    let (light_color, dark_color, highlight_color, arrow_color) = options.palette();
    let title = options.alt.clone().unwrap_or_else(|| {
        format!(
            "{} to move. Position: {}",
            if board.side_to_move() == Color::White {
                "White"
            } else {
                "Black"
            },
            board
        )
    });
    let (origin, extent) = if options.coordinates {
        (-5, 90)
    } else {
        (0, 80)
    };
    let mut document = Document::new()
        .set("viewBox", (origin, origin, extent, extent))
        .set("width", options.size)
        .set("height", options.size)
        .set("role", "img")
        .set("aria-label", title.clone())
        .set("style", "max-width:100%;height:auto")
        .add(Title::new(title));
    for square in ALL_SQUARES {
        let (x, y) = coordinate(square, &options.orientation);
        let light = (square.get_file().to_index() + square.get_rank().to_index()) % 2 != 0;
        document.append(
            Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", 10)
                .set("height", 10)
                .set("fill", if light { light_color } else { dark_color }),
        );
    }
    for &square in highlights {
        let (x, y) = coordinate(square, &options.orientation);
        document.append(
            Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", 10)
                .set("height", 10)
                .set("fill", highlight_color)
                .set("fill-opacity", 0.5),
        );
    }
    for square in ALL_SQUARES {
        if let Some((p, c)) = board.piece_on(square).zip(board.color_on(square)) {
            let (x, y) = coordinate(square, &options.orientation);
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
        document.append(arrow(line, false, options, arrow_color));
    }
    for line in knight_lines {
        document.append(arrow(line, true, options, arrow_color));
    }
    if options.coordinates {
        document.append(
            Rectangle::new()
                .set("x", -5)
                .set("y", -5)
                .set("width", 90)
                .set("height", 5)
                .set("fill", "#ffffff"),
        );
        document.append(
            Rectangle::new()
                .set("x", -5)
                .set("y", 80)
                .set("width", 90)
                .set("height", 5)
                .set("fill", "#ffffff"),
        );
        document.append(
            Rectangle::new()
                .set("x", -5)
                .set("y", 0)
                .set("width", 5)
                .set("height", 80)
                .set("fill", "#ffffff"),
        );
        document.append(
            Rectangle::new()
                .set("x", 80)
                .set("y", 0)
                .set("width", 5)
                .set("height", 80)
                .set("fill", "#ffffff"),
        );
        for i in 0..8 {
            let n = match options.orientation {
                Orientation::White => i,
                Orientation::Black => 7 - i,
            };
            for y in [-1.5, 83.5] {
                document.append(label(
                    char::from(b'a' + n).to_string(),
                    i as f32 * 10.0 + 5.0,
                    y,
                ));
            }
            for x in [-2.5, 82.5] {
                document.append(label((8 - n).to_string(), x, i as f32 * 10.0 + 6.0));
            }
        }
    }
    for (square, annotation) in &options.annotations {
        let (x, y) = coordinate(
            parse_square(square).expect("validated annotation square"),
            &options.orientation,
        );
        document.append(
            Circle::new()
                .set("cx", x + 8.0)
                .set("cy", y + 2.0)
                .set("r", 2.3)
                .set("fill", annotation.color()),
        );
        document.append(
            label(annotation.symbol().into(), x + 8.0, y + 3.0)
                .set("fill", "white")
                .set("font-weight", "bold"),
        );
    }
    // A single line keeps a Markdown HTML block intact, including inside lists and quotes.
    document.to_string().replace(['\n', '\r'], " ")
}

fn arrow(line: &Line, knight: bool, options: &Appearance, color: &str) -> Group {
    let mut group = Group::new().set("fill", color).set("stroke", color);
    if line.start == line.end {
        return group;
    }
    let (x1, y1) = coordinate(line.start, &options.orientation);
    let (x2, y2) = coordinate(line.end, &options.orientation);
    let start = (x1 + 5.0, y1 + 5.0);
    let end = (x2 + 5.0, y2 + 5.0);
    let bend = if (end.0 - start.0).abs() > (end.1 - start.1).abs() {
        (end.0, start.1)
    } else {
        (start.0, end.1)
    };
    let points = if knight {
        vec![start, bend, end]
    } else {
        vec![start, end]
    };
    group.append(
        Polyline::new()
            .set(
                "points",
                points
                    .iter()
                    .map(|(x, y)| format!("{x},{y}"))
                    .collect::<Vec<_>>()
                    .join(" "),
            )
            .set("fill", "none")
            .set("stroke-width", 1.5)
            .set("stroke-linejoin", "round"),
    );
    // Explicit arrow heads avoid document-global marker IDs when many boards share a page.
    if line.head.is_some() {
        group.append(arrow_head(if knight { bend } else { start }, end));
    }
    if line.head == Some(ArrowHead::Double) {
        group.append(arrow_head(if knight { bend } else { end }, start));
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

fn label(text: String, x: f32, y: f32) -> Text {
    Text::new(text)
        .set("x", x)
        .set("y", y)
        .set("font-size", 3)
        .set("font-family", "sans-serif")
        .set("text-anchor", "middle")
        .set("fill", "#222222")
}

pub(crate) fn html(svg: &str, options: &Appearance) -> String {
    let margin = match options.align {
        Align::Left => "0 auto 0 0",
        Align::Center => "0 auto",
        Align::Right => "0 0 0 auto",
    };
    let mut wrapper = Element::new("div");
    wrapper.assign("class", "chess-board");
    wrapper.assign(
        "style",
        format!("width:{}px;max-width:100%;margin:{margin}", options.size),
    );
    wrapper.append(Blob::new(svg));
    if let Some(caption) = &options.caption {
        let mut paragraph = Element::new("p");
        paragraph.assign("class", "chess-caption");
        paragraph.append(svg::node::Text::new(caption));
        wrapper.append(paragraph);
    }
    wrapper.to_string().replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flipping changes square placement, never the identity or upright shape of a piece.
    #[test]
    fn orientation_maps_every_square() {
        for square in ALL_SQUARES {
            let white = coordinate(square, &Orientation::White);
            let black = coordinate(square, &Orientation::Black);
            assert_eq!((white.0 + black.0, white.1 + black.1), (70.0, 70.0));
        }
        assert_eq!(coordinate(Square::A1, &Orientation::White), (0.0, 70.0));
        assert_eq!(coordinate(Square::H8, &Orientation::Black), (0.0, 70.0));
    }

    /// Captions and accessible descriptions are text, including when they contain markup.
    #[test]
    fn escapes_author_text() {
        let options = Appearance {
            caption: Some("<script>& caption".into()),
            alt: Some("A < B & C".into()),
            ..Appearance::default()
        };
        let svg = board(&Board::default(), &[], &[], &[], &options);
        let doc = roxmltree::Document::parse(&svg).expect("valid SVG");
        assert_eq!(
            doc.root_element().attribute("aria-label"),
            options.alt.as_deref()
        );
        let markup = html(&svg, &options);
        assert!(!markup.contains("<script>"));
        assert!(markup.contains("&lt;script&gt;&amp; caption"));
    }

    /// A knight arrow has perpendicular legs; both orientations retain its endpoints.
    #[test]
    fn knight_arrow_geometry() {
        let line = Line {
            start: Square::G1,
            end: Square::F3,
            head: Some(ArrowHead::Single),
        };
        for orientation in [Orientation::White, Orientation::Black] {
            let options = Appearance {
                orientation,
                ..Appearance::default()
            };
            let svg = board(
                &Board::default(),
                &[],
                &[],
                std::slice::from_ref(&line),
                &options,
            );
            let doc = roxmltree::Document::parse(&svg).expect("valid SVG");
            let path = doc
                .descendants()
                .find(|n| n.has_tag_name("polyline"))
                .expect("arrow shaft");
            let points: Vec<Vec<f32>> = path
                .attribute("points")
                .expect("points")
                .split_whitespace()
                .map(|point| {
                    point
                        .split(',')
                        .map(|n| n.parse().expect("coordinate"))
                        .collect()
                })
                .collect();
            assert_eq!(points.len(), 3);
            assert_eq!(points[0][0], points[1][0]);
            assert_eq!(points[1][1], points[2][1]);
            let (x, y) = coordinate(Square::F3, &options.orientation);
            assert_eq!(points[2], [x + 5.0, y + 5.0]);
        }
    }

    /// Boards must be valid standalone SVG without IDs that collide on a multi-board page.
    #[test]
    fn standalone_svg() {
        let line = Line {
            start: Square::E2,
            end: Square::E4,
            head: Some(ArrowHead::Double),
        };
        let svg = board(
            &Board::default(),
            &[Square::E4],
            &[line],
            &[],
            &Appearance::default(),
        );
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
