namespace Maze.Maui.App.Models
{
    /// <summary>
    /// Play 3D difficulty level. The lowercase token (see
    /// <see cref="DifficultyExtensions.ToQueryValue"/>) is what the
    /// <c>/game/?difficulty=…</c> query expects; the server maps it to a
    /// maze-size / timer / seed preset.
    /// </summary>
    public enum Difficulty { Easy, Tricky, Hard }

    /// <summary>
    /// Helpers for <see cref="Difficulty"/>.
    /// </summary>
    public static class DifficultyExtensions
    {
        /// <summary>
        /// Returns the lowercase token used in the <c>/game/?difficulty=…</c>
        /// query (e.g. <see cref="Difficulty.Tricky"/> → <c>"tricky"</c>).
        /// </summary>
        /// <param name="difficulty">Difficulty value</param>
        /// <returns>Lowercase query token</returns>
        public static string ToQueryValue(this Difficulty difficulty) =>
            difficulty.ToString().ToLowerInvariant();
    }
}
