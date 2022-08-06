# Chapter 1

Lets make a default board!

```chess
name: game
board: start
```

And play the move e4, but save a checkpoint.

```chess
name: game
checkpoint: other
moves: ["e4"]
```

Player plays and we match

```chess
name: game
moves: ["e5", "d4"]
```

Or they could have done e5

```chess
name: other
moves: ["d5"]
```


