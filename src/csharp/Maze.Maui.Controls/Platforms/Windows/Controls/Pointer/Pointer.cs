using Microsoft.Maui.Platform;
using Microsoft.UI.Input;
using Microsoft.UI.Xaml;
using System.Reflection;
using Windows.UI.Core;

namespace Maze.Maui.Controls.Pointer
{
    public static partial class Pointer
    {
        static partial void SetCursorPlatform(this VisualElement visualElement, Icon icon, IMauiContext? mauiContext)
        {
            ArgumentNullException.ThrowIfNull(mauiContext);
            UIElement view = visualElement.ToPlatform(mauiContext);
            view.ChangeCursor(InputCursor.CreateFromCoreCursor(new CoreCursor(GetCursor(icon), 1)));
        }

        static void ChangeCursor(this UIElement uiElement, InputCursor cursor)
        {
            Type type = typeof(UIElement);
            type.InvokeMember("ProtectedCursor", BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.SetProperty | BindingFlags.Instance, null, uiElement, new object[] { cursor });
        }

        static CoreCursorType GetCursor(Icon cursor)
        {
            return cursor switch
            {
                Icon.Wait => CoreCursorType.Wait,
                Icon.Arrow => CoreCursorType.Arrow,
                Icon.SizeAll => CoreCursorType.SizeAll,
                Icon.SizeNorthEastSouthWest => CoreCursorType.SizeNortheastSouthwest,
                Icon.SizeNorthSouth => CoreCursorType.SizeNorthSouth,
                Icon.SizeNorthWestSouthEast => CoreCursorType.SizeNorthwestSoutheast,
                Icon.SizeWestEast => CoreCursorType.SizeWestEast,
                _ => CoreCursorType.Arrow,
            };
        }
    }
}
