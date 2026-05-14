
namespace Maze.Maui.App.Views
{
    using CommunityToolkit.Maui.Extensions;
    using CommunityToolkit.Maui.Views;
    using Maze.Maui.App.Models;
#if WINDOWS
    using System.Runtime.InteropServices;
#endif

    /// <summary>
    /// A popup that prompts the user to choose a Play 3D difficulty
    /// (Easy / Tricky / Hard, defaulting to Tricky). Returns the chosen
    /// <see cref="Difficulty"/> on Play, or <c>null</c> on Cancel.
    ///
    /// Display-only — it does not know the per-difficulty maze size / timer /
    /// seed; the server resolves those when <c>/game/?difficulty=…</c> loads.
    /// Mirrors the React <c>Play3dDifficultyModal</c> used by the web front end.
    /// </summary>
    public partial class Play3dDifficultyPopup : Popup
    {
        /// <summary>
        /// Constructor
        /// </summary>
        public Play3dDifficultyPopup()
        {
            InitializeComponent();

            // Land focus on the default (Tricky) option. Deferred to the
            // dispatcher so the native handlers are attached by the time
            // Focus() runs.
            Opened += (s, e) => Dispatcher.Dispatch(() => TrickyRadio.Focus());

#if WINDOWS
            // Trap Tab / Shift+Tab so focus cycles inside the popup. CT.Maui v13
            // hosts this Popup as a Shell-navigated PopupPage rather than a
            // focus-trapping native dialog, so without this Tab leaks into the
            // page underneath. Mirrors GenerateMazePopup.
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
            if (!shift && PlayButton.IsFocused)
            {
                EasyRadio.Focus();
                e.Handled = true;
            }
            else if (shift && EasyRadio.IsFocused)
            {
                PlayButton.Focus();
                e.Handled = true;
            }
        }

        [DllImport("user32.dll")]
        private static extern short GetAsyncKeyState(int vKey);
        private const int VK_SHIFT = 0x10;
#endif

        /// <summary>
        /// Handles the Play button click. Closes the popup with the chosen
        /// difficulty.
        /// </summary>
        private async void OnPlayClicked(object sender, EventArgs e)
        {
            Difficulty chosen = EasyRadio.IsChecked ? Difficulty.Easy
                : HardRadio.IsChecked ? Difficulty.Hard
                : Difficulty.Tricky;
            await Navigation.ClosePopupAsync<Difficulty?>(chosen);
        }

        /// <summary>
        /// Handles the Cancel button click. Closes the popup with no selection.
        /// </summary>
        private async void OnCancelClicked(object sender, EventArgs e)
        {
            await Navigation.ClosePopupAsync<Difficulty?>(null);
        }
    }
}
