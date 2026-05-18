
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
            // Subscribe to PreviewKeyDown (tunneling) so we intercept navigation keys
            // before WinUI's ScrollViewer can process them (End/Home would otherwise
            // scroll the Shell page, shifting the grid up into the navigation bar).
            mauiWinWindow?.Content.PreviewKeyDown += OnKeyDown;

            // Suppress BringIntoViewRequested on the _dataGrid's native WinUI panel so
            // that adding virtual cells cannot bubble up past our ScrollViewer to the
            // Shell's outer ScrollViewer and shift the page layout (hiding the toolbar).
            // We suppress at the content panel level, not at the ScrollViewer level,
            // so MAUI's internal scroll-positioning logic is not affected.
            _dataGrid.HandlerChanged += (s, e) =>
            {
                if (_dataGrid.Handler?.PlatformView is Microsoft.UI.Xaml.FrameworkElement panel)
                {
                    panel.BringIntoViewRequested += (sender, args) => args.Handled = true;
                    // Anchor _dataGrid to the top-left of its ScrollViewer so that when the grid
                    // is smaller than the viewport it does not get centred by WinUI layout.
                    panel.HorizontalAlignment = Microsoft.UI.Xaml.HorizontalAlignment.Left;
                    panel.VerticalAlignment = Microsoft.UI.Xaml.VerticalAlignment.Top;
                }
            };
        }

        private void OnKeyDown(object sender, KeyRoutedEventArgs e)
        {
            // We're a window-level PreviewKeyDown handler so we see every key
            // anywhere in the window. Two filters keep the grid from
            // swallowing keys that aren't meant for it:
            //
            // 1. Page check: skip when Shell.Current has navigated away from
            //    the page that hosts this grid. CT.Maui v13 Popups are shown
            //    as Shell-navigated PopupPages — typing inside a CT.Maui
            //    Popup fires this handler with Shell.Current.CurrentPage set
            //    to that PopupPage, not the maze editor page.
            //
            // 2. Overlay check: skip when the focused element is inside a
            //    Microsoft.UI.Xaml.Controls.Primitives.Popup (e.g. MenuFlyout)
            //    or a ContentDialog (Shell.DisplayPromptAsync). These overlay
            //    in place without changing Shell.Current.CurrentPage, so the
            //    page check alone wouldn't catch them.
            if (!IsHostPageCurrent() || IsInsideOverlay(e.OriginalSource))
                return;

            var key = GetKey(e.Key);
            OnProcessKeyDown(GetKeyState(), key, true);
            // Mark navigation keys as handled so WinUI elements (e.g. Shell ScrollViewer)
            // do not also respond to them and shift the page layout.
            if (key == Keyboard.Key.Left || key == Keyboard.Key.Right ||
                key == Keyboard.Key.Up || key == Keyboard.Key.Down ||
                key == Keyboard.Key.Home || key == Keyboard.Key.End)
                e.Handled = true;
        }

        /// <summary>
        /// Returns <c>true</c> if the MAUI Page that hosts this grid is the
        /// Shell's current page. False when Shell has navigated to a
        /// different page (notably a CT MAUI <c>PopupPage</c>, which is how
        /// v13 shows popups). The window-level key handler uses this to
        /// stop processing keys destined for an unrelated page that
        /// happens to share the same window.
        /// </summary>
        private bool IsHostPageCurrent()
        {
            var currentPage = Shell.Current?.CurrentPage;
            if (currentPage is null) return true;
            Element? element = this;
            while (element is not null)
            {
                if (ReferenceEquals(element, currentPage)) return true;
                element = element.Parent;
            }
            return false;
        }

        /// <summary>
        /// Returns <c>true</c> if <paramref name="source"/> sits inside a
        /// Microsoft.UI.Xaml.Controls.Primitives.Popup (MenuFlyout etc.) or
        /// a ContentDialog (Shell.DisplayPromptAsync / DisplayAlertAsync).
        /// These overlay in place without changing Shell.Current.CurrentPage,
        /// so the host-page check on its own wouldn't reject their keys.
        /// </summary>
        private static bool IsInsideOverlay(object? source)
        {
            var element = source as Microsoft.UI.Xaml.DependencyObject;
            while (element is not null)
            {
                if (element is Microsoft.UI.Xaml.Controls.Primitives.Popup
                    or Microsoft.UI.Xaml.Controls.ContentDialog)
                    return true;
                element = Microsoft.UI.Xaml.Media.VisualTreeHelper.GetParent(element);
            }
            return false;
        }
        /// <summary>
        /// Determines the current keyboard press state
        /// </summary>
        /// <returns>Key state</returns>
        static Keyboard.KeyState GetKeyState()
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

        static Keyboard.Key GetKey(VirtualKey virtualKey)
        {
            return (Keyboard.Key)virtualKey;
        }
        /// <summary>
        /// Determines the current press state of a given key
        /// </summary>
        /// <returns>Key state</returns>

        [DllImport("user32.dll")]
        private static extern short GetAsyncKeyState(int vKey);

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
