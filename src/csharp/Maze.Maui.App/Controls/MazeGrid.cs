
using System.Diagnostics;

namespace Maze.Maui.App.Controls
{
    public class MazeGrid : InteractiveGrid.Grid
    {
        //  private Maze.Api.Maze maze = new Maze.Api.Maze(5, 5);

        public delegate void CellTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        public event CellTappedEventHandler? CellTapped;

        public delegate void CellDoubleTappedEventHandler(object sender, MazeGridCellTappedEventArgs e);
        public event CellDoubleTappedEventHandler? CellDoubleTapped;

        public delegate void SelectionChangedEventHandler(object sender, MazeGridSelectionChangedEventArgs e);
        public event SelectionChangedEventHandler? SelectionChanged;

        public MazeGrid()
        {
            this.RowCount = 30; //(int)maze.RowCount;
            this.ColumnCount = 15; //(int)maze.ColCount;
            PopulateGrid();
        }

        public override View GetCellContent(int row, int column)
        {
            return new Label
            {
                Text = $"({row + 1},{column + 1})",
                HorizontalOptions = LayoutOptions.Center,
                VerticalOptions = LayoutOptions.Center
            };
        }

        public override void OnCellTapped(InteractiveGrid.CellFrame cellFrame, bool triggerEvents)
        {
            if(triggerEvents && CellTapped != null)
            {
                CellTapped.Invoke(this, new MazeGridCellTappedEventArgs(cellFrame, 1));
            } else
            {
                base.OnCellTapped(cellFrame, false);
            }
        }

        public override void OnCellDoubleTapped(InteractiveGrid.CellFrame cellFrame, bool triggerEvents)
        {
            if (triggerEvents && CellDoubleTapped != null)
            {
                CellDoubleTapped.Invoke(this, new MazeGridCellTappedEventArgs(cellFrame, 2));
            }
            else
            {
                base.OnCellDoubleTapped(cellFrame, false);
            }
        }

        public override void OnSelectionChanged()
        {
            SelectionChanged?.Invoke(this, new MazeGridSelectionChangedEventArgs());
        }

    }

    public class MazeGridCellTappedEventArgs : EventArgs
    {
        public InteractiveGrid.CellFrame Cell { get; }
        public int Row { get => Cell.DisplayRow; }
        public int Column { get => Cell.DisplayColumn; }
        public int NumberTaps { get; }

        public MazeGridCellTappedEventArgs(InteractiveGrid.CellFrame cellFrame, int numberTaps)
        {
            Cell = cellFrame;
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
