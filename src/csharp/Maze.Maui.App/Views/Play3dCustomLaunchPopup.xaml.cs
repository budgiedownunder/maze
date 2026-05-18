using System.Globalization;
using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
#if WINDOWS
using System.Runtime.InteropServices;
#endif

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Native MAUI equivalent of the React SPA's <c>Play3dCustomLaunchModal</c>.
    /// Lets the user customise the per-launch settings (sky, wall texture,
    /// landmark toggles, time limit) for a 3D play of a user-edited maze.
    /// Returns the chosen <see cref="Play3dCustomLaunchSettings"/> on Play,
    /// or <c>null</c> on Cancel. Pre-fills from <see cref="Play3dCustomLaunchSettings.Load"/>
    /// so the user's previous choices are remembered.
    /// </summary>
    public partial class Play3dCustomLaunchPopup : Popup
    {
        // Display labels for the Sky picker. Index matches SkyTypeValues so
        // SelectedIndex maps to a wire value.
        private static readonly string[] SkyTypeLabels = { "Night", "Sunrise", "Day", "Sunset" };
        private static readonly string[] SkyTypeValues = { "night", "sunrise", "day", "sunset" };

        // Display labels for the Wall texture picker. Same index pairing.
        private static readonly string[] WallTypeLabels = { "Brick", "Dressed Stone", "Wood", "Cobblestone" };
        private static readonly string[] WallTypeValues = { "brick", "dressed_stone", "wood", "cobblestone" };

        /// <summary>
        /// Constructor. Pre-fills the form from the previously-saved settings.
        /// </summary>
        /// <param name="mazeName">Maze name to show in the popup title (truncated to fit)</param>
        public Play3dCustomLaunchPopup(string? mazeName = null)
        {
            InitializeComponent();

            if (!string.IsNullOrWhiteSpace(mazeName))
            {
                TitleLabel.Text = $"Play 3D — {mazeName}";
            }

            // Populate pickers.
            foreach (var label in SkyTypeLabels) SkyPicker.Items.Add(label);
            foreach (var label in WallTypeLabels) WallTexturePicker.Items.Add(label);

            // Pre-fill from saved settings.
            var settings = Play3dCustomLaunchSettingsStore.Load();
            SkyPicker.SelectedIndex = IndexOf(SkyTypeValues, settings.SkyType);
            WallTexturePicker.SelectedIndex = IndexOf(WallTypeValues, settings.WallType);
            QuadrantWallTypesCheck.IsChecked = settings.WallMaterialVariation;
            WallTintCheck.IsChecked = settings.WallTint;
            DeadEndObjectsCheck.IsChecked = settings.DeadEndObjects;
            WallDecorationsCheck.IsChecked = settings.WallDecorations;
            FloorAccentsCheck.IsChecked = settings.FloorAccents;
            TimerEntry.Text = settings.TimerSeconds.ToString(CultureInfo.InvariantCulture);

            // Apply the initial enabled/disabled state for wall texture +
            // wall tint based on the quadrant-variation checkbox.
            UpdateWallControlsEnabled();

#if WINDOWS
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
            // Trap Tab / Shift+Tab so focus cycles inside the popup. CT.Maui v13
            // hosts this Popup as a Shell-navigated PopupPage rather than a
            // focus-trapping native dialog, so without this Tab leaks into the
            // page underneath. Mirrors the same wiring in GenerateMazePopup.
            if (e.Key != Windows.System.VirtualKey.Tab) return;
            bool shift = (GetAsyncKeyState(VK_SHIFT) & 0x8000) != 0;
            if (!shift && PlayButton.IsFocused)
            {
                SkyPicker.Focus();
                e.Handled = true;
            }
            else if (shift && SkyPicker.IsFocused)
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
        /// Fires when the Quadrant wall types checkbox changes. Disables the
        /// wall texture picker AND the wall tint checkbox while quadrant
        /// variation is on — those two settings have no effect when material
        /// variation supersedes the per-cell tinted path.
        /// </summary>
        private void OnQuadrantWallTypesChanged(object sender, CheckedChangedEventArgs e)
        {
            UpdateWallControlsEnabled();
        }

        private void UpdateWallControlsEnabled()
        {
            bool variation = QuadrantWallTypesCheck.IsChecked;
            WallTexturePicker.IsEnabled = !variation;
            WallTextureLabel.Opacity = variation ? 0.5 : 1.0;
            WallTintCheck.IsEnabled = !variation;
            WallTintLabel.Opacity = variation ? 0.5 : 1.0;
        }

        /// <summary>
        /// Handles the Play button click. Validates the time limit (must be
        /// &gt; 0) and on success closes the popup with the chosen settings,
        /// having also persisted them via
        /// <see cref="Play3dCustomLaunchSettings.Save"/>.
        /// </summary>
        private async void OnPlayClicked(object sender, EventArgs e)
        {
            if (!int.TryParse(TimerEntry.Text, NumberStyles.Integer, CultureInfo.InvariantCulture, out int timer) || timer <= 0)
            {
                ErrorLabel.Text = "Time limit must be a positive number of seconds.";
                ErrorLabel.IsVisible = true;
                return;
            }

            var sky = SkyPicker.SelectedIndex >= 0 ? SkyTypeValues[SkyPicker.SelectedIndex] : "night";
            var wall = WallTexturePicker.SelectedIndex >= 0 ? WallTypeValues[WallTexturePicker.SelectedIndex] : "brick";

            var settings = new Play3dCustomLaunchSettings
            {
                SkyType = sky,
                WallType = wall,
                WallTint = WallTintCheck.IsChecked,
                WallMaterialVariation = QuadrantWallTypesCheck.IsChecked,
                DeadEndObjects = DeadEndObjectsCheck.IsChecked,
                WallDecorations = WallDecorationsCheck.IsChecked,
                FloorAccents = FloorAccentsCheck.IsChecked,
                TimerSeconds = timer,
            };
            Play3dCustomLaunchSettingsStore.Save(settings);
            await Navigation.ClosePopupAsync<Play3dCustomLaunchSettings?>(settings);
        }

        /// <summary>
        /// Handles the Cancel button click. Closes the popup with no selection.
        /// </summary>
        private async void OnCancelClicked(object sender, EventArgs e)
        {
            await Navigation.ClosePopupAsync<Play3dCustomLaunchSettings?>(null);
        }

        private static int IndexOf(string[] values, string target)
        {
            for (int i = 0; i < values.Length; i++)
            {
                if (values[i] == target) return i;
            }
            return 0;
        }
    }
}
