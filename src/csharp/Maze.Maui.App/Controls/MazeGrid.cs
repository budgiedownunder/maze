
namespace Maze.Maui.App.Controls
{
    public class MazeGrid : Maze.Maui.App.Controls.InteractiveGrid.Grid
    {
      //  private Maze.Api.Maze maze = new Maze.Api.Maze(5, 5);

        public MazeGrid()
        {
            this.RowCount = 10; //(int)maze.RowCount;
            this.ColCount = 10; //(int)maze.ColCount;
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
