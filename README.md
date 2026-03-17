# FIDE Elo Rating Calculator

A simple calculator written in Rust.
It calculates rating with the FIDE's formula effective from March 2024.
 the game.

Just download, compile, and use.


## Usage

🟡 **[WARN] Compilation requires cargo 1.93 or later.**

or if you need actual yellow markdown styling:

> [WARN] Compilation requires cargo 1.93 or later.

Note: Standard markdown doesn't support text colors. Use bold/italic for emphasis, or HTML `<span style="color: yellow;">` if your renderer supports it.

```
cargo run --release # or `cargo run`
mv target/release/rankfide.exe .. # or `mv target/debug/rankfide.exe ..`
mv probabilities.csv ..
mv initial.csv ..
```
