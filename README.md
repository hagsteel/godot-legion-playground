# Godot / Legion playground

This is a simple project using Godot, Rust and Legion.

I'm using this to prototype various things for my game.

## Directories

**godot** contains the Godot project

**rust** contains most of the code for this, using Legion ECS

**test** contains the test project, this is only used to run the tests.

## Testing

To run the tests you can use headless Godot from https://godotengine.org/download/server,
and add it to your path as `godot-headless` to run the `test.sh` shell
script in `rust/`.
