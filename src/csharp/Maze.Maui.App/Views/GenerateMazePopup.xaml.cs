
namespace Maze.Maui.App.Views
{
    using CommunityToolkit.Maui.Extensions;
    using CommunityToolkit.Maui.Views;
    using Maze.Api;

    /// <summary>
    /// A popup that prompts the user for maze generation options.
    /// Start and finish cell values are displayed and entered as 1-based (row 1 = top row).
    /// Returns a <see cref="Maze.GenerationOptions"/> on confirmation (with Seed set to 0 as a
    /// placeholder — the caller is responsible for assigning the final seed), or <c>null</c> on cancel.
    /// </summary>
    public partial class GenerateMazePopup : Popup
    {
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
        /// <param name="generationError">Optional error message from a previous generation attempt, displayed inline</param>
        public GenerateMazePopup(uint rows, uint cols,
            uint startRow, uint startCol, uint finishRow, uint finishCol,
            uint minSolutionLength, string? generationError = null)
        {
            InitializeComponent();

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
        }

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
        /// Start/finish entries are 1-based and converted to 0-based for the API.
        /// </summary>
        /// <param name="options">The parsed options on success (Seed is set to 0; caller must assign)</param>
        /// <param name="error">An error message on failure</param>
        /// <returns>True if valid, false otherwise</returns>
        private bool TryParseOptions(out Maze.GenerationOptions? options, out string error)
        {
            options = null;

            if (!uint.TryParse(RowsEntry.Text?.Trim(), out uint rows) || rows < 3)
            { error = "Rows must be a whole number of 3 or more."; return false; }

            if (!uint.TryParse(ColsEntry.Text?.Trim(), out uint cols) || cols < 3)
            { error = "Columns must be a whole number of 3 or more."; return false; }

            // Start/finish are entered 1-based: valid range is [1, rows] and [1, cols]
            if (!uint.TryParse(StartRowEntry.Text?.Trim(), out uint startRow1) || startRow1 < 1 || startRow1 > rows)
            { error = $"Start Row must be between 1 and {rows}."; return false; }

            if (!uint.TryParse(StartColEntry.Text?.Trim(), out uint startCol1) || startCol1 < 1 || startCol1 > cols)
            { error = $"Start Column must be between 1 and {cols}."; return false; }

            if (!uint.TryParse(FinishRowEntry.Text?.Trim(), out uint finishRow1) || finishRow1 < 1 || finishRow1 > rows)
            { error = $"Finish Row must be between 1 and {rows}."; return false; }

            if (!uint.TryParse(FinishColEntry.Text?.Trim(), out uint finishCol1) || finishCol1 < 1 || finishCol1 > cols)
            { error = $"Finish Column must be between 1 and {cols}."; return false; }

            // Convert to 0-based for the API
            uint startRow = startRow1 - 1;
            uint startCol = startCol1 - 1;
            uint finishRow = finishRow1 - 1;
            uint finishCol = finishCol1 - 1;

            if (startRow == finishRow && startCol == finishCol)
            { error = "Start and Finish cells must be different."; return false; }

            if (!uint.TryParse(MinSolutionLengthEntry.Text?.Trim(), out uint minSolutionLength) || minSolutionLength < 1)
            { error = "Min Solution Length must be a whole number of 1 or more."; return false; }

            error = string.Empty;
            options = new Maze.GenerationOptions
            {
                RowCount = rows,
                ColCount = cols,
                Seed = 0, // placeholder — caller assigns the final seed
                StartRow = startRow,
                StartCol = startCol,
                FinishRow = finishRow,
                FinishCol = finishCol,
                MinSpineLength = minSolutionLength,
            };
            return true;
        }
    }
}
