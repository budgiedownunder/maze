namespace Maze.Maui.Controls.Pointer
{
    /// <summary>
    /// The static `Pointer` class manages application pointers
    /// </summary>
    public static partial class Pointer
    {
        /// <summary>
        /// Sets the icon to be displayed for a given visual element when a pointer hovers over it
        /// </summary>
        /// <param name="visualElement">Visual element</param>
        /// <param name="icon">Icon to be displayed on hover</param>
        public static void SetCursor(VisualElement visualElement, Icon icon)
        {
            SetCursorPlatform(visualElement, icon, Application.Current?.Windows.LastOrDefault()?.Page?.Handler?.MauiContext);
        }
        static partial void SetCursorPlatform(this VisualElement visualElement, Icon icon, IMauiContext? mauiContext);
    }
}
