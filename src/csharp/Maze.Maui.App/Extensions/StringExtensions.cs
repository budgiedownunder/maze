namespace Maze.Maui.App.Extensions;

/// <summary>
/// Extension methods for <see cref="string"/>.
/// </summary>
internal static class StringExtensions
{
    /// <summary>
    /// Returns the string with its first character converted to uppercase.
    /// If the string is null or empty, it is returned unchanged.
    /// </summary>
    /// <param name="s">The string to capitalize.</param>
    /// <returns>The string with its first character in uppercase.</returns>
    internal static string CapitalizeFirst(this string s) =>
        string.IsNullOrEmpty(s) ? s : char.ToUpper(s[0]) + s[1..];
}
