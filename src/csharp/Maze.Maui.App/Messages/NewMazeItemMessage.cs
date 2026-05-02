using Maze.Maui.App.Models;

namespace Maze.Maui.App.Messages
{
    /// <summary>
    /// Sent when a new maze has been successfully saved and should appear
    /// in the cached maze list. Fires from the maze-edit save path.
    /// </summary>
    public record NewMazeItemMessage(MazeItem Item);
}
