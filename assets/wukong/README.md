# Wukong sprite atlas

The renderer expects `assets/wukong/atlas.png`, a transparent PNG atlas with a native
size of 569 × 164 pixels. The five sprite rectangles are defined in
`src/wukong_render.rs` and are aligned to a shared baseline.

The atlas is extracted from the user-approved Wukong evolution reference. It is a
single still pose per stage; current animation loops move the sprite and animate
background clouds, leaves, aura, and effects. It is not a finished frame-by-frame
sprite sheet. Replace this atlas only with a revised, approved set of five consistent
models, keeping the rectangle coordinates and baseline metadata in sync.

The companion `atlas.json` describes each rectangle. Preserve transparency and use
nearest-neighbour scaling; do not add smooth interpolation.
