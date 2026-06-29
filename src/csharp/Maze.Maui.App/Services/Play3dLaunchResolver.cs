using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Shared orchestration for the Play 3D launch chooser (Run / Custom Run… / Cancel),
    /// used by both the maze editor and the maze list so the two behave identically.
    /// </summary>
    public static class Play3dLaunchResolver
    {
        /// <summary>
        /// Runs the Run / Custom Run… / Cancel chooser and resolves the settings to launch
        /// with. <c>Run</c> uses <paramref name="saved"/> (the maze's saved settings, or
        /// defaults when null); <c>Custom Run…</c> opens the settings popup seeded from those
        /// for a one-off launch and returns to the chooser if cancelled; <c>Cancel</c> aborts.
        /// </summary>
        /// <param name="dialogService">The dialog service hosting the popups</param>
        /// <param name="mazeName">Maze name shown in the popup titles</param>
        /// <param name="saved">The maze's saved settings, or null for defaults</param>
        /// <returns>The launch settings, or <c>null</c> if the user cancelled the launch</returns>
        public static async Task<MazeGameSettings?> ResolveAsync(IDialogService dialogService, string? mazeName, MazeGameSettings? saved)
        {
            MazeGameSettings effectiveSaved = saved ?? new MazeGameSettings();
            while (true)
            {
                Play3dLaunchChoice choice = await dialogService.ShowPlay3dLaunchChooserAsync(mazeName);
                if (choice == Play3dLaunchChoice.Run)
                {
                    return effectiveSaved;
                }
                if (choice == Play3dLaunchChoice.CustomRun)
                {
                    // One-off launch; Cancel from the settings popup returns to the chooser.
                    MazeGameSettings? custom = await dialogService.ShowMazeGameSettingsAsync(mazeName, effectiveSaved);
                    if (custom is not null)
                    {
                        return custom;
                    }
                    continue;
                }
                return null; // Cancel / dismiss
            }
        }
    }
}
