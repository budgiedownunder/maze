
using System.Runtime.InteropServices;
using Microsoft.UI.Xaml.Input;
using Windows.System;

namespace MazeMauiApp.Controls
{
    public partial class MazeGrid
    {
        partial void InitializePlatformSpecificCode()
        {
            // Get the native WinUI Window
            var windowObject = App.Current?.Windows[0].Handler?.PlatformView;
            if (windowObject == null)
            {
                System.Diagnostics.Debug.WriteLine("PlatformView is null");
                return;
            }
            var mauiWinWindow = (Microsoft.UI.Xaml.Window)windowObject;
            if (mauiWinWindow != null)
            {
                // Subscribe to KeyDown event on the window's content (which is the root element)
                mauiWinWindow.Content.KeyDown += OnKeyDown;
            }
        }

        private void OnKeyDown(object sender, KeyRoutedEventArgs e)
        {
            var shiftPressed = IsShiftKeyPressed();
            var ctrlPressed = IsCtrlKeyPressed();
            var endPressed = IsEndKeyPressed();
            var homePressed = IsHomeKeyPressed();

            switch (e.Key)
            {
                case VirtualKey.Left:
                    {
                        int colOffset = ctrlPressed ? -activeCellCol + 1 : -1;
                        MoveActiveCellOffset(shiftPressed, colOffset, 0);
                    }
                    break;
                case VirtualKey.Right:
                    {
                        int colOffset = ctrlPressed ? this.ColCount - activeCellCol : 1;
                        MoveActiveCellOffset(shiftPressed, colOffset, 0);
                    }
                    break;
                case VirtualKey.Up:
                    {
                        int rowOffset = ctrlPressed ? -activeCellRow + 1 : -1;
                        MoveActiveCellOffset(shiftPressed, 0, rowOffset);
                    }
                    break;
                case VirtualKey.Down:
                    {
                        int rowOffset = ctrlPressed ? this.RowCount - activeCellRow : 1;
                        MoveActiveCellOffset(shiftPressed, 0, rowOffset);
                    }
                    break;
                case VirtualKey.Home:
                    {
                        int rowOffset = ctrlPressed ? -activeCellRow + 1 : 0;
                        int colOffset = -activeCellCol;
                        MoveActiveCellOffset(shiftPressed, colOffset, rowOffset);
                    }
                    break;
                case VirtualKey.End:
                    {
                        int rowOffset = ctrlPressed ? this.RowCount - activeCellRow : 0;
                        int colOffset = this.ColCount - activeCellCol;
                        MoveActiveCellOffset(shiftPressed, colOffset, rowOffset);
                    }
                    break;
            }
        }

        [DllImport("user32.dll")]
        public static extern short GetAsyncKeyState(int vKey);

        private const int VK_SHIFT = 0x10;
        private const int VK_CTRL = 0x11;
        private const int VK_END = 0x23;
        private const int VK_HOME = 0x24;

        private static bool IsVirtualKeyPressed(int keyCode)
        {
            return (GetAsyncKeyState(keyCode) & 0x8000) != 0;
        }
        private static bool IsShiftKeyPressed()
        {
            return IsVirtualKeyPressed(VK_SHIFT);
        }

        private static bool IsCtrlKeyPressed()
        {
            return IsVirtualKeyPressed(VK_CTRL);
        }
        private static bool IsEndKeyPressed()
        {
            return IsVirtualKeyPressed(VK_END);
        }
        private static bool IsHomeKeyPressed()
        {
            return IsVirtualKeyPressed(VK_HOME);
        }
    }
}
