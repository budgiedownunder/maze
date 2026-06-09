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
    /// or <c>null</c> on Cancel. Pre-fills from <see cref="Play3dCustomLaunchSettingsStore.Load"/>
    /// so the user's previous choices are remembered.
    /// </summary>
    public partial class Play3dCustomLaunchPopup : Popup
    {
        // Display labels for the Sky picker. Index matches SkyTypeValues so
        // SelectedIndex maps to a wire value. `Dungeon` + `Chamber` are the
        // two enclosed-roof variants (dark-rock ceiling / wall-material ceiling); 
        // same set the React modal exposes.
        private static readonly string[] SkyTypeLabels = { "Night", "Sunrise", "Day", "Sunset", "Dungeon", "Chamber" };
        private static readonly string[] SkyTypeValues = { "night", "sunrise", "day", "sunset", "dungeon", "chamber" };

        // Display labels for the Wall texture picker. Same index pairing.
        private static readonly string[] WallTypeLabels = { "Brick", "Dressed Stone", "Wood", "Cobblestone" };
        private static readonly string[] WallTypeValues = { "brick", "dressed_stone", "wood", "cobblestone" };

        // Display labels for the Door style picker. Same index pairing.
        private static readonly string[] DoorStyleLabels = { "Swing", "Slide", "Portcullis", "Dissolve" };
        private static readonly string[] DoorStyleValues = { "swing", "slide", "portcullis", "dissolve" };

        // Display labels for the Key holder picker. Same index pairing.
        private static readonly string[] KeyHolderLabels = { "Pedestal", "Chest", "Floating Key" };
        private static readonly string[] KeyHolderValues = { "pedestal", "chest", "floating_key" };

        // Display labels for the Enemy type picker. Same index pairing.
        private static readonly string[] EnemyTypeLabels = { "Goblin", "Ghost" };
        private static readonly string[] EnemyTypeValues = { "goblin", "ghost" };

        // Display labels for the Health style picker. Same index pairing.
        private static readonly string[] HealthStyleLabels = { "Heart", "Potion" };
        private static readonly string[] HealthStyleValues = { "heart", "potion" };

        // The user's perimeter-walls preference. Enclosed skies (dungeon / chamber) always
        // wall the perimeter, so the checkbox is forced on + disabled for them while this
        // preference is preserved and re-applied when an open sky is chosen again — matching
        // the React modal, which forces the box visually but still emits the stored value.
        private bool _perimeterWallsPreference = true;

        /// <summary>
        /// Constructor. Pre-fills the form from the previously-saved settings.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title (wraps in the header if it's long)</param>
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
            foreach (var label in DoorStyleLabels) DoorStylePicker.Items.Add(label);
            foreach (var label in KeyHolderLabels) KeyHolderPicker.Items.Add(label);
            foreach (var label in EnemyTypeLabels) EnemyTypePicker.Items.Add(label);
            foreach (var label in HealthStyleLabels) HealthStylePicker.Items.Add(label);

            // Pre-fill from saved settings.
            var settings = Play3dCustomLaunchSettingsStore.Load();
            SkyPicker.SelectedIndex = IndexOf(SkyTypeValues, settings.SkyType);
            WallTexturePicker.SelectedIndex = IndexOf(WallTypeValues, settings.WallType);
            DoorStylePicker.SelectedIndex = IndexOf(DoorStyleValues, settings.DoorStyle);
            KeyHolderPicker.SelectedIndex = IndexOf(KeyHolderValues, settings.KeyHolder);
            EnemyTypePicker.SelectedIndex = IndexOf(EnemyTypeValues, settings.EnemyType);
            HealthStylePicker.SelectedIndex = IndexOf(HealthStyleValues, settings.HealthStyle);
            QuadrantWallTypesCheck.IsChecked = settings.WallMaterialVariation;
            WallTintCheck.IsChecked = settings.WallTint;
            DeadEndObjectsCheck.IsChecked = settings.DeadEndObjects;
            WallDecorationsCheck.IsChecked = settings.WallDecorations;
            FloorAccentsCheck.IsChecked = settings.FloorAccents;
            _perimeterWallsPreference = settings.PerimeterWalls;
            TimerEntry.Text = settings.TimerSeconds.ToString(CultureInfo.InvariantCulture);

            // Apply the initial enabled/disabled state for wall texture +
            // wall tint based on the quadrant-variation checkbox.
            UpdateWallControlsEnabled();

            // Couple perimeter walls to the sky (enclosed skies force it on). Wired after
            // the prefill above so the initial picker/preference assignments don't fire the
            // handlers; then apply the initial perimeter-walls state.
            SkyPicker.SelectedIndexChanged += OnSkyTypeChanged;
            PerimeterWallsCheck.CheckedChanged += OnPerimeterWallsChanged;
            UpdatePerimeterWallsState();

            // Start on the Scene tab (its panel is visible in XAML; this sets
            // the matching tab-button highlight).
            SelectTab(LaunchTab.Scene);

            // Cap the popup body's maximum height to (almost) the
            // host window height. Combined with the Grid's middle * row,
            // this lets the inner ScrollView shrink while keeping
            // the pinned title + Cancel/Play row visible on a short
            // window. On a tall window the cap is well above the
            // natural content height, so the Border stays
            // content-sized — popup doesn't inflate. Re-applied on
            // window resize; unsubscribed when the popup closes.
            Loaded += OnPopupLoaded;
            Closed += OnPopupClosed;

#if WINDOWS
            Loaded += OnLoadedWindows;
#endif
        }

        // Margin between the host window height and the popup Border
        // max height — leaves room for the OS title bar + popup chrome
        // + a few px of breathing space so the popup never sits flush
        // against the window edge. Conservative on the high side so
        // the pinned button row never clips on any platform.
        private const double PopupVerticalMarginPx = 80;
        // Floor for the popup Border height — well below any plausible
        // popup with title + a couple of form rows + buttons so the
        // popup never collapses to unusable on extreme small windows.
        private const double MinPopupHeightPx = 240;

        private Microsoft.Maui.Controls.Window? _trackedWindow;

        private void OnPopupLoaded(object? sender, EventArgs e)
        {
            UpdateRootBorderMaxHeight();
            _trackedWindow = (Application.Current?.Windows.Count > 0 ? Application.Current.Windows[0] : null);
            if (_trackedWindow is { } w)
                w.SizeChanged += OnWindowSizeChanged;
        }

        private void OnPopupClosed(object? sender, EventArgs e)
        {
            if (_trackedWindow is { } w)
            {
                w.SizeChanged -= OnWindowSizeChanged;
                _trackedWindow = null;
            }
        }

        private void OnWindowSizeChanged(object? sender, EventArgs e) => UpdateRootBorderMaxHeight();

        private void UpdateRootBorderMaxHeight()
        {
            var window = (Application.Current?.Windows.Count > 0 ? Application.Current.Windows[0] : null);
            if (window is null) return;
            double available = Math.Max(MinPopupHeightPx, window.Height - PopupVerticalMarginPx);
            RootBorder.MaximumHeightRequest = available;
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

        // Identifies which group of launch settings is currently shown. The
        // fields are split across these tabs so the popup reads as a few short
        // panels rather than one long scrolling list.
        private enum LaunchTab { Scene, Objects, Decor }

        private void OnSceneTabClicked(object sender, EventArgs e) => SelectTab(LaunchTab.Scene);
        private void OnObjectsTabClicked(object sender, EventArgs e) => SelectTab(LaunchTab.Objects);
        private void OnDecorTabClicked(object sender, EventArgs e) => SelectTab(LaunchTab.Decor);

        /// <summary>
        /// Shows the chosen tab's panel (hiding the others) and updates the tab
        /// buttons so the active one is highlighted. All controls stay in the
        /// visual tree regardless of the active tab, so prefill and the Play
        /// read-back are unaffected by which tab is showing.
        /// </summary>
        private void SelectTab(LaunchTab tab)
        {
            SetPanelActive(SceneTab, tab == LaunchTab.Scene);
            SetPanelActive(ObjectsTab, tab == LaunchTab.Objects);
            SetPanelActive(DecorTab, tab == LaunchTab.Decor);

            ApplyTabButtonStyle(SceneTabButton, SceneTabUnderline, tab == LaunchTab.Scene);
            ApplyTabButtonStyle(ObjectsTabButton, ObjectsTabUnderline, tab == LaunchTab.Objects);
            ApplyTabButtonStyle(DecorTabButton, DecorTabUnderline, tab == LaunchTab.Decor);
        }

        // Show + enable only the active tab panel, but keep every panel measured
        // so the content area always sizes to the largest tab. Collapsing inactive
        // panels via IsVisible would make the popup resize between tabs and centre
        // narrower panels (the form is centred, so a narrow panel floats mid-width);
        // Opacity 0 + InputTransparent hides and disables them while preserving their
        // footprint in the layout.
        private static void SetPanelActive(View panel, bool active)
        {
            panel.Opacity = active ? 1.0 : 0.0;
            panel.InputTransparent = !active;
        }

        // Highlight the selected tab with a bold label, full opacity and a
        // visible accent underline; dim the rest and hide their underlines.
        // The bold/opacity cues plus the themed underline colour read correctly
        // under both light and dark themes.
        private static void ApplyTabButtonStyle(Button button, BoxView underline, bool selected)
        {
            button.FontAttributes = selected ? FontAttributes.Bold : FontAttributes.None;
            button.Opacity = selected ? 1.0 : 0.6;
            underline.IsVisible = selected;
        }

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
        /// Fires when the Sky picker changes — re-applies the perimeter-walls coupling
        /// (enclosed skies always wall the perimeter).
        /// </summary>
        private void OnSkyTypeChanged(object? sender, EventArgs e) => UpdatePerimeterWallsState();

        /// <summary>
        /// Captures a genuine user toggle of the perimeter-walls checkbox. The control is
        /// disabled (and its checked state forced) under an enclosed sky, so we only record
        /// the preference when it's enabled — keeping it across sky changes.
        /// </summary>
        private void OnPerimeterWallsChanged(object? sender, CheckedChangedEventArgs e)
        {
            if (PerimeterWallsCheck.IsEnabled)
            {
                _perimeterWallsPreference = PerimeterWallsCheck.IsChecked;
            }
        }

        private void UpdatePerimeterWallsState()
        {
            bool skyEnclosed = SkyPicker.SelectedIndex >= 0
                && SkyTypeValues[SkyPicker.SelectedIndex] is "dungeon" or "chamber";
            // Set Enabled before Checked so OnPerimeterWallsChanged sees the right state.
            PerimeterWallsCheck.IsEnabled = !skyEnclosed;
            // Enclosed skies always wall the perimeter — show it on; otherwise reflect the
            // stored preference.
            PerimeterWallsCheck.IsChecked = skyEnclosed || _perimeterWallsPreference;
        }

        /// <summary>
        /// Handles the Play button click. Validates the time limit (must be
        /// &gt; 0) and on success closes the popup with the chosen settings,
        /// having also persisted them via
        /// <see cref="Play3dCustomLaunchSettingsStore.Save"/>.
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
            var doorStyle = DoorStylePicker.SelectedIndex >= 0 ? DoorStyleValues[DoorStylePicker.SelectedIndex] : "swing";
            var keyHolder = KeyHolderPicker.SelectedIndex >= 0 ? KeyHolderValues[KeyHolderPicker.SelectedIndex] : "pedestal";
            var enemyType = EnemyTypePicker.SelectedIndex >= 0 ? EnemyTypeValues[EnemyTypePicker.SelectedIndex] : "goblin";
            var healthStyle = HealthStylePicker.SelectedIndex >= 0 ? HealthStyleValues[HealthStylePicker.SelectedIndex] : "heart";

            var settings = new Play3dCustomLaunchSettings
            {
                SkyType = sky,
                WallType = wall,
                // Emit the stored preference, not the (possibly sky-forced) checkbox — the
                // game walls an enclosed-sky perimeter regardless, matching the React modal.
                PerimeterWalls = _perimeterWallsPreference,
                DoorStyle = doorStyle,
                KeyHolder = keyHolder,
                EnemyType = enemyType,
                HealthStyle = healthStyle,
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
