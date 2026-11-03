use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    #[default]
    Player,
    Gif,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Animation {
    pub format: Format,
    pub delay: u16,
    pub repeat: bool,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            format: Format::Player,
            delay: 800,
            repeat: true,
        }
    }
}

impl Animation {
    pub fn validate(&self, moves: usize) -> Result<()> {
        if !(1..=511).contains(&moves) {
            bail!("animation requires 1–511 moves (at most 512 frames)");
        }
        if !(100..=60_000).contains(&self.delay) || !self.delay.is_multiple_of(10) {
            bail!("animation delay must be 100–60000 milliseconds, in multiples of 10");
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Frame {
    pub svg: String,
    pub label: String,
}

/// Encode full opaque frames so captures and moving pieces leave no trails between frames.
pub fn gif_frames(
    mut frames: impl Iterator<Item = Result<resvg::tiny_skia::Pixmap>>,
    animation: &Animation,
) -> Result<Vec<u8>> {
    let first = frames.next().expect("validated nonempty animation")?;
    let mut bytes = Vec::new();
    {
        let mut encoder =
            gif::Encoder::new(&mut bytes, first.width() as u16, first.height() as u16, &[])?;
        if animation.repeat {
            encoder.set_repeat(gif::Repeat::Infinite)?;
        }
        for pixels in std::iter::once(Ok(first)).chain(frames) {
            let pixels = pixels?;
            let mut rgba = pixels.data().to_vec();
            let mut frame = gif::Frame::from_rgba_speed(
                pixels.width() as u16,
                pixels.height() as u16,
                &mut rgba,
                10,
            );
            frame.delay = animation.delay / 10;
            frame.dispose = gif::DisposalMethod::Background;
            encoder.write_frame(&frame)?;
        }
    }
    Ok(bytes)
}
