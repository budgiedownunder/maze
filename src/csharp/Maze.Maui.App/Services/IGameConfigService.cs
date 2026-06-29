using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Retrieves server-controlled game configuration. Currently the Play 3D
    /// difficulty presets; a configuration concern kept separate from scoring
    /// (<see cref="IScoresService"/>).
    /// </summary>
    public interface IGameConfigService
    {
        /// <summary>
        /// Reads a curated Play 3D difficulty's preset. The leaderboard UI uses its
        /// fixed seed to build the difficulty's challenge board key.
        /// </summary>
        /// <param name="difficulty">The curated difficulty</param>
        /// <returns>The difficulty's config (difficulty label + fixed seed)</returns>
        Task<Play3dConfig> GetPlay3dConfigAsync(Difficulty difficulty);
    }
}
