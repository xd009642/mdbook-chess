use crate::animation::{Animation, Format, Frame};
use crate::{options::Appearance, render};
use anyhow::{Context, Result};
use resvg::{tiny_skia, usvg};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use svg::node::{element::Element, Blob};
use svg::Node;

const ASSET_DIR: &str = "mdbook-chess-assets";

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Output {
    #[default]
    Auto,
    InlineSvg,
    Svg,
    Png,
}

#[derive(Debug)]
pub struct Diagram {
    pub svg: String,
    pub appearance: Appearance,
    pub output: Option<Output>,
    pub animation: Option<Animation>,
    pub frames: Vec<Frame>,
}

pub struct Exporter {
    source_dir: PathBuf,
    format: Output,
    html: bool,
    svg_options: Option<usvg::Options<'static>>,
}

impl Exporter {
    pub fn new(source_dir: PathBuf, renderer: &str, format: Output) -> Self {
        Self {
            source_dir,
            format,
            html: renderer == "html",
            svg_options: None,
        }
    }

    pub fn render(&mut self, diagram: &Diagram, chapter: Option<&Path>) -> Result<String> {
        if self.html {
            if let Some(animation) = &diagram.animation {
                return self.animation(diagram, animation, chapter);
            }
        }
        let format = match diagram.output.unwrap_or(self.format) {
            Output::Auto if self.html => Output::InlineSvg,
            Output::Auto => Output::Png,
            other => other,
        };
        let (bytes, extension) = match format {
            Output::InlineSvg => return Ok(render::html(&diagram.svg, &diagram.appearance)),
            Output::Svg => (diagram.svg.as_bytes().to_vec(), "svg"),
            Output::Png => (self.rasterize(&diagram.svg)?.encode_png()?, "png"),
            Output::Auto => unreachable!("auto output is resolved above"),
        };
        let url = self.write_asset(&bytes, extension, chapter)?;
        let alt = diagram
            .appearance
            .alt
            .as_deref()
            .unwrap_or("Chess position");
        let title = diagram
            .appearance
            .caption
            .as_ref()
            .map(|caption| format!(" \"{}\"", markdown_text(caption)))
            .unwrap_or_default();
        Ok(format!("![{}]({url}{title})", markdown_text(alt)))
    }

    fn animation(
        &mut self,
        diagram: &Diagram,
        animation: &Animation,
        chapter: Option<&Path>,
    ) -> Result<String> {
        let script = self.write_asset(include_bytes!("../res/replay.js"), "js", chapter)?;
        let style = self.write_asset(include_bytes!("../res/replay.css"), "css", chapter)?;
        let mut container = Element::new("div");
        container.assign(
            "class",
            if animation.format == Format::Player {
                "chess-replay"
            } else {
                "chess-gif"
            },
        );
        container.assign("data-delay", animation.delay);
        container.assign("data-repeat", animation.repeat.to_string());
        if animation.format == Format::Player {
            for (i, frame) in diagram.frames.iter().enumerate() {
                let mut div = Element::new("div");
                div.assign(
                    "class",
                    if i + 1 == diagram.frames.len() {
                        "chess-frame chess-final"
                    } else {
                        "chess-frame"
                    },
                );
                div.assign("data-label", frame.label.as_str());
                if i + 1 != diagram.frames.len() {
                    div.assign("hidden", "hidden");
                }
                div.append(Blob::new(&frame.svg));
                container.append(div);
            }
            container.append(Blob::new(r#"<div class="chess-controls" hidden><div class="chess-buttons"><button type="button" data-action="previous" aria-label="Previous position">Previous</button><button type="button" data-action="play" aria-label="Play moves">Play</button><button type="button" data-action="next" aria-label="Next position">Next</button></div><input type="range" min="0" value="0" step="1" aria-label="Move position"><output aria-live="polite"></output></div>"#));
        } else {
            let pixels = diagram
                .frames
                .iter()
                .map(|frame| self.rasterize(&frame.svg));
            let bytes = crate::animation::gif_frames(pixels, animation)?;
            let gif_url = self.write_asset(&bytes, "gif", chapter)?;
            let poster = self.rasterize(&diagram.svg)?.encode_png()?;
            let poster_url = self.write_asset(&poster, "png", chapter)?;
            let alt = diagram
                .appearance
                .alt
                .as_deref()
                .unwrap_or("Chess move sequence");
            container.append(Blob::new(format!("<img class=\"chess-poster\" src=\"{poster_url}\" alt=\"{}\"><img class=\"chess-animation\" data-src=\"{gif_url}\" alt=\"{}\" hidden><div class=\"chess-controls\" hidden><button type=\"button\" data-action=\"gif\">Play GIF</button></div><a href=\"{gif_url}\" download>Download GIF</a>", html_attribute(alt), html_attribute(alt))));
        }
        let markup = format!("<link rel=\"stylesheet\" href=\"{style}\">{container}<script src=\"{script}\" defer></script>");
        Ok(render::html(&markup, &diagram.appearance))
    }

    pub fn rasterize(&mut self, svg: &str) -> Result<tiny_skia::Pixmap> {
        let options = self.svg_options.get_or_insert_with(|| {
            let mut options = usvg::Options::default();
            options.fontdb_mut().load_system_fonts();
            options
        });
        let tree = usvg::Tree::from_str(svg, options).context("parsing composed SVG")?;
        let size = tree.size().to_int_size();
        let mut pixels = tiny_skia::Pixmap::new(size.width(), size.height())
            .context("allocating board image")?;
        pixels.fill(tiny_skia::Color::WHITE);
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixels.as_mut());
        Ok(pixels)
    }

    fn write_asset(&self, bytes: &[u8], extension: &str, chapter: Option<&Path>) -> Result<String> {
        let name = format!("{:x}.{extension}", Sha256::digest(bytes));
        let directory = self.source_dir.join(ASSET_DIR);
        fs::create_dir_all(&directory)
            .with_context(|| format!("creating {}", directory.display()))?;
        let path = directory.join(&name);
        // Content-addressed assets are immutable. Reusing them also avoids mdbook serve rebuild loops.
        if !path.exists() {
            fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
        }
        let depth = chapter
            .and_then(Path::parent)
            .map(|p| {
                p.components()
                    .filter(|c| matches!(c, std::path::Component::Normal(_)))
                    .count()
            })
            .unwrap_or(0);
        Ok(format!("{}{ASSET_DIR}/{name}", "../".repeat(depth)))
    }
}

fn markdown_text(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        if ch.is_ascii_punctuation() {
            escaped.push('\\');
        }
        escaped.push(if ch == '\n' || ch == '\r' { ' ' } else { ch });
    }
    escaped
}

fn html_attribute(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::Board;
    use pulldown_cmark::{Event, Parser, Tag};

    /// GIFs retain every chess position and timing; print backends receive only the final PNG.
    #[test]
    fn animation_export_and_static_fallback() {
        for (size, delay, repeat) in [(96, 100, false), (160, 1200, true)] {
            let initial = Board::default();
            let final_board = initial
                .make_move_new(chess::ChessMove::from_san(&initial, "e4").expect("legal move"));
            let appearance = Appearance {
                size,
                ..Appearance::default()
            };
            let first = render::board(&initial, &[], &[], &[], &appearance);
            let last = render::board(&final_board, &[], &[], &[], &appearance);
            let diagram = Diagram {
                svg: last.clone(),
                appearance,
                output: None,
                animation: Some(Animation {
                    format: Format::Gif,
                    delay,
                    repeat,
                }),
                frames: vec![
                    Frame {
                        svg: first,
                        label: "Initial".into(),
                    },
                    Frame {
                        svg: last,
                        label: "e4".into(),
                    },
                ],
            };
            let directory = tempfile::tempdir().expect("temporary book");
            let mut exporter = Exporter::new(directory.path().into(), "html", Output::Auto);
            let markup = exporter.render(&diagram, None).expect("GIF export");
            assert!(markup.contains("Download GIF"));
            let path = fs::read_dir(directory.path().join(ASSET_DIR))
                .expect("assets")
                .map(|entry| entry.expect("asset").path())
                .find(|path| path.extension().is_some_and(|e| e == "gif"))
                .expect("GIF file");
            let mut options = gif::DecodeOptions::new();
            options.set_color_output(gif::ColorOutput::RGBA);
            let mut decoder = options
                .read_info(fs::File::open(path).expect("GIF file"))
                .expect("valid GIF");
            assert_eq!((decoder.width(), decoder.height()), (size, size));
            let mut decoded = Vec::new();
            while let Some(frame) = decoder.read_next_frame().expect("valid GIF frame") {
                assert_eq!(frame.delay, delay / 10);
                decoded.push(frame.buffer.to_vec());
            }
            assert_eq!(decoded.len(), 2);
            assert_ne!(decoded[0], decoded[1]);
            assert_eq!(
                decoder.repeat(),
                if repeat {
                    gif::Repeat::Infinite
                } else {
                    gif::Repeat::Finite(0)
                }
            );
            let print_dir = tempfile::tempdir().expect("temporary print book");
            let mut print = Exporter::new(print_dir.path().into(), "typst", Output::Auto);
            let markdown = print.render(&diagram, None).expect("static fallback");
            assert!(markdown.starts_with("![") && markdown.contains(".png"));
            let assets: Vec<_> = fs::read_dir(print_dir.path().join(ASSET_DIR))
                .expect("assets")
                .collect();
            assert_eq!(assets.len(), 1);
        }
    }

    /// Nested chapters must reference real images; identical images are shared across chapters.
    #[test]
    fn portable_images_resolve_and_decode() {
        let directory = tempfile::tempdir().expect("temporary source directory");
        let mut exporter = Exporter::new(directory.path().into(), "typst", Output::Auto);
        for format in [Output::Svg, Output::Png, Output::Auto] {
            let appearance = Appearance {
                size: 128,
                alt: Some("[King] & pawn".into()),
                caption: Some("A \"quoted\" caption".into()),
                ..Appearance::default()
            };
            let diagram = Diagram {
                svg: render::board(&Board::default(), &[], &[], &[], &appearance),
                appearance,
                output: Some(format),
                animation: None,
                frames: vec![],
            };
            let markdown = exporter
                .render(&diagram, Some(Path::new("openings/deep/chapter.md")))
                .expect("export image");
            let (url, title) = Parser::new(&markdown)
                .find_map(|e| match e {
                    Event::Start(Tag::Image {
                        dest_url, title, ..
                    }) => Some((dest_url.into_string(), title.into_string())),
                    _ => None,
                })
                .expect("portable Markdown image");
            assert!(url.starts_with("../../mdbook-chess-assets/"));
            assert_eq!(title, "A \"quoted\" caption");
            let path = directory.path().join(url.trim_start_matches("../"));
            let bytes = fs::read(&path).expect("image on disk");
            if format != Output::Svg {
                let pixels = tiny_skia::Pixmap::decode_png(&bytes).expect("valid PNG");
                assert_eq!((pixels.width(), pixels.height()), (128, 128));
            } else {
                roxmltree::Document::parse(std::str::from_utf8(&bytes).expect("UTF-8 SVG"))
                    .expect("valid SVG");
            }
            let modified = fs::metadata(&path)
                .expect("metadata")
                .modified()
                .expect("mtime");
            let root_link = exporter
                .render(&diagram, Some(Path::new("chapter.md")))
                .expect("repeat export");
            assert!(!root_link.contains("../"));
            assert_eq!(
                fs::metadata(&path)
                    .expect("metadata")
                    .modified()
                    .expect("mtime"),
                modified
            );
        }
    }
}
