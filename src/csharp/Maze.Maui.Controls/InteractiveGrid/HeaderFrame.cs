using Microsoft.Maui.Controls;

namespace Maze.Maui.Controls.InteractiveGrid
{
    /// <summary>
    /// Represents a type of header
    /// </summary>
    public enum HeaderType
    {
        /// <summary>
        /// Corner header
        /// </summary>
        Corner = 0,
        /// <summary>
        /// Row header
        /// </summary>
        Row = 1,
        /// <summary>
        /// Column header
        /// </summary>
        Column = 2
    }
    /// <summary>
    /// The `HeaderFrame` class represents a header cell frame
    /// </summary>
    public class HeaderFrame : Border
    {
        // Private properties
        HeaderType type;
        /// <summary>
        /// Header type
        /// </summary>
        /// <returns>Header type</returns>
        public HeaderType Type { get => type; }
        /// <summary>
        /// Positional index
        /// </summary>
        /// <returns>Index</returns>
        public int Index { get; set; }
        /// <summary>
        /// Display index
        /// </summary>
        /// <returns>Display index</returns>
        public int DisplayIndex { get => type == HeaderType.Corner ? Index : Index + 1; }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="type">Header type</param>
        /// <param name="index">Positional index</param>
        public HeaderFrame(HeaderType type, int index)
        {
            this.type = type;
            Index = index;
        }
    }
}
