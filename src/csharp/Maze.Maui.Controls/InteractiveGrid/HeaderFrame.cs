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
        /// <returns>Corner header</returns>
        Corner = 0,
        /// <summary>
        /// Row header
        /// </summary>
        /// <returns>Row header</returns>
        Row = 1,
        /// <summary>
        /// Column header
        /// </summary>
        /// <returns>Column header</returns>
        Column = 2
    }
    /// <summary>
    /// The `HeaderFrame` class represents a header cell frame
    /// </summary>
    public class HeaderFrame : Border
    {
        // Private properties
        readonly HeaderType type;
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
