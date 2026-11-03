# Board presentation

## Study from Black’s side

```chess
moves: [d4, Nf6, c4, g6, Nc3, Bg7]
orientation: black
coordinates: true
theme: green
size: 360
align: center
caption: "King’s Indian Defence — Black to prepare ...d6 and ...e5."
alt: "King’s Indian Defence from Black’s perspective."
knight-lines: [f6->e4, b8->c6]
annotations: {g7: good}
```

## Explain an endgame

```chess
board: "k7/1q6/8/1Q6/8/8/7P/6K1 w - - 0 1"
moves: [Qxb7, Kxb7, h4]
coordinates: true
theme: blue
size: 360
last-move: true
colors: {highlight: "#e9c46a", arrow: "#b54635"}
lines: [h4-h8, h4-d4, h8-d8, d4-d8]
caption: "The square rule: can the king catch the pawn?"
```

## Author-supplied move assessments

Annotations describe the author’s analysis; the plugin does not run an engine.

```chess
moves: [e4, e5, Nf3, Nc6, Bc4, Nf6]
coordinates: true
theme: monochrome
size: 360
annotations: {e4: brilliant, e5: good, f3: interesting, c6: dubious, c4: mistake, f6: blunder}
caption: "All six annotation symbols, shown for illustration only."
```
