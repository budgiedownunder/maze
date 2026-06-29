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
        /// <summary>The curated Play 3D difficulties (global boards).</summary>
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
    /// An option in the Game picker (the second cascade level): exactly one of a
    /// played maze or a curated difficulty.
    /// </summary>
    public class GameOption
    {
        /// <summary>Display label (maze name or difficulty name).</summary>
        public string Label { get; }

        /// <summary>The stored maze id, or <c>null</c> for a curated difficulty.</summary>
        public string? MazeId { get; }

        /// <summary>The curated difficulty, or <c>null</c> for a stored maze.</summary>
        public Difficulty? Difficulty { get; }

        private GameOption(string label, string? mazeId, Difficulty? difficulty)
        {
            Label = label;
            MazeId = mazeId;
            Difficulty = difficulty;
        }

        /// <summary>A game option for a played stored maze.</summary>
        /// <param name="mazeId">The maze id</param>
        /// <param name="name">Display name</param>
        /// <returns>The option</returns>
        public static GameOption ForMaze(string mazeId, string name) => new(name, mazeId, null);

        /// <summary>A game option for a curated difficulty.</summary>
        /// <param name="difficulty">The difficulty</param>
        /// <returns>The option</returns>
        public static GameOption ForDifficulty(Difficulty difficulty) => new(difficulty.ToString(), null, difficulty);

        /// <inheritdoc/>
        public override string ToString() => Label;
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
