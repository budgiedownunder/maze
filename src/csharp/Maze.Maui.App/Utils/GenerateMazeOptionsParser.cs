namespace Maze.Maui.App.Utils;

/// <summary>
/// Result of a successful <see cref="GenerateMazeOptionsParser.TryParse"/>.
/// Start/finish are emitted 0-based (the API convention); the parser
/// performs the 1→0 conversion on the user-entered values.
/// </summary>
/// <param name="Rows">Maze row count.</param>
/// <param name="Cols">Maze column count.</param>
/// <param name="StartRow">Start cell row, 0-based.</param>
/// <param name="StartCol">Start cell column, 0-based.</param>
/// <param name="FinishRow">Finish cell row, 0-based.</param>
/// <param name="FinishCol">Finish cell column, 0-based.</param>
/// <param name="MinSolutionLength">Minimum solution length.</param>
internal sealed record ParsedGenerateOptions(
    uint Rows,
    uint Cols,
    uint StartRow,
    uint StartCol,
    uint FinishRow,
    uint FinishCol,
    uint MinSolutionLength);

/// <summary>
/// Parses the string inputs from <see cref="Views.GenerateMazePopup"/>
/// into a validated <see cref="ParsedGenerateOptions"/>. The popup uses
/// this helper so the validation chain — rows/cols, cell-count cap,
/// start/finish range, start ≠ finish, min-solution-length — can be
/// unit-tested without spinning up MAUI.
/// </summary>
internal static class GenerateMazeOptionsParser
{
    /// <summary>
    /// Parses the popup's text inputs. Returns <c>true</c> and assigns
    /// <paramref name="parsed"/> on success; returns <c>false</c> with
    /// the first-failing-rule message in <paramref name="error"/> on
    /// failure.
    /// </summary>
    /// <param name="rowsText">Rows entry text.</param>
    /// <param name="colsText">Columns entry text.</param>
    /// <param name="startRowText">Start row entry text (1-based as entered).</param>
    /// <param name="startColText">Start column entry text (1-based as entered).</param>
    /// <param name="finishRowText">Finish row entry text (1-based as entered).</param>
    /// <param name="finishColText">Finish column entry text (1-based as entered).</param>
    /// <param name="minSolutionLengthText">Min solution length entry text.</param>
    /// <param name="maxMazeCells">Server-reported cell-count cap, or <c>null</c> if no cap.</param>
    /// <param name="parsed">The parsed options on success; <c>null</c> on failure.</param>
    /// <param name="error">An error message on failure; empty string on success.</param>
    internal static bool TryParse(
        string? rowsText,
        string? colsText,
        string? startRowText,
        string? startColText,
        string? finishRowText,
        string? finishColText,
        string? minSolutionLengthText,
        int? maxMazeCells,
        out ParsedGenerateOptions? parsed,
        out string error)
    {
        parsed = null;

        if (!uint.TryParse(rowsText?.Trim(), out uint rows) || rows < 3)
        { error = "Rows must be a whole number of 3 or more."; return false; }

        if (!uint.TryParse(colsText?.Trim(), out uint cols) || cols < 3)
        { error = "Columns must be a whole number of 3 or more."; return false; }

        if (MazeCellCap.Exceeds(rows, cols, maxMazeCells))
        { error = $"Total cells (rows × columns) cannot exceed {maxMazeCells}."; return false; }

        // Start/finish are entered 1-based: valid range is [1, rows] and [1, cols]
        if (!uint.TryParse(startRowText?.Trim(), out uint startRow1) || startRow1 < 1 || startRow1 > rows)
        { error = $"Start Row must be between 1 and {rows}."; return false; }

        if (!uint.TryParse(startColText?.Trim(), out uint startCol1) || startCol1 < 1 || startCol1 > cols)
        { error = $"Start Column must be between 1 and {cols}."; return false; }

        if (!uint.TryParse(finishRowText?.Trim(), out uint finishRow1) || finishRow1 < 1 || finishRow1 > rows)
        { error = $"Finish Row must be between 1 and {rows}."; return false; }

        if (!uint.TryParse(finishColText?.Trim(), out uint finishCol1) || finishCol1 < 1 || finishCol1 > cols)
        { error = $"Finish Column must be between 1 and {cols}."; return false; }

        // Convert to 0-based for the API
        uint startRow = startRow1 - 1;
        uint startCol = startCol1 - 1;
        uint finishRow = finishRow1 - 1;
        uint finishCol = finishCol1 - 1;

        if (startRow == finishRow && startCol == finishCol)
        { error = "Start and Finish cells must be different."; return false; }

        if (!uint.TryParse(minSolutionLengthText?.Trim(), out uint minSolutionLength) || minSolutionLength < 1)
        { error = "Min Solution Length must be a whole number of 1 or more."; return false; }

        error = string.Empty;
        parsed = new ParsedGenerateOptions(
            Rows: rows,
            Cols: cols,
            StartRow: startRow,
            StartCol: startCol,
            FinishRow: finishRow,
            FinishCol: finishCol,
            MinSolutionLength: minSolutionLength);
        return true;
    }
}
