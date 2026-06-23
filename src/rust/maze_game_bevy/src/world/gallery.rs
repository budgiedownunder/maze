//! Native-only "rig gallery" demos, selected via the `MAZE_DEMO` environment
//! variable, for eyeballing each entity rig against its default without
//! authoring a maze through the web stack. `gallery` shows every type in one
//! maze; the focused values (`enemies`, `health`, `keysdoors`, `treasure`,
//! `walls`, `finishes`) show one type in isolation, which keeps verification
//! simple as more rig types are added.
//!
//! `finishes` is a special case: the interim-finish transition rigs (ladder /
//! portal) are driven by `GameConfig::finish_type` + the run's level structure,
//! not by per-cell overrides like the rest of the gallery, so they can't be
//! expressed as grid cells. [`finish_rig_cells`] instead names the cells where
//! `spawn_world` code-spawns each rig for this focus.
//!
//! Enemies are neutralised — stationary (a huge `movePeriodMs`) and harmless
//! (zero `damage`) — so the rigs can be inspected without being chased or killed;
//! their `enemyType` (goblin default / ghost override) still drives the rig.
//! Static rigs (health/key/door) sit where they're visible; doors are openable
//! (collect the keys first) so each open-motion can be seen. The full gallery
//! also places a swing door on the top boundary row (open into the maze) to
//! confirm a boundary-capped corridor still swings. Not used by the web/WASM
//! path (it always supplies its own maze).

use crate::state::FinishType;
use serde_json::{json, Value};

/// Recognised `MAZE_DEMO` values. `gallery` shows everything; the others focus
/// on a single entity type.
const FOCUSES: &[&str] =
    &["gallery", "enemies", "health", "keysdoors", "treasure", "walls", "finishes"];

/// The multi-level demo selectors (dispatched in `world::spawn_world`, not here).
/// Listed so [`validate_demo_env`] can recognise them.
const MULTILEVEL_DEMOS: &[&str] = &["multilevel_edge", "multilevel_centre"];

/// The requested gallery focus from `MAZE_DEMO`, or `None` when it is unset or
/// names something other than a known gallery (in which case the caller falls
/// back to the normal demo maze).
pub(crate) fn requested_focus() -> Option<String> {
    let value = std::env::var("MAZE_DEMO").ok()?;
    FOCUSES.contains(&value.as_str()).then_some(value)
}

/// Validates the `MAZE_DEMO` environment variable at startup. `Ok(())` when it is
/// unset / empty (the normal game) or names a known demo (a rig gallery focus or a
/// multi-level demo); `Err(message)` for any other value, so a typo fails loudly
/// with the valid list rather than silently falling back to the default demo grid.
pub fn validate_demo_env() -> Result<(), String> {
    validate_demo_value(std::env::var("MAZE_DEMO").ok().as_deref())
}

/// The env-free core of [`validate_demo_env`], so the policy is unit-testable
/// without touching the process-global `MAZE_DEMO`.
fn validate_demo_value(value: Option<&str>) -> Result<(), String> {
    match value {
        None | Some("") => Ok(()),
        Some(v) if FOCUSES.contains(&v) || MULTILEVEL_DEMOS.contains(&v) => Ok(()),
        Some(v) => Err(format!(
            "MAZE_DEMO='{v}' is not a recognised demo.\n  Valid values: {}, {}\n  (or leave MAZE_DEMO unset for the normal game).",
            FOCUSES.join(", "),
            MULTILEVEL_DEMOS.join(", "),
        )),
    }
}

#[cfg(test)]
mod validation_tests {
    use super::validate_demo_value;

    #[test]
    fn validate_demo_value_accepts_known_demos_and_unset() {
        assert!(validate_demo_value(None).is_ok(), "unset is the normal game");
        assert!(validate_demo_value(Some("")).is_ok(), "empty is the normal game");
        assert!(validate_demo_value(Some("gallery")).is_ok());
        assert!(validate_demo_value(Some("walls")).is_ok());
        assert!(validate_demo_value(Some("multilevel_edge")).is_ok());
        assert!(validate_demo_value(Some("multilevel_centre")).is_ok());
    }

    #[test]
    fn validate_demo_value_rejects_unknown_values_with_a_helpful_message() {
        // A typo / the old name / an unknown value all fail, and the message
        // lists the valid values to guide the fix.
        for bad in ["multilevel", "multilevel_center", "typo", "Gallery"] {
            let err = validate_demo_value(Some(bad)).unwrap_err();
            assert!(err.contains(bad), "message should quote the bad value: {err}");
            assert!(err.contains("multilevel_centre"), "message should list valid values: {err}");
        }
    }
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
        "finishes" => {
            // A straight spine the player walks S → F. The interim-finish rigs
            // are code-spawned mid-spine (see `finish_rig_cells`); `F` keeps the
            // gold orb (the final-level finish), so all three markers line up.
            build(&["WWWWWWW", "S     F", "WWWWWWW"])
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
                "WETETHTHTWW W WWWWW", // alcoves: enemies/health/treasure (1–8); finish rigs at (3,11)/(3,13)
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

/// Cells where `spawn_world` should code-spawn an interim-finish rig for `focus`,
/// as `(rig, row, col)`. The transition rigs aren't cell overrides, so the
/// `finishes` gallery places them here — mid-spine corridor cells (two open
/// neighbours, so no dead-end landmark spawns on top of them). Empty for every
/// other focus.
pub(crate) fn finish_rig_cells(focus: &str) -> Vec<(FinishType, usize, usize)> {
    match focus {
        // `finishes`: mid-spine corridor cells. `gallery`: the two carved dead-end
        // alcoves off the spine (their landmark is suppressed). Both code-spawned.
        "finishes" => vec![(FinishType::Ladder, 1, 2), (FinishType::Portal, 1, 4)],
        "gallery" => vec![(FinishType::Ladder, 3, 11), (FinishType::Portal, 3, 13)],
        _ => vec![],
    }
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
