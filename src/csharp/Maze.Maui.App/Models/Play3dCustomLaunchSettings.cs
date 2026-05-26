using System.Collections.Generic;
using System.Globalization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// Per-launch customisation values for the Play 3D button on user-edited
    /// mazes. Mirrors the React SPA's <c>Play3dCustomLaunchSettings</c> in
    /// <c>src/react/maze_web_server/src/utils/play3dCustomLaunchSettings.ts</c>
    /// so the MAUI app and the web SPA offer the same set of knobs.
    ///
    /// This is a plain POCO with no MAUI / persistence dependencies — the
    /// <c>Maze.Maui.App.Tests</c> project file-links it for unit testing.
    /// Preferences I/O lives in <c>Services.Play3dCustomLaunchSettingsStore</c>
    /// in the main app project.
    /// </summary>
    public sealed class Play3dCustomLaunchSettings
    {
        /// <summary>
        /// Preferences key under which the settings are persisted. Shared
        /// between the store and any caller that wants to clear it.
        /// </summary>
        public const string PreferencesKey = "play3dCustomLaunchSettings";

        /// <summary>Sky type wire token (lowercase).</summary>
        public string SkyType { get; set; } = "night";

        /// <summary>Wall texture wire token (lowercase / snake_case).</summary>
        public string WallType { get; set; } = "brick";

        /// <summary>Door style wire token (lowercase / snake_case) — selects the
        /// 3D door rig (swing / slide / portcullis / dissolve).</summary>
        public string DoorStyle { get; set; } = "swing";

        /// <summary>Key-holder style wire token (lowercase / snake_case) — selects
        /// the 3D pickup rig (pedestal / chest / floating_key).</summary>
        public string KeyHolder { get; set; } = "pedestal";

        /// <summary>Per-cell wall tint variation on?</summary>
        public bool WallTint { get; set; } = false;

        /// <summary>Per-quadrant wall material variation on? Disables wall_type + wall_tint when true.</summary>
        public bool WallMaterialVariation { get; set; } = false;

        /// <summary>Dead-end landmark objects on?</summary>
        public bool DeadEndObjects { get; set; } = true;

        /// <summary>Sparse wall decorations on?</summary>
        public bool WallDecorations { get; set; } = true;

        /// <summary>Floor accents at junctions on?</summary>
        public bool FloorAccents { get; set; } = true;

        /// <summary>Time limit (seconds). Must be &gt; 0.</summary>
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
                "doorStyle=" + Uri.EscapeDataString(DoorStyle),
                "keyHolder=" + Uri.EscapeDataString(KeyHolder),
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
            s is "night" or "sunrise" or "day" or "sunset";

        /// <summary>
        /// Returns <c>true</c> when <paramref name="s"/> is a recognised
        /// wall-type wire value. Used by the store + popup to fall back to
        /// the default on a stale stored value.
        /// </summary>
        public static bool IsValidWallType(string s) =>
            s is "brick" or "dressed_stone" or "wood" or "cobblestone";

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
    }
}
