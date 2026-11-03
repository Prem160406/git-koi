# Wukong sprite atlas

The renderer embeds `assets/wukong/atlas.png`, a transparent RGBA atlas with a native
size of 1286 × 368 pixels. The five atlas rectangles and their shared baseline are
recorded in `atlas.json` and mirrored in `src/wukong_render.rs`.

The sprites are extracted from the user-approved Sun Wukong evolution board's
Evolution Line section. Each sprite retains its original pixel proportions, has four
transparent safety pixels around its extracted bounds, and is separated from adjacent
atlas rectangles by twelve pixels. The atlas is a single still pose per stage; current
animation loops move the sprite and animate background clouds, leaves, aura, and effects.
It is not a finished frame-by-frame sprite sheet.

The companion `atlas.json` describes each rectangle and baseline. Preserve transparency
and use nearest-neighbour scaling; do not add smooth interpolation.
