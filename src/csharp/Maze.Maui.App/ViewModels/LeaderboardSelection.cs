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
