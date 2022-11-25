# ulvestein

A wolftein-style raycaster test project. A playground
project for Wolfenstein 3D-like game. Running it should
be as easy as `cargo run`. Since this uses software-like rendering,
the dev profile has some optimisations turned on so the performance isn't horrible.

## Goals

[x] Wolfstein-esque 3D software rendering (using `pixels` to get a pixel framebuffer that it will use the GPU to draw)
[x] See-through materials like windows
[x] Reflective materials like mirros
[] Adjustable height of walls and sprites
[] Ground textures
[] Sprites (specifically ones with locations on the 2D map that will be drawn appropriately like the walls)
[] A gun that shoot things

### Non-goals

- Complete Wolfenstein 3D engine

## Known issues

Sometimes when I run the game on my laptop, it will freeze completely, but restarting it usually works. Weirdly, it seems to happen
the first time I run after I have compiled it. The cause is unknown to me.

