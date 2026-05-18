using System.Text.Json;
using Maze.Maui.App.Models;
using Microsoft.Maui.Storage;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Loads + saves <see cref="Play3dCustomLaunchSettings"/> to MAUI
    /// <see cref="Preferences"/>. Pulled out of the model so the test
    /// project (plain net10.0, no MAUI runtime) can file-link the model
    /// without dragging in MAUI types.
    /// </summary>
    public static class Play3dCustomLaunchSettingsStore
    {
        /// <summary>
        /// Loads the user's last-saved settings from
        /// <see cref="Preferences"/>, or returns the defaults if no
        /// settings are stored / the stored payload is invalid. Same
        /// forgiving policy as the React SPA's
        /// <c>loadPlay3dCustomLaunchSettings</c>.
        /// </summary>
        public static Play3dCustomLaunchSettings Load()
        {
            var raw = Preferences.Default.Get(Play3dCustomLaunchSettings.PreferencesKey, string.Empty);
            if (string.IsNullOrWhiteSpace(raw)) return new Play3dCustomLaunchSettings();
            try
            {
                var parsed = JsonSerializer.Deserialize<Play3dCustomLaunchSettings>(raw);
                if (parsed is null) return new Play3dCustomLaunchSettings();
                // Validate enums; fall back to defaults on unknown wire values.
                if (!Play3dCustomLaunchSettings.IsValidSkyType(parsed.SkyType)) parsed.SkyType = "night";
                if (!Play3dCustomLaunchSettings.IsValidWallType(parsed.WallType)) parsed.WallType = "brick";
                if (parsed.TimerSeconds <= 0) parsed.TimerSeconds = 60;
                return parsed;
            }
            catch (JsonException)
            {
                return new Play3dCustomLaunchSettings();
            }
        }

        /// <summary>Persists the settings to <see cref="Preferences"/>.</summary>
        public static void Save(Play3dCustomLaunchSettings settings)
        {
            Preferences.Default.Set(
                Play3dCustomLaunchSettings.PreferencesKey,
                JsonSerializer.Serialize(settings));
        }
    }
}
