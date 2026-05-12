namespace Maze.Maui.App.Utils;

/// <summary>
/// Helpers for evaluating maze size against the server-reported cell-count
/// cap (<c>AppFeatures.MaxMazeCells</c>).
/// </summary>
internal static class MazeCellCap
{
    /// <summary>
    /// Returns <c>true</c> when <paramref name="rows"/> × <paramref name="cols"/>
    /// would exceed the supplied cap. A <c>null</c> cap means the configured
    /// store imposes no cap, in which case this always returns <c>false</c>.
    /// </summary>
    /// <param name="rows">Maze row count.</param>
    /// <param name="cols">Maze column count.</param>
    /// <param name="max">Cap from <c>AppFeatures.MaxMazeCells</c>, or <c>null</c>.</param>
    internal static bool Exceeds(uint rows, uint cols, int? max)
    {
        if (max is not int cap) return false;
        // Compute as ulong so a pathological (uint × uint) never overflows.
        ulong cells = (ulong)rows * cols;
        return cells > (ulong)cap;
    }
}
