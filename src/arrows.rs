use crate::*;
use chess::Square;
use serde::de::{self, Deserialize, Deserializer, Unexpected, Visitor};
use std::f32::consts::PI;
use std::fmt;
use std::str::FromStr;
use tracing::info;

pub const ARROW_COLOUR: &'static str = "#008a00";
pub const ARROW_OPACITY: f32 = 0.50196078;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Line {
    pub start: Square,
    pub end: Square,
    pub head: Option<ArrowHead>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Type of arrow head
pub enum ArrowHead {
    /// An arrow like `start -> end`
    Single,
    /// An arrow like `start <-> end`
    Double,
}

impl<'de> Deserialize<'de> for Line {
    fn deserialize<D>(deserializer: D) -> Result<Line, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ArrowVisitor)
    }
}

struct ArrowVisitor;

impl<'de> Visitor<'de> for ArrowVisitor {
    type Value = Line;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a string containing two squares joined by a line or arrow"
        )
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let splits: Vec<&str> = s.split("-").collect();
        if splits.len() != 2 {
            Err(de::Error::invalid_value(Unexpected::Str(s), &self))
        } else {
            if splits[0].len() < 2 || splits[0].len() > 3 {
                Err(de::Error::invalid_value(Unexpected::Str(splits[0]), &self))
            } else if splits[1].len() < 2 || splits[1].len() > 3 {
                Err(de::Error::invalid_value(Unexpected::Str(splits[1]), &self))
            } else {
                let square_1 = &splits[0][..2];

                let square_1 = match Square::from_str(square_1) {
                    Ok(s) => s,
                    Err(_) => {
                        return Err(de::Error::invalid_value(Unexpected::Str(square_1), &self));
                    }
                };

                let square_2 = match splits[1].len() {
                    2 => splits[1],
                    3 => &splits[1][1..],
                    _ => unreachable!(),
                };

                let square_2 = match Square::from_str(square_2) {
                    Ok(s) => s,
                    Err(_) => {
                        return Err(de::Error::invalid_value(Unexpected::Str(square_2), &self));
                    }
                };

                let (invert, head) = match (splits[0].chars().nth(2), splits[1].chars().nth(0)) {
                    (Some('<'), Some('>')) => (false, Some(ArrowHead::Double)),
                    (Some('<'), _) => (true, Some(ArrowHead::Single)),
                    (None, Some('>')) => (false, Some(ArrowHead::Single)),
                    (Some(t), _) | (_, Some(t)) => {
                        if splits[0].len() != 2 && splits[1].len() != 2 {
                            return Err(de::Error::invalid_value(Unexpected::Char(t), &self));
                        } else {
                            (false, None)
                        }
                    }
                    (None, None) => (false, None),
                };

                if invert {
                    Ok(Line {
                        start: square_2,
                        end: square_1,
                        head,
                    })
                } else {
                    Ok(Line {
                        start: square_1,
                        end: square_2,
                        head,
                    })
                }
            }
        }
    }
}

impl Line {
    /// Generates an SVG string for the arrow. Currently relies on marker definitions written into
    /// board.
    pub fn svg_string(&self) -> String {
        if self.start == self.end {
            String::new()
        } else {
            let (mut x1, mut y1) = coordinate_from_square(&self.start);
            let (mut x2, mut y2) = coordinate_from_square(&self.end);
            const RETREAT: f32 = 4.0;

            // Move to centre of square;
            x1 += 5.0;
            y1 += 5.0;
            x2 += 5.0;
            y2 += 5.0;

            let norm = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
            let t = RETREAT / norm;

            x1 = (1.0 - t) * x1 + t * x2;
            y1 = (1.0 - t) * y1 + t * y2;

            let t = (norm - RETREAT) / norm;
            x2 = (1.0 - t) * x1 + t * x2;
            y2 = (1.0 - t) * y1 + t * y2;

            let mut s = format!(
                r##"<line x1="{}" y1="{}" x2="{}" y2={} stroke="green" stroke-width="2.0" "##,
                x1, y1, x2, y2
            );

            match self.head.as_ref() {
                Some(ArrowHead::Single) => s.push_str(r#" marker-start="url(#startarrow)"/>"#),
                Some(ArrowHead::Double) => {
                    s.push_str(r#" marker-start="url(#startarrow)" marker-end="url(#endarrow)"/>"#)
                }
                None => s.push_str("/>"),
            }

            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestStruct {
        lines: Vec<Line>,
    }

    #[test]
    fn arrow_deser() {
        let x = serde_json::from_str::<TestStruct>(
            r#"{"lines": ["d4->g6", "c3<-c4", "g7-h7", "a1<->a8"]}"#,
        )
        .unwrap();

        let expected = vec![
            Line {
                start: Square::D4,
                end: Square::G6,
                head: Some(ArrowHead::Single),
            },
            Line {
                start: Square::C4,
                end: Square::C3,
                head: Some(ArrowHead::Single),
            },
            Line {
                start: Square::G7,
                end: Square::H7,
                head: None,
            },
            Line {
                start: Square::A1,
                end: Square::A8,
                head: Some(ArrowHead::Double),
            },
        ];

        assert_eq!(x.lines, expected);
    }

    #[test]
    fn invalid_arrows() {
        assert!(serde_json::from_str::<TestStruct>(r#"{"lines": ["d4"]}"#).is_err());
        assert!(serde_json::from_str::<TestStruct>(r#"{"lines": ["d4-->g6"]}"#).is_err());
        assert!(serde_json::from_str::<TestStruct>(r#"{"lines": ["d4->>g6"]}"#).is_err());
        assert!(serde_json::from_str::<TestStruct>(r#"{"lines": ["d4<<->g6"]}"#).is_err());
        assert!(serde_json::from_str::<TestStruct>(r#"{"lines": ["d4->z6"]}"#).is_err());
        assert!(serde_json::from_str::<TestStruct>(r#"{"lines": ["d4->d20"]}"#).is_err());
        assert!(serde_json::from_str::<TestStruct>(r#"{"lines": ["d4z6"]}"#).is_err());
        assert!(serde_json::from_str::<TestStruct>(r#"{"lines": ["d4-7z6"]}"#).is_err());
        assert!(serde_json::from_str::<TestStruct>(r#"{"lines": ["d4=>z6"]}"#).is_err());
    }
}
