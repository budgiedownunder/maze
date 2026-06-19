
namespace Maze.Maui.App.Views
{
    using CommunityToolkit.Maui.Extensions;
    using CommunityToolkit.Maui.Views;
    using Maze.Api;
    using Maze.Maui.App.Utils;
#if WINDOWS
    using System.Runtime.InteropServices;
#endif

    /// <summary>
    /// A popup that prompts the user for maze generation options.
    /// Start and finish cell values are displayed and entered as 1-based (row 1 = top row).
    /// Returns a <see cref="Maze.GenerationOptions"/> on confirmation (with Seed set to 0 as a
    /// placeholder — the caller is responsible for assigning the final seed), or <c>null</c> on cancel.
    /// This is how the page appears on Windows Desktop:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Windows</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/windows-generate-maze.png" height="500" width="500"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    ///  
    /// and this is how it appears on Android/iOS devices:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Android</strong></th>
    ///         <th><strong>iOS</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/android-generate-maze.png" width="250"/></td>
    ///         <td><img src="../../images/screenshots/ios-generate-maze.png" width="250"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    /// </summary>
    public partial class GenerateMazePopup : Popup
    {
        private readonly int? _maxMazeCells;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="rows">Default row count</param>
        /// <param name="cols">Default column count</param>
        /// <param name="startRow">Default start cell row (0-based)</param>
        /// <param name="startCol">Default start cell column (0-based)</param>
        /// <param name="finishRow">Default finish cell row (0-based)</param>
        /// <param name="finishCol">Default finish cell column (0-based)</param>
        /// <param name="minSolutionLength">Default minimum solution length</param>
        /// <param name="doorCount">Default number of real path doors to auto-place (0 = none)</param>
        /// <param name="spareDoors">Default number of decoy doors to plant (0 = none)</param>
        /// <param name="spareKeys">Default number of spare keys to plant (0 = none)</param>
        /// <param name="enemyCount">Default number of enemies to auto-place (0 = none)</param>
        /// <param name="healthCount">Default number of health pickups to auto-place (0 = none)</param>
        /// <param name="treasureCount">Default number of treasure cells to auto-place (0 = none)</param>
        /// <param name="maxMazeCells">Server-reported cell-count cap (<c>AppFeatures.MaxMazeCells</c>); <c>null</c> means no cap</param>
        /// <param name="generationError">Optional error message from a previous generation attempt, displayed inline</param>
        public GenerateMazePopup(uint rows, uint cols,
            uint startRow, uint startCol, uint finishRow, uint finishCol,
            uint minSolutionLength,
            uint doorCount = 0, uint spareDoors = 0, uint spareKeys = 0,
            uint enemyCount = 0, uint healthCount = 0, uint treasureCount = 0,
            int? maxMazeCells = null, string? generationError = null)
        {
            InitializeComponent();
            _maxMazeCells = maxMazeCells;

            RowsEntry.Text = rows.ToString();
            ColsEntry.Text = cols.ToString();
            // Seed the clamp baselines so an Unfocused with no edit is a no-op.
            _lastClampedRows = RowsEntry.Text;
            _lastClampedCols = ColsEntry.Text;

            // Display start/finish as 1-based
            StartRowEntry.Text = (startRow + 1).ToString();
            StartColEntry.Text = (startCol + 1).ToString();
            FinishRowEntry.Text = (finishRow + 1).ToString();
            FinishColEntry.Text = (finishCol + 1).ToString();

            MinSolutionLengthEntry.Text = minSolutionLength.ToString();

            DoorCountEntry.Text = doorCount.ToString();
            SpareDoorsEntry.Text = spareDoors.ToString();
            SpareKeysEntry.Text = spareKeys.ToString();

            EnemyCountEntry.Text = enemyCount.ToString();
            HealthCountEntry.Text = healthCount.ToString();
            TreasureCountEntry.Text = treasureCount.ToString();

            if (generationError is not null)
            {
                ErrorLabel.Text = $"Generation failed: {generationError}";
                ErrorLabel.IsVisible = true;
            }

            // Start on the Size tab (its panel is visible in XAML; this sets the
            // matching tab-button highlight).
            SelectTab(GenerateTab.Size);

            // Changing the grid dimensions resets start/finish to the usual
            // top-left / bottom-right; only arm this after the prefill above so
            // construction doesn't clobber the seeded start/finish positions.
            _initialized = true;

            // Land focus on the first edit field. Deferred to the dispatcher so
            // the native handlers are attached by the time Focus() runs.
            Opened += (s, e) => Dispatcher.Dispatch(() => RowsEntry.Focus());

            // Cap the popup body's maximum height to (almost) the host
            // window height. Combined with the Grid's middle * row, this
            // lets the inner ScrollView shrink while keeping the pinned
            // title + Cancel/Generate row visible on a short window. On
            // a tall window the cap is well above the natural content
            // height, so the Border stays content-sized — popup doesn't
            // inflate. Re-applied on window resize; unsubscribed when
            // the popup closes. Same approach as MazeGameSettingsPopup.
            Loaded += OnPopupLoaded;
            Closed += OnPopupClosed;

#if WINDOWS
            // Trap Tab / Shift+Tab so focus cycles inside the popup. CT.Maui v13
            // hosts this Popup as a Shell-navigated PopupPage rather than a
            // focus-trapping native dialog, so without this Tab leaks into the
            // toolbar / page underneath.
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
            if (e.Key != Windows.System.VirtualKey.Tab) return;
            bool shift = (GetAsyncKeyState(VK_SHIFT) & 0x8000) != 0;
            if (!shift && GenerateButton.IsFocused)
            {
                RowsEntry.Focus();
                e.Handled = true;
            }
            else if (shift && RowsEntry.IsFocused)
            {
                GenerateButton.Focus();
                e.Handled = true;
            }
        }

        [DllImport("user32.dll")]
        private static extern short GetAsyncKeyState(int vKey);
        private const int VK_SHIFT = 0x10;
#endif

        // Identifies which group of generate fields is currently shown. The
        // fields are split across these tabs so the popup reads as a few short
        // panels rather than one long scrolling list.
        private enum GenerateTab { Size, Features }

        private void OnSizeTabClicked(object sender, EventArgs e) => SelectTab(GenerateTab.Size);
        private void OnFeaturesTabClicked(object sender, EventArgs e) => SelectTab(GenerateTab.Features);

        /// <summary>
        /// Shows the chosen tab's panel (hiding the other) and updates the tab
        /// buttons so the active one is highlighted. All controls stay in the
        /// visual tree regardless of the active tab, so prefill and the Generate
        /// read-back are unaffected by which tab is showing.
        /// </summary>
        private void SelectTab(GenerateTab tab)
        {
            SetPanelActive(SizeTab, tab == GenerateTab.Size);
            SetPanelActive(FeaturesTab, tab == GenerateTab.Features);

            ApplyTabButtonStyle(SizeTabButton, SizeTabUnderline, tab == GenerateTab.Size);
            ApplyTabButtonStyle(FeaturesTabButton, FeaturesTabUnderline, tab == GenerateTab.Features);
        }

        // Show + enable only the active tab panel, but keep every panel measured so
        // the content area always sizes to the largest tab — the popup doesn't
        // resize between tabs. Collapsing inactive panels via IsVisible would drop
        // them from the layout, so the popup would shrink/grow per tab. Opacity 0 +
        // InputTransparent hides and disables them while preserving their footprint.
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

        // True once the constructor's prefill is complete, so the
        // dimension-change reset handlers below don't fire during construction
        // (which would overwrite the seeded start/finish positions).
        private readonly bool _initialized;

        // Last dimension values the clamp ran against — so an Unfocused that
        // didn't actually change Rows/Columns is a no-op (we must not "fix" a
        // deliberately out-of-range start/finish the user is about to submit;
        // that's the parser's job). Seeded from the constructor's prefill.
        private string _lastClampedRows = string.Empty;
        private string _lastClampedCols = string.Empty;

        // Re-clamping start/finish to the new bounds runs only when the user
        // commits an actual dimension change — Unfocused (tab/click away) or
        // Completed (Enter) — not on every keystroke. Clamping per-keystroke
        // would snap against the intermediate value (e.g. the "1" of a "15"
        // being typed) before the edit is finished. A committed change nudges a
        // start/finish coordinate only if it would now fall outside the new
        // bounds (start→top/left corner, finish→new far edge); in-range
        // coordinates are left exactly as the author set them. Entries are
        // 1-based.
        private void OnRowsCommitted(object sender, EventArgs e)
        {
            if (RowsEntry.Text == _lastClampedRows) return;
            _lastClampedRows = RowsEntry.Text;
            ClampStartFinish(RowsEntry.Text, StartRowEntry, FinishRowEntry);
        }

        private void OnColsCommitted(object sender, EventArgs e)
        {
            if (ColsEntry.Text == _lastClampedCols) return;
            _lastClampedCols = ColsEntry.Text;
            ClampStartFinish(ColsEntry.Text, StartColEntry, FinishColEntry);
        }

        private void ClampStartFinish(string dimensionText, Entry startEntry, Entry finishEntry)
        {
            if (!_initialized) return;
            if (!int.TryParse(dimensionText, out int max) || max < 1) return;
            if (!InRange(startEntry.Text, max)) startEntry.Text = "1";
            if (!InRange(finishEntry.Text, max)) finishEntry.Text = max.ToString();
        }

        // True when a 1-based coordinate string sits within 1..max (inclusive).
        private static bool InRange(string coordText, int max) =>
            int.TryParse(coordText, out int n) && n >= 1 && n <= max;

        /// <summary>
        /// Handles the Generate button click. Validates inputs and closes the popup with the
        /// generation options on success, or shows an inline error on failure.
        /// </summary>
        private async void OnGenerateClicked(object sender, EventArgs e)
        {
            if (!TryParseOptions(out var options, out string error))
            {
                ErrorLabel.Text = error;
                ErrorLabel.IsVisible = true;
                return;
            }

            await Navigation.ClosePopupAsync<Maze.GenerationOptions?>(options);
        }

        /// <summary>
        /// Handles the Cancel button click.
        /// </summary>
        private async void OnCancelClicked(object sender, EventArgs e)
        {
            await Navigation.ClosePopupAsync<Maze.GenerationOptions?>(null);
        }

        /// <summary>
        /// Parses and validates the form entries into a <see cref="Maze.GenerationOptions"/> instance.
        /// Delegates to <see cref="GenerateMazeOptionsParser.TryParse"/> for the validation chain
        /// — see that method for the per-field rules. Start/finish entries are 1-based as entered
        /// and emitted 0-based on the returned options.
        /// </summary>
        /// <param name="options">The parsed options on success (Seed is set to 0; caller must assign)</param>
        /// <param name="error">An error message on failure</param>
        /// <returns>True if valid, false otherwise</returns>
        private bool TryParseOptions(out Maze.GenerationOptions? options, out string error)
        {
            options = null;

            if (!GenerateMazeOptionsParser.TryParse(
                rowsText: RowsEntry.Text,
                colsText: ColsEntry.Text,
                startRowText: StartRowEntry.Text,
                startColText: StartColEntry.Text,
                finishRowText: FinishRowEntry.Text,
                finishColText: FinishColEntry.Text,
                minSolutionLengthText: MinSolutionLengthEntry.Text,
                doorCountText: DoorCountEntry.Text,
                spareDoorsText: SpareDoorsEntry.Text,
                spareKeysText: SpareKeysEntry.Text,
                enemyCountText: EnemyCountEntry.Text,
                healthCountText: HealthCountEntry.Text,
                treasureCountText: TreasureCountEntry.Text,
                maxMazeCells: _maxMazeCells,
                out var parsed,
                out error))
            {
                return false;
            }

            options = new Maze.GenerationOptions
            {
                RowCount = parsed!.Rows,
                ColCount = parsed.Cols,
                Seed = 0, // placeholder — caller assigns the final seed
                StartRow = parsed.StartRow,
                StartCol = parsed.StartCol,
                FinishRow = parsed.FinishRow,
                FinishCol = parsed.FinishCol,
                MinSpineLength = parsed.MinSolutionLength,
                DoorCount = parsed.DoorCount,
                SpareDoors = parsed.SpareDoors,
                SpareKeys = parsed.SpareKeys,
                EnemyCount = parsed.EnemyCount,
                HealthCount = parsed.HealthCount,
                TreasureCount = parsed.TreasureCount,
            };
            return true;
        }
    }
}
