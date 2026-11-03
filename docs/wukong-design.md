# Wukong visual and evolution direction

This document records the agreed direction for the Wukong character mode. The original
Git-Koi pipeline is preserved while the new character mode is developed separately.

## Character reference

Use the user's approved original evolution / behaviour / sprite showcase image as the
primary visual reference. Do not redesign the monkey from scratch or substitute later
simplified concept variations.

The five permanent forms are:

1. The Celestial Stone
2. Stone-born Monkey
3. Handsome Monkey King
4. Great Sage, Heaven's Equal
5. Victorious Fighting Buddha

The monkey should become progressively taller and more imposing across the line while
remaining recognizably the same character. Preserve the approved face, fur identity,
tail, headband language, red-and-gold costume language, and the original warm pixel-art
rendering. Do not turn the final form into a completely different character.

## Palette and atmosphere

Preserve the original approved palette:

- chestnut / burnt-sienna fur
- deep crimson and burgundy cloth
- warm antique gold, amber, and orange highlights
- cream-gold cloud highlights
- deep charcoal and plum-purple background

The approved reference includes a golden aura around the full evolution line, including
the stone. Preserve that motif as a progression: subtle at the stone and early monkey
stages, more pronounced around later forms. Floating leaves, sparks, and small particles
are part of the visual language. Background clouds should remain visible as a separate
atmospheric layer, with subtle parallax in animated output.

Do not let aura effects obscure the character silhouette.

## Runtime model

Keep two independent systems:

- Permanent evolution is based on lifetime GitHub-recognized contributions. It never
  reverses because of inactivity.
- Daily behaviour is based on recent activity and controls the current animation and
  energy effects. A fully evolved character can still rest or meditate.

The first implementation uses configurable prototype thresholds. Do not treat these
numbers as final balancing or an account-age fairness policy.

## Asset production requirements

The approved evolution board's Evolution Line is the source for the current five
transparent still sprites in `assets/wukong/atlas.png`. Their extracted rectangles,
padding, and shared baseline are documented in `assets/wukong/atlas.json`. The atlas
does not contain the Behaviour States or Sprite Showcase figures, and it is not a
frame-by-frame animation sheet.

Each form will need dedicated idle, walk, and behaviour-specific animation frames in a
later asset milestone. Background clouds, aura, leaves, and sparks should be separable
layers where practical. Keep the approved designs when producing those frames.
