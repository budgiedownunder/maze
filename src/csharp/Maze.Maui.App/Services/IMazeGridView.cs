using Maze.Api;
using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Abstracts the subset of <c>MazeGrid</c> behaviour that
    /// <see cref="ViewModels.MazeGameViewModel"/> drives during game play.
    /// MAUI controls cannot be instantiated in a non-MAUI test host, so the
    /// view model takes this interface and tests inject a mock; the
    /// production <c>MazeGrid</c> control implements it directly.
    /// </summary>
    public interface IMazeGridView
    {
        /// <summary>Gets or sets whether user pointer/keyboard interaction with the grid is suppressed.</summary>
        bool IsInteractionLocked { get; set; }

        /// <summary>Initializes the grid for the given maze.</summary>
        void Initialize(bool enablePanSupport, MazeItem? mazeItem);

        /// <summary>Renders the player sprite at the given cell, facing the given direction.</summary>
        void SetPlayerAt(int row, int col, MazeGameDirection direction);

        /// <summary>Marks the given cell as visited (drops a trail dot).</summary>
        void SetVisitedDotAt(int row, int col);

        /// <summary>Switches the player sprite at the given cell to the celebration pose.</summary>
        void SetPlayerCelebrate(int row, int col);
    }
}
