
namespace Maze.Maui.App.Controls.Pointer
{
    public static partial class Pointer
    {
        public static void SetCursor(VisualElement visualElement, Icon icon)
        {
            SetCursorPlatform(visualElement, icon, Application.Current?.Windows.LastOrDefault()?.Page?.Handler?.MauiContext);
        }
        static partial void SetCursorPlatform(this VisualElement visualElement, Icon icon, IMauiContext? mauiContext);
    }
}
