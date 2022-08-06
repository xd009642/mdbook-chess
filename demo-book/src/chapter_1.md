# Chapter 1

```chess

```

Lets look at the Caro-Kann defense. First lets create a new board

```chess
save: caro-kann
board: start
```

And White plays e4 and black starts the defense with c6

```chess
load: caro-kann
moves: ["e4", "c6"]
save: ["caro-kann-alt", "caro-kann-alt2"]
```

Now we have the c6 pawn protecting d5 we can move our black pawn here next:

```chess
load: caro-kann
moves: ["d4", "d5"]
```

So now the main line is:

```chess
load: caro-kann
moves: ["e5", "Bf5"]
overwrite: false
```

But there is an alternative where we capture and continue to push the pawn

```chess
load: caro-kann
moves: ["Nc3", "dxe4"]
```

Of course it doesn't work if this happens because an en-passant is threatened:

```chess
load: caro-kann-alt
overwrite: false
moves: ["e5", "d5"]
```

Last check of loading:

```chess
load: caro-kann-alt2
```
