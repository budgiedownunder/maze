
using System.Runtime.InteropServices;
using Microsoft.UI.Xaml.Input;
using Windows.System;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public partial class Grid
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

            switch (e.Key)
            {
                case VirtualKey.Left:
                    MoveActiveCellLeft(shiftPressed, ctrlPressed);
                    break;
                case VirtualKey.Right:
                    MoveActiveCellRight(shiftPressed, ctrlPressed);
                    break;
                case VirtualKey.Up:
                    MoveActiveCellUp(shiftPressed, ctrlPressed);
                    break;
                case VirtualKey.Down:
                    MoveActiveCellDown(shiftPressed, ctrlPressed);
                    break;
                case VirtualKey.Home:
                    MoveActiveCellToRowStart(shiftPressed, ctrlPressed);
                    break;
                case VirtualKey.End:
                    MoveActiveCellToColumnEnd(shiftPressed, ctrlPressed);
                    break;
                case VirtualKey.Tab:
                    if (ctrlPressed) return;
                    if (anchorCell == null)
                    {
                        MoveActiveCellOffset(false, shiftPressed ? -1 : 1, 0);
                        return;
                    }
                    if (shiftPressed)
                        MoveAnchorCellToPrevWithinSelection();
                    else
                            MoveAnchorCellToNextWithinSelection();
                    break;
            }
        }

        [DllImport("user32.dll")]
        public static extern short GetAsyncKeyState(int vKey);

        private const int VK_SHIFT = 0x10;
        private const int VK_CTRL = 0x11;

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
    }
}
