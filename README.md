# C8E - Chip-8 Emulator

### Chip-8 Emulator written in Rust

This is me trying to build something in Rust, in order to enhance my understanding of the language. Also, I've been learning some computer architecture, so that definitely influenced my decision to make an emulator :)

Working on the emulator was a blast, for the most part. 
I did run into some challenges though, the biggest of which was dealing with input keys. I could'nt get responsive input at first, and even now, the input still feels janky.
I also had a bit of a headache with timing stuff. But it was an overall good experience.

As of now, the emulator works, and most games run fine. For some reason though, Tetris does not run properly. I hope I find out why soon.
I also haven't touched audio, for now.


### How to run
This emulator relies on `sdl2`, so you want to make sure you have that installed.
On Ubuntu, this can be done with `apt install libsdl2-dev`

To build, run `cargo build --release` and then run with `./target/release/c8e <path_to_rom>`