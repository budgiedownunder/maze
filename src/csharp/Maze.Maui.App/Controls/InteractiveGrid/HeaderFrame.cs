using Microsoft.Maui.Controls;
using Maze.Maui.App.Controls;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public enum HeaderType
    {
        Corner = 0,
        Row = 1,
        Column = 2
    }

    public class HeaderFrame : Frame
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
