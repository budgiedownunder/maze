
namespace Maze.Maui.App.Controls
{
    public class MazeGrid : Maze.Maui.App.Controls.InteractiveGrid.Grid
    {
        //  private Maze.Api.Maze maze = new Maze.Api.Maze(5, 5);

        public MazeGrid()
        {
            this.RowCount = 20; //(int)maze.RowCount;
            this.ColumnCount = 30; //(int)maze.ColCount;
            PopulateGrid();
        }

        public override View GetCellContent(int row, int col)
        {
            return new Label
            {
                Text = $"({row + 1},{col + 1})",
                HorizontalOptions = LayoutOptions.Center,
                VerticalOptions = LayoutOptions.Center
            };
        }
    }
}
