using System.Text.Json;
using Maze.Maui.App.Models;
using Microsoft.Maui.Storage;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Loads + saves <see cref="MazeGameSettings"/> to MAUI
    /// <see cref="Preferences"/>. Pulled out of the model so the test
    /// project (plain net10.0, no MAUI runtime) can file-link the model
    /// without dragging in MAUI types.
    /// </summary>
    public static class MazeGameSettingsStore
    {
        /// <summary>
        /// Loads the user's last-saved settings from
        /// <see cref="Preferences"/>, or returns the defaults if no
        /// settings are stored / the stored payload is invalid. Same
        /// forgiving policy as the React SPA's
        /// <c>loadMazeGameSettings</c>.
        /// </summary>
        public static MazeGameSettings Load()
        {
            var raw = Preferences.Default.Get(MazeGameSettings.PreferencesKey, string.Empty);
            if (string.IsNullOrWhiteSpace(raw)) return new MazeGameSettings();
            try
            {
                var parsed = JsonSerializer.Deserialize<MazeGameSettings>(raw);
                if (parsed is null) return new MazeGameSettings();
                // Validate enums; fall back to defaults on unknown wire values.
                if (!MazeGameSettings.IsValidSkyType(parsed.SkyType)) parsed.SkyType = "night";
                if (!MazeGameSettings.IsValidWallType(parsed.WallType)) parsed.WallType = "brick";
                if (!MazeGameSettings.IsValidDoorStyle(parsed.DoorStyle)) parsed.DoorStyle = "swing";
                if (!MazeGameSettings.IsValidKeyHolder(parsed.KeyHolder)) parsed.KeyHolder = "pedestal";
                if (!MazeGameSettings.IsValidEnemyType(parsed.EnemyType)) parsed.EnemyType = "goblin";
                if (!MazeGameSettings.IsValidHealthStyle(parsed.HealthStyle)) parsed.HealthStyle = "heart";
                if (parsed.TimerSeconds <= 0) parsed.TimerSeconds = 60;
                return parsed;
            }
            catch (JsonException)
            {
                return new MazeGameSettings();
            }
        }

        /// <summary>Persists the settings to <see cref="Preferences"/>.</summary>
        public static void Save(MazeGameSettings settings)
        {
            Preferences.Default.Set(
                MazeGameSettings.PreferencesKey,
                JsonSerializer.Serialize(settings));
        }
    }
}
