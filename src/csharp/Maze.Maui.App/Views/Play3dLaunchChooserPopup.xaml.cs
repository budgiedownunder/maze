namespace Maze.Maui.App.Views
{
    using CommunityToolkit.Maui.Extensions;
    using CommunityToolkit.Maui.Views;
    using Maze.Maui.App.Services;
#if WINDOWS
    using System.Runtime.InteropServices;
#endif

    /// <summary>
    /// Asks how to launch a 3D play of a user-edited maze: <c>Run</c> (use the
    /// maze's saved settings), <c>Custom Run…</c> (open the settings popup for a
    /// one-off launch), or <c>Cancel</c>. Mirrors the React SPA's
    /// <c>Play3dLaunchChooser</c>. Vertical full-width buttons are a deliberate
    /// touch target — a caret/popover menu is hard to tap.
    /// </summary>
    public partial class Play3dLaunchChooserPopup : Popup
    {
        /// <summary>
        /// Constructor.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title</param>
        public Play3dLaunchChooserPopup(string? mazeName = null)
        {
            InitializeComponent();

            if (!string.IsNullOrWhiteSpace(mazeName))
            {
                TitleLabel.Text = $"Play 3D — {mazeName}";
            }

            // Land focus on Run (the default action). Deferred to the dispatcher so
            // the native handlers are attached by the time Focus() runs.
            Opened += (s, e) => Dispatcher.Dispatch(() => RunButton.Focus());

#if WINDOWS
            // Trap Tab / Shift+Tab so focus cycles inside the popup. CT.Maui v13
            // hosts this Popup as a Shell-navigated PopupPage rather than a
            // focus-trapping native dialog, so without this Tab leaks into the
            // page underneath. Mirrors the other popups.
            Loaded += OnLoadedWindows;
#endif
        }

#if WINDOWS
        private void OnLoadedWindows(object? sender, EventArgs e)
        {
            if (Handler?.PlatformView is Microsoft.UI.Xaml.UIElement native)
                native.PreviewKeyDown += OnNativePreviewKeyDown;
        }

        private void OnNativePreviewKeyDown(object sender, Microsoft.UI.Xaml.Input.KeyRoutedEventArgs e)
        {
            if (e.Key != Windows.System.VirtualKey.Tab) return;
            bool shift = (GetAsyncKeyState(VK_SHIFT) & 0x8000) != 0;
            if (!shift && CancelButton.IsFocused)
            {
                RunButton.Focus();
                e.Handled = true;
            }
            else if (shift && RunButton.IsFocused)
            {
                CancelButton.Focus();
                e.Handled = true;
            }
        }

        [DllImport("user32.dll")]
        private static extern short GetAsyncKeyState(int vKey);
        private const int VK_SHIFT = 0x10;
#endif

        /// <summary>Handles the Run button click. Launches with the maze's saved settings.</summary>
        private async void OnRunClicked(object sender, EventArgs e)
            => await Navigation.ClosePopupAsync<Play3dLaunchChoice>(Play3dLaunchChoice.Run);

        /// <summary>Handles the Custom Run button click. Opens the one-off settings popup.</summary>
        private async void OnCustomRunClicked(object sender, EventArgs e)
            => await Navigation.ClosePopupAsync<Play3dLaunchChoice>(Play3dLaunchChoice.CustomRun);

        /// <summary>Handles the Cancel button click. Dismisses without launching.</summary>
        private async void OnCancelClicked(object sender, EventArgs e)
            => await Navigation.ClosePopupAsync<Play3dLaunchChoice>(Play3dLaunchChoice.Cancel);
    }
}
