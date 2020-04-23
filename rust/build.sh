#!/bin/sh
clear
if cargo build --release; then
cp target/release/libplayground.so ../godot/lib/libplayground.so
fi

