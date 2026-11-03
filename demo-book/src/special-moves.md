# Special moves

SAN uses `O-O` for castling, `exd6` for an en-passant capture and `a8=Q+` for
promotion with check. The diagrams show the resulting positions.

## Castling moves both pieces

```chess
board: "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"
moves: [O-O]
coordinates: true
last-move: true
caption: "The white king moves to g1 and the rook to f1."
```

## En passant removes the bypassed pawn

```chess
board: "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1"
moves: [exd6]
coordinates: true
last-move: true
caption: "White arrives on d6; the black pawn on d5 is removed."
```

## Promotion replaces the pawn

```chess
board: "4k3/P7/8/8/8/8/8/4K3 w - - 0 1"
moves: ["a8=Q+"]
coordinates: true
last-move: true
caption: "The a7 pawn becomes a queen on a8, giving check."
```
