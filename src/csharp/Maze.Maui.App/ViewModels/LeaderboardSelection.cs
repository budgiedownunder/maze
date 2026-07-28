using System.Globalization;
using Maze.Maui.App.Models;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// The two kinds of leaderboard subject the selector cascades into.
    /// </summary>
    public enum LeaderboardGameType
    {
        /// <summary>The player's own stored mazes they have played.</summary>
        MyMazes,
        /// <summary>A stored 3D game's board (keyed on <c>def:&lt;id&gt;</c>).</summary>
        Play3d,
    }

    /// <summary>
    /// An option in the Game Type picker (the first cascade level).
    /// </summary>
    public class GameTypeOption
    {
        /// <summary>Which kind of subject this option selects.</summary>
        public LeaderboardGameType Kind { get; }

        /// <summary>Display label.</summary>
        public string Label { get; }

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="kind">The game-type kind</param>
        /// <param name="label">Display label</param>
        public GameTypeOption(LeaderboardGameType kind, string label)
        {
            Kind = kind;
            Label = label;
        }

        /// <inheritdoc/>
        public override string ToString() => Label;
    }

    /// <summary>
    /// An option in the maze Game picker (the second cascade level for the Mazes type).
    /// </summary>
    public class GameOption
    {
        /// <summary>Display label (maze name).</summary>
        public string Label { get; }

        /// <summary>The stored maze id.</summary>
        public string MazeId { get; }

        private GameOption(string label, string mazeId)
        {
            Label = label;
            MazeId = mazeId;
        }

        /// <summary>A game option for a played stored maze.</summary>
        /// <param name="mazeId">The maze id</param>
        /// <param name="name">Display name</param>
        /// <returns>The option</returns>
        public static GameOption ForMaze(string mazeId, string name) => new(name, mazeId);

        /// <inheritdoc/>
        public override string ToString() => Label;
    }

    /// <summary>
    /// The stored 3D game whose leaderboard is shown — chosen via the game picker or
    /// resolved from a card / the caller's most-recent run. <see cref="OwnerId"/>
    /// gates whether the caller may reset that board (owner or admin);
    /// <see cref="Rotation"/> decides static (<c>def:&lt;id&gt;</c>) vs daily
    /// (<c>def:&lt;id&gt;:&lt;date&gt;</c>) board keying.
    /// </summary>
    public sealed class PickedGame
    {
        /// <summary>The game definition id.</summary>
        public string Id { get; init; } = "";

        /// <summary>Display name.</summary>
        public string Name { get; init; } = "";

        /// <summary>The owning user id (for the reset gate).</summary>
        public string OwnerId { get; init; } = "";

        /// <summary>The game's rotation (<c>static</c> / <c>daily</c>).</summary>
        public string Rotation { get; init; } = GameVocabulary.Rotation.Static;

        /// <summary>Builds a picked game from a definition / play-fetch response.</summary>
        /// <param name="definition">The game definition</param>
        /// <returns>The picked game</returns>
        public static PickedGame From(GameDefinition definition) => new()
        {
            Id = definition.Id,
            Name = definition.Name,
            OwnerId = definition.OwnerId,
            Rotation = definition.Rotation,
        };
    }

    /// <summary>
    /// A selectable day for a daily game's leaderboard: the raw <c>yyyy-mm-dd</c>
    /// board key (<see cref="DateUtc"/>) with a display label — either "Today" (the
    /// pinned first entry) or the date formatted as e.g. <c>20 Jul 2026</c>.
    /// </summary>
    public sealed class BoardDateOption
    {
        /// <summary>Display label ("Today", or the formatted date).</summary>
        public string Label { get; }

        /// <summary>The <c>yyyy-mm-dd</c> UTC date keying the <c>def:&lt;id&gt;:&lt;date&gt;</c> board.</summary>
        public string DateUtc { get; }

        private BoardDateOption(string label, string dateUtc)
        {
            Label = label;
            DateUtc = dateUtc;
        }

        /// <summary>The pinned "Today" option (today's board may be mid-day / empty).</summary>
        /// <param name="dateUtc">Today's <c>yyyy-mm-dd</c> (UTC)</param>
        /// <returns>The option</returns>
        public static BoardDateOption Today(string dateUtc) => new("Today", dateUtc);

        /// <summary>An option for a past day that has a board.</summary>
        /// <param name="dateUtc">The <c>yyyy-mm-dd</c> (UTC)</param>
        /// <returns>The option</returns>
        public static BoardDateOption ForDate(string dateUtc) => new(FormatDate(dateUtc), dateUtc);

        /// <inheritdoc/>
        public override string ToString() => Label;

        // Human-friendly, culture-invariant, unambiguous (e.g. "20 Jul 2026");
        // falls back to the raw value if it isn't a yyyy-mm-dd date.
        private static string FormatDate(string dateUtc) =>
            DateTime.TryParseExact(dateUtc, "yyyy-MM-dd", CultureInfo.InvariantCulture, DateTimeStyles.None, out DateTime parsed)
                ? parsed.ToString("d MMM yyyy", CultureInfo.InvariantCulture)
                : dateUtc;
    }

    /// <summary>
    /// One of the player's mazes, with a display name from the maze list.
    /// </summary>
    public class MazeOption
    {
        /// <summary>The stored maze id.</summary>
        public string MazeId { get; }

        /// <summary>Display name.</summary>
        public string Name { get; }

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="mazeId">The maze id</param>
        /// <param name="name">Display name</param>
        public MazeOption(string mazeId, string name)
        {
            MazeId = mazeId;
            Name = name;
        }
    }
}
