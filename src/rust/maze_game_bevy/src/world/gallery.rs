//! Native-only "rig gallery" demos, selected via the `MAZE_DEMO` environment
//! variable, for eyeballing each entity rig against its default without
//! authoring a maze through the web stack. `gallery` shows every type in one
//! maze; the focused values (`enemies`, `health`, `keysdoors`) show one type in
//! isolation, which keeps verification simple as more rig types are added.
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
const FOCUSES: &[&str] = &["gallery", "enemies", "health", "keysdoors"];

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
                "WEWEWHWHWWWWWWWWWWW", // alcoves: goblin, ghost, heart, potion
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

        // Default-rig cells stay bare chars (no override entry).
        assert_eq!(wire_at(&game, (3, 5)), None, "heart is the default health rig");
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
}
