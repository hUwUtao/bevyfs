# bevyfs

A filesystem that is intended to use with Bevy the game engine, for the best of compression. There is a reason, because out there, there is not much filesystem for Bevy to choose, and zstd is godtier!

This filesystem took the idea of squashfs, and actually making it feel like home in Rust!

- [x] Filesystem
- [ ] Actually work (50%)
- [ ] Configurable, and embed into build script please (50%)
  
## actually squashing stuff

- train your dictionary, there is no option, since fs is ro, it is best to train first!

    ```sh
    zstd --train -o target/PAKDICT -r assets -B64 
    ```

- `cargo run --bin bundle` - yea create `target/PAK*`, that might be configurable in the future, and needly (feature.3) too