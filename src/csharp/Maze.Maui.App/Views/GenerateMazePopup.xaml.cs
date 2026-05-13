
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
        /// <param name="maxMazeCells">Server-reported cell-count cap (<c>AppFeatures.MaxMazeCells</c>); <c>null</c> means no cap</param>
        /// <param name="generationError">Optional error message from a previous generation attempt, displayed inline</param>
        public GenerateMazePopup(uint rows, uint cols,
            uint startRow, uint startCol, uint finishRow, uint finishCol,
            uint minSolutionLength, int? maxMazeCells = null, string? generationError = null)
        {
            InitializeComponent();
            _maxMazeCells = maxMazeCells;

            RowsEntry.Text = rows.ToString();
            ColsEntry.Text = cols.ToString();

            // Display start/finish as 1-based
            StartRowEntry.Text = (startRow + 1).ToString();
            StartColEntry.Text = (startCol + 1).ToString();
            FinishRowEntry.Text = (finishRow + 1).ToString();
            FinishColEntry.Text = (finishCol + 1).ToString();

            MinSolutionLengthEntry.Text = minSolutionLength.ToString();

            if (generationError is not null)
            {
                ErrorLabel.Text = $"Generation failed: {generationError}";
                ErrorLabel.IsVisible = true;
            }

            // Land focus on the first edit field. Deferred to the dispatcher so
            // the native handlers are attached by the time Focus() runs.
            Opened += (s, e) => Dispatcher.Dispatch(() => RowsEntry.Focus());

#if WINDOWS
            // Trap Tab / Shift+Tab so focus cycles inside the popup. CT.Maui v13
            // hosts this Popup as a Shell-navigated PopupPage rather than a
            // focus-trapping native dialog, so without this Tab leaks into the
            // toolbar / page underneath.
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
            };
            return true;
        }
    }
}
