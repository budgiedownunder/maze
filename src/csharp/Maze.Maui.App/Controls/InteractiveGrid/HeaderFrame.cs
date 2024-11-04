using Microsoft.Maui.Controls;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public enum HeaderType
    {
        Corner = 0,
        Row = 1,
        Column = 2
    }

    public class HeaderFrame : Border
    {
        HeaderType type;
        public HeaderType Type { get => type; }
        public int Position { get; set; }

        public HeaderFrame(HeaderType type, int position)
        {
            this.type = type;
            Position = position;    
        }
    }
}
