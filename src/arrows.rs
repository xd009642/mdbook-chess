use chess::Square;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Line {
    pub start: Square,
    pub end: Square,
    pub head: Option<ArrowHead>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ArrowHead {
    Single,
    Double,
}

impl<'de> Deserialize<'de> for Line {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        for (separator, head, reverse) in [
            ("<->", Some(ArrowHead::Double), false),
            ("->", Some(ArrowHead::Single), false),
            ("<-", Some(ArrowHead::Single), true),
            ("-", None, false),
        ] {
            if let Some((a, b)) = value.split_once(separator) {
                let parse = |s: &str| {
                    if s.len() != 2 || !s.is_ascii() {
                        return Err(serde::de::Error::custom("expected a square from a1 to h8"));
                    }
                    Square::from_str(s).map_err(serde::de::Error::custom)
                };
                let (start, end) = (parse(a)?, parse(b)?);
                return Ok(if reverse {
                    Self {
                        start: end,
                        end: start,
                        head,
                    }
                } else {
                    Self { start, end, head }
                });
            }
        }
        Err(serde::de::Error::custom(
            "expected a1-b2, a1->b2, a1<-b2 or a1<->b2",
        ))
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
        .expect("valid arrow fixture");

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
