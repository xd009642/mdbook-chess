use crate::{options::Appearance, render};
use anyhow::{Context, Result};
use resvg::{tiny_skia, usvg};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use chess::Board;
    use pulldown_cmark::{Event, Parser, Tag};

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
