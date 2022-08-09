use crate::*;
use chess::Square;
use std::f32::consts::PI;

pub const ARROW_COLOUR: &'static str = "#008a00";
pub const ARROW_OPACITY: f32 = 0.50196078;

pub struct Line {
    start: Square,
    end: Square,
    head: Option<ArrowHead>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ArrowHead {
    Single,
    Double,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum AnchorPoints {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl AnchorPoints {
    fn offset(&self) -> (f32, f32) {
        match *self {
            Self::Top => (5.0, 1.0),
            Self::Bottom => (5.0, 9.0),
            Self::Left => (1.0, 5.0),
            Self::Right => (9.0, 5.0),
            Self::TopLeft => (1.0, 1.0),
            Self::TopRight => (9.0, 1.0),
            Self::BottomLeft => (9.0, 1.0),
            Self::BottomRight => (1.0, 9.0),
        }
    }
}

impl Line {
    fn workout_anchors(&self) -> (AnchorPoints, AnchorPoints) {
        let start_rank = self.start.get_rank().to_index() as isize;
        let end_rank = self.end.get_rank().to_index() as isize;
        let start_file = self.start.get_file().to_index() as isize;
        let end_file = self.end.get_file().to_index() as isize;
        if start_file == end_file {
            if end_rank > start_rank {
                (AnchorPoints::Right, AnchorPoints::Left)
            } else {
                (AnchorPoints::Left, AnchorPoints::Right)
            }
        } else if start_rank == end_rank {
            if end_file > start_file {
                (AnchorPoints::Top, AnchorPoints::Bottom)
            } else {
                (AnchorPoints::Bottom, AnchorPoints::Top)
            }
        } else {
            let dx = (end_rank - start_rank) as f32;
            let dy = (end_file - start_file) as f32;
            let angle = dy.atan2(dx) * (180.0 / PI);

            todo!()
        }
    }

    pub fn svg_string(&self) -> String {
        if self.start == self.end {
            String::new()
        } else {
            let (mut x1, mut y1) = coordinate_from_square(&self.start);
            let (mut x2, mut y2) = coordinate_from_square(&self.end);

            let (a1, a2) = self.workout_anchors();
            let a1 = a1.offset();
            let a2 = a2.offset();
            x1 += a1.0;
            y1 += a1.1;
            x2 += a2.0;
            y2 += a2.1;

            let mut s = format!(
                r##"<line x1="{}" y1="{}" x2="{}" y2={} stroke="orange" sroke-opacity="0.6" stroke-width="0.3" "##,
                x1, y1, x2, y2
            );

            match self.head.as_ref() {
                Some(ArrowHead::Single) => s.push_str(r#" marker-start="url(#startarrow)"/>"#),
                Some(ArrowHead::Double) => {
                    s.push_str(r#" marker-start="url(#startarrow)" marker-end="url(#endarrow)/>"#)
                }
                None => s.push_str("/>"),
            }

            s
        }
    }
}
