# raytris

Fairly naive rust re-implmentation of a [tetris in go from rosetta code wiki](https://rosettacode.org/wiki/Tetris/Go).

The style is terrible because it was copied from this go code, pls don't @ me or think this is how I would write it - why think for yourself when you can copy someone else right? (or as my mother would say, "why have a dog and bark yourself").

Currently blocks above completed lines don't fall correctly, idk why.

Uses:
- [rand](https://crates.io/crates/rand) crate for rng
- [raylib](https://crates.io/crates/raylib) crate for bindings to raylib, used for rendering + various misc functions
- [color-eyre](https://crates.io/crates/color-eyre) for nicer stack traces on panic

Good times, hope you enjoy laughing at how hellish this is (turns out rust & go are very different when it comes to naive implementations).
