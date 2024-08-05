### Langton's Ant with Rust and Bevy

Welcome to the [Langton's Ant](https://en.wikipedia.org/wiki/Langton%27s_ant) implementation using Rust and [Bevy](https://bevyengine.org)! This project demonstrates Langton's Ant, a classic algorithmic system that exhibits complex emergent behavior from simple rules. The project leverages the Bevy game engine to visualize the ant's movement on a grid.

## Getting Started

To get started with the project, you'll need to have Rust and Cargo installed on your machine. If you haven't installed Rust yet, you can get it from [the official Rust website](https://www.rust-lang.org/).

### Usage

```shell
cargo run
```

### Custom render rate (60 is default)

```shell
cargo run -- --rate 144
cargo run -- -r 144
```

### Pattern support

```shell
cargo run -- --pattern RRLLLRLLLRRR
cargo run -- -p RRLLLRLLLRRR
```

### Controls

Use `space` - pause/unpause iteration

## License

This project is licensed under the MIT License - see the LICENSE file for details.
