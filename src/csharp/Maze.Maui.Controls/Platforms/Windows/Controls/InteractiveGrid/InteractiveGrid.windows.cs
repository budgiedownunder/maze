
using System.Runtime.InteropServices;
using Microsoft.UI.Xaml.Input;
using Microsoft.Maui.Controls;
using Windows.System;

namespace Maze.Maui.Controls.InteractiveGrid
{
    public partial class Grid
    {
        partial void InitializePlatformSpecificCode()
        {
            // Get the native WinUI Window
            var windowObject = Application.Current?.Windows[0].Handler?.PlatformView;
            if (windowObject is null)
            {
                System.Diagnostics.Debug.WriteLine("PlatformView is null");
                return;
            }
            var mauiWinWindow = (Microsoft.UI.Xaml.Window)windowObject;
            if (mauiWinWindow is not null)
            {
                // Subscribe to PreviewKeyDown (tunneling) so we intercept navigation keys
                // before WinUI's ScrollViewer can process them (End/Home would otherwise
                // scroll the Shell page, shifting the grid up into the navigation bar).
                mauiWinWindow.Content.PreviewKeyDown += OnKeyDown;
            }

            // Suppress BringIntoViewRequested on the _dataGrid's native WinUI panel so
            // that adding virtual cells cannot bubble up past our ScrollViewer to the
            // Shell's outer ScrollViewer and shift the page layout (hiding the toolbar).
            // We suppress at the content panel level, not at the ScrollViewer level,
            // so MAUI's internal scroll-positioning logic is not affected.
            _dataGrid.HandlerChanged += (s, e) =>
            {
                if (_dataGrid.Handler?.PlatformView is Microsoft.UI.Xaml.UIElement panel)
                    panel.BringIntoViewRequested += (sender, args) => args.Handled = true;
            };
        }

        private void OnKeyDown(object sender, KeyRoutedEventArgs e)
        {
            var key = GetKey(e.Key);
            OnProcessKeyDown(GetKeyState(), key, true);
            // Mark navigation keys as handled so WinUI elements (e.g. Shell ScrollViewer)
            // do not also respond to them and shift the page layout.
            if (key == Keyboard.Key.Left  || key == Keyboard.Key.Right ||
                key == Keyboard.Key.Up    || key == Keyboard.Key.Down  ||
                key == Keyboard.Key.Home  || key == Keyboard.Key.End)
                e.Handled = true;
        }
        /// <summary>
        /// Determines the current keyboard press state
        /// </summary>
        /// <returns>Key state</returns>
        Keyboard.KeyState GetKeyState()
        {
            Keyboard.KeyState state = Keyboard.KeyState.None;

            if (IsShiftKeyPressed())
                state |= Keyboard.KeyState.Shift;
            if (IsCtrlKeyPressed())
                state |= Keyboard.KeyState.Ctrl;
            if (IsCapsLockKeyPressed())
                state |= Keyboard.KeyState.CapsLock;

            return state;
        }

        Keyboard.Key GetKey(VirtualKey virtualKey)
        {
            return (Keyboard.Key)virtualKey;
        }
        /// <summary>
        /// Determines the current press state of a given key
        /// </summary>
        /// <returns>Key state</returns>

        [DllImport("user32.dll")]
        public static extern short GetAsyncKeyState(int vKey);

        private const int VK_SHIFT = 0x10;
        private const int VK_CTRL = 0x11;
        private const int VK_CAPITAL = 0x14;

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

        private static bool IsCapsLockKeyPressed()
        {
            return IsVirtualKeyPressed(VK_CAPITAL);
        }

    }
}
