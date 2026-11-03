# Portable diagrams

## SVG file

This diagram is a Markdown image backed by a standalone SVG. The same file can be
included in a PDF document or opened independently.

```chess
output: svg
moves: [e4, e5, Nf3, Nc6, Bb5]
coordinates: true
theme: green
size: 360
alt: "Ruy Lopez: White bishop on b5 pins the c6 knight."
```

## PNG file

PNG works with image-capable backends that do not accept inline HTML or SVG.
The raster image includes the coordinates, arrows and annotations.

```chess
output: png
board: "k7/1q6/8/1Q6/8/8/7P/6K1 w - - 0 1"
moves: [Qxb7, Kxb7, h4]
coordinates: true
theme: blue
size: 360
lines: [h4-h8, h4-d4, h8-d8, d4-d8]
alt: "Square rule: the black king on b7 cannot catch the h4 pawn."
```
