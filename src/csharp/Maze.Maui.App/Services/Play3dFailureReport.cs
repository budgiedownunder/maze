using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Pure helpers that turn a <see cref="GameFailure"/> into the two things the
    /// 3D game page needs when a run dies: a sentence for the player, and a
    /// subject label for the log. Kept free of page and WebView dependencies so
    /// both are unit-testable (the page's navigation path is not, matching
    /// <see cref="Play3dGameHostUrl"/>).
    /// </summary>
    public static class Play3dFailureReport
    {
        /// <summary>
        /// The sentence to show the player. Falls back to the generic reason when
        /// the failure carries none — a run that died without being classifiable
        /// still has to say something, and an empty alert would read as a bug in
        /// the app rather than a stopped game.
        /// </summary>
        /// <param name="failure">The failure reported by the page or the platform</param>
        /// <returns>A non-empty player-facing message</returns>
        public static string AlertMessage(GameFailure failure) =>
            string.IsNullOrWhiteSpace(failure.Reason) ? GameFailure.GenericReason : failure.Reason.Trim();

        /// <summary>
        /// Describes what was being played, for the log. The page knows its launch
        /// subject but not the run's dimensions — a <c>?def=</c> launch is resolved
        /// server-side, so MAUI never sees its rows/cols/level count.
        /// </summary>
        /// <param name="mazeId">The stored maze id, when launched from a maze</param>
        /// <param name="definitionId">The game-definition id, when launched from a stored game</param>
        /// <returns>A short subject label, never empty</returns>
        public static string Subject(string? mazeId, string? definitionId)
        {
            if (!string.IsNullOrEmpty(mazeId)) return $"maze {mazeId}";
            if (!string.IsNullOrEmpty(definitionId)) return $"definition {definitionId}";
            return "no subject";
        }
    }
}
