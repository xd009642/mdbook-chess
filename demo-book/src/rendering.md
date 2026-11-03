# Compositing and Markdown

Boards share a page without sharing SVG identifiers. Highlights sit behind pieces;
arrows sit above them.

```chess
moves: [e4, e5, Nf3]
highlights: [e4, e5]
lines: [f3->e5, a1<->a8]
```

| Position | Notes |
| --- | --- |
| **Main line** | _Emphasis_ and tables remain unchanged. |

> A diagram inside a quotation:
>
> ~~~chess
> moves: [d4, d5]
> lines: [c1->f4]
> ~~~
> **The explanation stays in the same quotation.**

1. A diagram inside a list:

   ```chess
   moves: [e4, c5]
   ```

2. Ordinary code remains ordinary code:

   ```rust
   let message = "not a chess diagram";
   ```
