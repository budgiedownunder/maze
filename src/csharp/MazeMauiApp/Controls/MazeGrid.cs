
namespace MazeMauiApp.Controls
{
    public class MazeGrid : MazeMauiApp.Controls.InteractiveGrid.Grid
    {
        private Maze.Api.Maze maze = new Maze.Api.Maze(5, 5);

        public MazeGrid()
        {
            this.RowCount = (int)maze.RowCount;
            this.ColCount = (int)maze.ColCount;
            PopulateGrid();
        }

        public override View GetCellContent(int row, int col)
        {
            return new Label
            {
                Text = $"({row},{col})",
                HorizontalOptions = LayoutOptions.Center,
                VerticalOptions = LayoutOptions.Center
            };
        }
    }
}
