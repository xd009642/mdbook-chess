# Replaying move sequences

## Step through an opening

The controls start paused. Use Previous/Next, the slider (including arrow keys),
or Play/Pause. With JavaScript disabled, the final position remains visible.

```chess
moves: [e4, e5, Nf3, Nc6, Bb5, a6]
animation: {format: player, delay: 800, repeat: true}
coordinates: true
theme: green
size: 360
last-move: true
caption: "The Ruy Lopez, ending with the Morphy Defence."
```

## Download a GIF

The preview starts as a static final position. Click Play GIF to animate it, or
use Download GIF to reuse it elsewhere. Printed pages show the final board.

```chess
board: "k7/1q6/8/1Q6/8/8/7P/6K1 w - - 0 1"
moves: [Qxb7, Kxb7, h4]
animation: {format: gif, delay: 1000, repeat: true}
coordinates: true
theme: blue
size: 320
last-move: true
lines: [h4-h8, h4-d4, h8-d8, d4-d8]
caption: "Trade queens and push the pawn; the square rule appears on the final frame."
```

## Replaying a saved variation

```chess
moves: [d4, d5]
save: queen-pawn
size: 256
```

The animation begins at the saved position, and leaves it available for other
variations because `overwrite` is disabled.

```chess
load: queen-pawn
overwrite: false
moves: [c4, e6, Nc3, Nf6]
animation: {repeat: false}
coordinates: true
orientation: black
size: 320
```
