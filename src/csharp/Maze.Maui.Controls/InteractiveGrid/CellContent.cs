namespace Maze.Maui.Controls.InteractiveGrid
{
    /// <summary>
    /// The `DefaultCellContent` class represents the default content for a grid cell (which is an empty label)
    /// </summary>
    public class DefaultCellContent : ContentView
    {
        /// <summary>
        /// Constructor
        /// </summary>
        public DefaultCellContent()
        {
            Content = new Label();
        }
    }
}
