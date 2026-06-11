using System.Collections.Generic;
using System.Globalization;
using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// The 3D game settings for a user-edited maze (sky, walls, rig styles,
    /// landmarks, timer). Persisted per-maze on <c>MazeItem.GameSettings</c> and
    /// mirrors the React SPA's <c>MazeGameSettings</c> in
    /// <c>src/react/maze_web_server/src/utils/mazeGameSettings.ts</c>. The
    /// <c>[JsonPropertyName]</c> camelCase keys match the React/server
    /// <c>game_settings</c> wire shape, so a maze's settings round-trip across
    /// both clients.
    ///
    /// A plain POCO — the <c>Maze.Maui.App.Tests</c> project file-links it for
    /// unit testing.
    /// </summary>
    public sealed class MazeGameSettings
    {
        /// <summary>Sky type wire token (lowercase).</summary>
        [JsonPropertyName("skyType")]
        public string SkyType { get; set; } = "night";

        /// <summary>Wall texture wire token (lowercase / snake_case).</summary>
        [JsonPropertyName("wallType")]
        public string WallType { get; set; } = "brick";

        /// <summary>Whether the maze perimeter is walled at the grid edge under an open
        /// sky. Enclosed skies (dungeon / chamber) always wall it regardless.</summary>
        [JsonPropertyName("perimeterWalls")]
        public bool PerimeterWalls { get; set; } = true;

        /// <summary>Door style wire token (lowercase / snake_case) — selects the
        /// 3D door rig (swing / slide / portcullis / dissolve).</summary>
        [JsonPropertyName("doorStyle")]
        public string DoorStyle { get; set; } = "swing";

        /// <summary>Key-holder style wire token (lowercase / snake_case) — selects
        /// the 3D pickup rig (pedestal / chest / floating_key).</summary>
        [JsonPropertyName("keyHolder")]
        public string KeyHolder { get; set; } = "pedestal";

        /// <summary>Enemy type wire token (lowercase) — selects the 3D enemy
        /// rig (goblin / ghost).</summary>
        [JsonPropertyName("enemyType")]
        public string EnemyType { get; set; } = "goblin";

        /// <summary>Health-pickup style wire token (lowercase) — selects the
        /// 3D pickup rig (heart / potion).</summary>
        [JsonPropertyName("healthStyle")]
        public string HealthStyle { get; set; } = "heart";

        /// <summary>Per-cell wall tint variation on?</summary>
        [JsonPropertyName("wallTint")]
        public bool WallTint { get; set; } = false;

        /// <summary>Per-quadrant wall material variation on? Disables wall_type + wall_tint when true.</summary>
        [JsonPropertyName("wallMaterialVariation")]
        public bool WallMaterialVariation { get; set; } = false;

        /// <summary>Dead-end landmark objects on?</summary>
        [JsonPropertyName("deadEndObjects")]
        public bool DeadEndObjects { get; set; } = true;

        /// <summary>Sparse wall decorations on?</summary>
        [JsonPropertyName("wallDecorations")]
        public bool WallDecorations { get; set; } = true;

        /// <summary>Floor accents at junctions on?</summary>
        [JsonPropertyName("floorAccents")]
        public bool FloorAccents { get; set; } = true;

        /// <summary>Time limit (seconds). Must be &gt; 0.</summary>
        [JsonPropertyName("timerSeconds")]
        public int TimerSeconds { get; set; } = 60;

        /// <summary>
        /// Appends every settings field as a `&amp;name=value` URL query
        /// fragment (no leading `?` or `&amp;` — caller decides). Booleans
        /// emit as <c>0</c>/<c>1</c>. Field names match the
        /// <c>StartConfig</c> camelCase wire format the
        /// <c>maze_game_bevy_wasm</c> boundary expects, so <c>/game/index.html</c>
        /// can pass them straight through.
        /// </summary>
        public string ToQueryString()
        {
            var ci = CultureInfo.InvariantCulture;
            var parts = new List<string>
            {
                "skyType=" + Uri.EscapeDataString(SkyType),
                "wallType=" + Uri.EscapeDataString(WallType),
                "perimeterWalls=" + (PerimeterWalls ? "1" : "0"),
                "doorStyle=" + Uri.EscapeDataString(DoorStyle),
                "keyHolder=" + Uri.EscapeDataString(KeyHolder),
                "enemyType=" + Uri.EscapeDataString(EnemyType),
                "healthStyle=" + Uri.EscapeDataString(HealthStyle),
                "wallTint=" + (WallTint ? "1" : "0"),
                "wallMaterialVariation=" + (WallMaterialVariation ? "1" : "0"),
                "deadEndObjects=" + (DeadEndObjects ? "1" : "0"),
                "wallDecorations=" + (WallDecorations ? "1" : "0"),
                "floorAccents=" + (FloorAccents ? "1" : "0"),
                "timerSeconds=" + TimerSeconds.ToString(ci),
            };
            return string.Join("&", parts);
        }

        /// <summary>
        /// Returns <c>true</c> when <paramref name="s"/> is a recognised
        /// sky-type wire value. Used by the store + popup to fall back to
        /// the default on a stale stored value.
        /// </summary>
        public static bool IsValidSkyType(string s) =>
            s is "night" or "sunrise" or "day" or "sunset" or "dungeon" or "chamber";

        /// <summary>
        /// Returns <c>true</c> when <paramref name="s"/> is a recognised
        /// wall-type wire value. Used by the store + popup to fall back to
        /// the default on a stale stored value.
        /// </summary>
        public static bool IsValidWallType(string s) =>
            s is "brick" or "dressed_stone" or "wood" or "cobblestone"
                or "water" or "lava" or "iron_fence";

        /// <summary>
        /// Returns <c>true</c> when <paramref name="s"/> is a recognised
        /// door-style wire value. Used by the store + popup to fall back to
        /// the default on a stale stored value.
        /// </summary>
        public static bool IsValidDoorStyle(string s) =>
            s is "swing" or "slide" or "portcullis" or "dissolve";

        /// <summary>
        /// Returns <c>true</c> when <paramref name="s"/> is a recognised
        /// key-holder wire value. Used by the store + popup to fall back to
        /// the default on a stale stored value.
        /// </summary>
        public static bool IsValidKeyHolder(string s) =>
            s is "pedestal" or "chest" or "floating_key";

        /// <summary>
        /// Returns <c>true</c> when <paramref name="s"/> is a recognised
        /// enemy-type wire value. Used by the store + popup to fall back to
        /// the default on a stale stored value.
        /// </summary>
        public static bool IsValidEnemyType(string s) =>
            s is "goblin" or "ghost";

        /// <summary>
        /// Returns <c>true</c> when <paramref name="s"/> is a recognised
        /// health-style wire value. Used by the store + popup to fall back to
        /// the default on a stale stored value.
        /// </summary>
        public static bool IsValidHealthStyle(string s) =>
            s is "heart" or "potion";
    }
}
