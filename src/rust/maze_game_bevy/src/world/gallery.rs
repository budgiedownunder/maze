//! Native-only "rig gallery" demos, selected via the `MAZE_DEMO` environment
//! variable, for eyeballing each entity rig against its default without
//! authoring a maze through the web stack. `gallery` shows every type in one
//! maze; the focused values (`enemies`, `health`, `keysdoors`, `treasure`,
//! `walls`) show one type in isolation, which keeps verification simple as more
//! rig types are added.
//!
//! Enemies are neutralised — stationary (a huge `movePeriodMs`) and harmless
//! (zero `damage`) — so the rigs can be inspected without being chased or killed;
//! their `enemyType` (goblin default / ghost override) still drives the rig.
//! Static rigs (health/key/door) sit where they're visible; doors are openable
//! (collect the keys first) so each open-motion can be seen. The full gallery
//! also places a swing door on the top boundary row (open into the maze) to
//! confirm a boundary-capped corridor still swings. Not used by the web/WASM
//! path (it always supplies its own maze).

use serde_json::{json, Value};

/// Recognised `MAZE_DEMO` values. `gallery` shows everything; the others focus
/// on a single entity type.
const FOCUSES: &[&str] = &["gallery", "enemies", "health", "keysdoors", "treasure", "walls"];

/// The requested gallery focus from `MAZE_DEMO`, or `None` when it is unset or
/// names something other than a known gallery (in which case the caller falls
/// back to the normal demo maze).
pub(crate) fn requested_focus() -> Option<String> {
    let value = std::env::var("MAZE_DEMO").ok()?;
    FOCUSES.contains(&value.as_str()).then_some(value)
}

/// The gallery maze JSON for `focus` (`enemies` / `health` / `keysdoors`, or any
/// other value for the full `gallery`).
pub(crate) fn json(focus: &str) -> String {
    // Neutralised enemies: stationary + harmless display pieces.
    let goblin = || json!([{ "type": "E", "damage": 0, "movePeriodMs": 3_600_000.0 }]);
    let ghost =
        || json!([{ "type": "E", "enemyType": "ghost", "damage": 0, "movePeriodMs": 3_600_000.0 }]);

    // Build a pure-char grid from row templates; rig overrides are overlaid after.
    let build = |rows: &[&str]| -> Vec<Vec<Value>> {
        rows.iter()
            .map(|r| r.chars().map(|c| Value::String(c.to_string())).collect())
            .collect()
    };

    let grid: Vec<Vec<Value>> = match focus {
        "enemies" => {
            // Goblin + ghost in alcoves off a short spine.
            let mut g = build(&["WWWWW", "S   F", "WEWEW", "WWWWW"]);
            g[2][1] = goblin();
            g[2][3] = ghost();
            g
        }
        "health" => {
            // Heart (default) + potion in alcoves off a short spine.
            let mut g = build(&["WWWWW", "S   F", "WHWHW", "WWWWW"]);
            g[2][3] = json!([{ "type": "H", "healthStyle": "potion" }]);
            g
        }
        "treasure" => {
            // All four treasure styles in dead-end alcoves off a short spine, so
            // the open chests face the spine (outward) and aren't auto-collected
            // by walking it. Silver is the bare-'T' default; the rest override.
            let mut g = build(&["WWWWWWWWW", "S       F", "WTWTWTWTW", "WWWWWWWWW"]);
            g[2][3] = json!([{ "type": "T", "style": "gold" }]);
            g[2][5] = json!([{ "type": "T", "style": "diamonds" }]);
            g[2][7] = json!([{ "type": "T", "style": "jewels" }]);
            g
        }
        "walls" => {
            // A straight spine flanked by every wall type, each beside default
            // (brick) walls for contrast. North wall = the solid textures
            // (dressed stone / wood / cobblestone); south wall = the non-occluding
            // types (water / lava / iron fence). Plain `'W'` cells stay brick.
            let mut g = build(&[
                "WWWWWWWW", // north wall: dressed_stone(1) / wood(3) / cobblestone(5)
                "S      F", // spine the player walks
                "WWWWWWWW", // south wall: water(1) / lava(3) / iron_fence(5)
            ]);
            g[0][1] = json!([{ "type": "W", "wallType": "dressed_stone" }]);
            g[0][3] = json!([{ "type": "W", "wallType": "wood" }]);
            g[0][5] = json!([{ "type": "W", "wallType": "cobblestone" }]);
            g[2][1] = json!([{ "type": "W", "wallType": "water" }]);
            g[2][3] = json!([{ "type": "W", "wallType": "lava" }]);
            g[2][5] = json!([{ "type": "W", "wallType": "iron_fence" }]);
            g
        }
        "keysdoors" => {
            // Spine: keys (pedestal/chest/floating + spares) then the four door
            // styles; a boundary swing door (open south) sits at (0, 1).
            let mut g = build(&[
                "WDWWWWWWWWWWWWWW", // boundary swing door at col 1
                "WKWWWWWWWWWWWWWW", // stub + key feeding it
                "S KKKKKD D D D F", // keys (2–6) then doors (7,9,11,13)
                "WWWWWWWWWWWWWWWW",
            ]);
            g[2][3] = json!([{ "type": "K", "keyHolder": "chest" }]);
            g[2][4] = json!([{ "type": "K", "keyHolder": "floating_key" }]);
            g[2][9] = json!([{ "type": "D", "doorStyle": "slide" }]);
            g[2][11] = json!([{ "type": "D", "doorStyle": "portcullis" }]);
            g[2][13] = json!([{ "type": "D", "doorStyle": "dissolve" }]);
            g
        }
        // "gallery" (all) — every type in one maze.
        _ => {
            let mut g = build(&[
                "WDWWWWWWWWWWWWWWWWW", // boundary swing door at col 1
                "WKWWWWWWWWWWWWWWWWW", // stub + key feeding it
                "S    KKKKKD D D D F", // spine: keys (5–9) then doors (10,12,14,16)
                "WETETHTHTWWWWWWWWWW", // alcoves: goblin/ghost (1,3), heart/potion (5,7), treasure (2,4,6,8)
                "WWWWWWWWWWWWWWWWWWW",
            ]);
            g[2][6] = json!([{ "type": "K", "keyHolder": "chest" }]);
            g[2][7] = json!([{ "type": "K", "keyHolder": "floating_key" }]);
            g[2][12] = json!([{ "type": "D", "doorStyle": "slide" }]);
            g[2][14] = json!([{ "type": "D", "doorStyle": "portcullis" }]);
            g[2][16] = json!([{ "type": "D", "doorStyle": "dissolve" }]);
            g[3][1] = goblin();
            g[3][3] = ghost();
            g[3][7] = json!([{ "type": "H", "healthStyle": "potion" }]);
            // Treasure: silver (bare 'T' at col 2) + gold / diamonds / jewels.
            g[3][4] = json!([{ "type": "T", "style": "gold" }]);
            g[3][6] = json!([{ "type": "T", "style": "diamonds" }]);
            g[3][8] = json!([{ "type": "T", "style": "jewels" }]);
            g
        }
    };

    json!({ "grid": grid }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use maze::{MazeGame, MazeGameOptions};

    // Serialise the (cap-1) override on a cell back to its wire JSON, or `None`
    // for a default-rig (bare-char) cell.
    fn wire_at(game: &MazeGame, rc: (usize, usize)) -> Option<String> {
        game.cell_entities()
            .get(&rc)
            .and_then(|v| v.first())
            .map(|e| serde_json::to_string(e).expect("cell entity serialises"))
    }

    #[test]
    fn every_gallery_is_valid_and_playable() {
        // The galleries are hand-authored, so guard each like the demo: it must
        // parse into a structurally valid, loadable maze.
        for focus in FOCUSES {
            MazeGame::from_json_with_options(&json(focus), MazeGameOptions::default())
                .unwrap_or_else(|e| panic!("gallery '{focus}' must be a valid maze: {e:?}"));
        }
    }

    #[test]
    fn full_gallery_overrides_land_on_the_expected_cells() {
        let game = MazeGame::from_json_with_options(&json("gallery"), MazeGameOptions::default())
            .expect("gallery maze must be a valid, loadable maze");

        // Overridden cells carry their variant. Enemies are neutralised
        // (stationary + harmless) while keeping their rig.
        assert_eq!(
            wire_at(&game, (3, 1)).as_deref(),
            Some(r#"{"type":"E","damage":0,"movePeriodMs":3600000.0}"#),
            "goblin keeps the default rig but is neutralised",
        );
        assert_eq!(
            wire_at(&game, (3, 3)).as_deref(),
            Some(r#"{"type":"E","enemyType":"ghost","damage":0,"movePeriodMs":3600000.0}"#),
        );
        assert_eq!(wire_at(&game, (3, 7)).as_deref(), Some(r#"{"type":"H","healthStyle":"potion"}"#));
        assert_eq!(wire_at(&game, (2, 6)).as_deref(), Some(r#"{"type":"K","keyHolder":"chest"}"#));
        assert_eq!(wire_at(&game, (2, 7)).as_deref(), Some(r#"{"type":"K","keyHolder":"floating_key"}"#));
        assert_eq!(wire_at(&game, (2, 12)).as_deref(), Some(r#"{"type":"D","doorStyle":"slide"}"#));
        assert_eq!(wire_at(&game, (2, 14)).as_deref(), Some(r#"{"type":"D","doorStyle":"portcullis"}"#));
        assert_eq!(wire_at(&game, (2, 16)).as_deref(), Some(r#"{"type":"D","doorStyle":"dissolve"}"#));

        // Treasure styles: gold / diamonds / jewels overrides; silver is bare.
        assert_eq!(wire_at(&game, (3, 4)).as_deref(), Some(r#"{"type":"T","style":"gold"}"#));
        assert_eq!(wire_at(&game, (3, 6)).as_deref(), Some(r#"{"type":"T","style":"diamonds"}"#));
        assert_eq!(wire_at(&game, (3, 8)).as_deref(), Some(r#"{"type":"T","style":"jewels"}"#));

        // Default-rig cells stay bare chars (no override entry).
        assert_eq!(wire_at(&game, (3, 5)), None, "heart is the default health rig");
        assert_eq!(wire_at(&game, (3, 2)), None, "silver is the default treasure rig");
        assert_eq!(wire_at(&game, (2, 5)), None, "pedestal key is the default rig");
        assert_eq!(wire_at(&game, (2, 10)), None, "swing door is the default rig");
    }

    #[test]
    fn enemies_gallery_neutralises_both_enemies() {
        let game = MazeGame::from_json_with_options(&json("enemies"), MazeGameOptions::default())
            .expect("enemies gallery must be a valid, loadable maze");
        assert_eq!(
            wire_at(&game, (2, 1)).as_deref(),
            Some(r#"{"type":"E","damage":0,"movePeriodMs":3600000.0}"#),
        );
        assert_eq!(
            wire_at(&game, (2, 3)).as_deref(),
            Some(r#"{"type":"E","enemyType":"ghost","damage":0,"movePeriodMs":3600000.0}"#),
        );
    }

    #[test]
    fn treasure_gallery_overrides_land_on_the_expected_cells() {
        let game = MazeGame::from_json_with_options(&json("treasure"), MazeGameOptions::default())
            .expect("treasure gallery must be a valid, loadable maze");
        // Silver is the bare-'T' default; gold / diamonds / jewels override.
        assert_eq!(wire_at(&game, (2, 1)), None, "silver is the default treasure rig");
        assert_eq!(wire_at(&game, (2, 3)).as_deref(), Some(r#"{"type":"T","style":"gold"}"#));
        assert_eq!(wire_at(&game, (2, 5)).as_deref(), Some(r#"{"type":"T","style":"diamonds"}"#));
        assert_eq!(wire_at(&game, (2, 7)).as_deref(), Some(r#"{"type":"T","style":"jewels"}"#));
    }

    #[test]
    fn walls_gallery_overrides_land_on_the_expected_cells() {
        let game = MazeGame::from_json_with_options(&json("walls"), MazeGameOptions::default())
            .expect("walls gallery must be a valid, loadable maze");
        // North wall: the three non-default solid textures.
        assert_eq!(wire_at(&game, (0, 1)).as_deref(), Some(r#"{"type":"W","wallType":"dressed_stone"}"#));
        assert_eq!(wire_at(&game, (0, 3)).as_deref(), Some(r#"{"type":"W","wallType":"wood"}"#));
        assert_eq!(wire_at(&game, (0, 5)).as_deref(), Some(r#"{"type":"W","wallType":"cobblestone"}"#));
        // South wall: the three non-occluding types.
        assert_eq!(wire_at(&game, (2, 1)).as_deref(), Some(r#"{"type":"W","wallType":"water"}"#));
        assert_eq!(wire_at(&game, (2, 3)).as_deref(), Some(r#"{"type":"W","wallType":"lava"}"#));
        assert_eq!(wire_at(&game, (2, 5)).as_deref(), Some(r#"{"type":"W","wallType":"iron_fence"}"#));
        // Plain 'W' cells between them stay brick (the default — no override entry).
        assert_eq!(wire_at(&game, (0, 2)), None, "brick is the default wall");
        assert_eq!(wire_at(&game, (2, 2)), None, "brick is the default wall");
    }
}
