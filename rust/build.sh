#!/bin/sh
tmux renamew -t $TMX_WINID building...
clear
if exectime cargo build --release; then
cp target/release/libplayground.so ../godot/lib/libplayground.so
fi

