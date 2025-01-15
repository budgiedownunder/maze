
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
                // Subscribe to KeyDown event on the window's content (which is the root element)
                mauiWinWindow.Content.KeyDown += OnKeyDown;
            }
        }

        private void OnKeyDown(object sender, KeyRoutedEventArgs e)
        {
            OnProcessKeyDown(GetKeyState(), GetKey(e.Key), true);
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
