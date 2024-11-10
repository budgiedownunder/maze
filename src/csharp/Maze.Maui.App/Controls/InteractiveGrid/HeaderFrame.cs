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
        public int Index { get; set; }

        public int DisplayIndex { get => type == HeaderType.Corner ? Index : Index + 1; }

        public HeaderFrame(HeaderType type, int index)
        {
            this.type = type;
            Index = index;
        }
    }
}
