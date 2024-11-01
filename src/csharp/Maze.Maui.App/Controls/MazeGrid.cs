
namespace Maze.Maui.App.Controls
{
    public class MazeGrid : Maze.Maui.App.Controls.InteractiveGrid.Grid
    {
        //  private Maze.Api.Maze maze = new Maze.Api.Maze(5, 5);

        public delegate void CellTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        public event CellTappedEventHandler? CellTapped;

        public delegate void CellDoubleTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        public event CellDoubleTappedEventHandler? CellDoubleTapped;

        public delegate void SelectionChnagedEventHandler(object sender, MazeGridSelectionChangedEventArgs e);
        public event SelectionChnagedEventHandler? SelectionChanged;

        public MazeGrid()
        {
            this.RowCount = 30; //(int)maze.RowCount;
            this.ColumnCount = 15; //(int)maze.ColCount;
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

        public override void OnCellTapped(Frame cell, int row, int column, bool triggerEvents)
        {
            if(triggerEvents && CellTapped != null)
            {
                CellTapped.Invoke(this, new MazeGridCellTappedEventArgs(cell, row, column, 1));
            } else
            {
                base.OnCellTapped(cell, row, column, false);
            }
        }

        public override void OnCellDoubleTapped(Frame cell, int row, int column, bool triggerEvents)
        {
            if (triggerEvents && CellDoubleTapped != null)
            {
                CellDoubleTapped.Invoke(this, new MazeGridCellTappedEventArgs(cell, row, column, 2));
            }
            else
            {
                base.OnCellDoubleTapped(cell, row, column, false);
            }
        }

        public override void OnSelectionChanged()
        {
            SelectionChanged?.Invoke(this, new MazeGridSelectionChangedEventArgs());
        }

    }

    public class MazeGridCellTappedEventArgs : EventArgs
    {
        public Frame Cell{ get; }
        public int Row { get; }
        public int Column { get; }
        public int NumberTaps { get; }

        public MazeGridCellTappedEventArgs(Frame cell, int row, int column, int numberTaps)
        {
            Cell = cell;
            Row = row;
            Column = column;
            NumberTaps = numberTaps;
        }
    }

    public class MazeGridSelectionChangedEventArgs : EventArgs
    {
        public MazeGridSelectionChangedEventArgs()
        {
        }
    }

}
