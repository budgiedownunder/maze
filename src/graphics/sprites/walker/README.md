# Walker Sprites

Source frames and assembled GIF animations for the walker character.

## Animations

| Animation | Frames | FPS | Loop | Description |
|-----------|--------|-----|------|-------------|
| `walker_down` | 6 | 10 | Yes | Walking downward |
| `walker_up` | 6 | 10 | Yes | Walking upward |
| `walker_left` | 6 | 10 | Yes | Walking left |
| `walker_right` | 6 | 10 | Yes | Walking right |
| `walker_celebrate` | 6 | 8 | Yes | Walker celebrating |

## Structure

Each animation has a subfolder containing numbered PNG frames (`*_00.png` … `*_05.png`). The assembled GIF for each animation sits alongside its folder.

```
walker/
  walker_down/         ← PNG frames
  walker_down.gif      ← assembled GIF
  walker_up/
  walker_up.gif
  walker_left/
  walker_left.gif
  walker_right/
  walker_right.gif
  walker_celebrate/
  walker_celebrate.gif
  sprite_sheet.png     ← combined sprite sheet
  sprite_atlas.json    ← animation layout definition
  source_manifest.json ← frame ordering and animation metadata
```

- `sprite_atlas.json` maps animation names to rows and column indices within `sprite_sheet.png`, for use by renderers or animation tools that consume a sprite atlas.

## Output locations

The assembled GIFs are deployed to:

- `src/csharp/Maze.Maui.App/Resources/Images/` — MAUI app
- `src/react/maze_web_server/public/images/maze/` — React web app
