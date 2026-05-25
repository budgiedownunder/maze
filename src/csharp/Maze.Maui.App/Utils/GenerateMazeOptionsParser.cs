// Inside `namespace Maze.Maui.App.Utils`, an unqualified `Maze.X` would
// resolve to the top-level `Maze` namespace, not the `Maze.Api.Maze`
// class. Alias the class at file scope so the constants/helpers below
// read naturally.
using ApiMaze = global::Maze.Api.Maze;

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
/// <param name="DoorCount">Number of real path doors to auto-place (0 = none).</param>
/// <param name="SpareDoors">Number of decoy doors to plant on off-spine branches (0 = none).</param>
/// <param name="SpareKeys">Number of spare keys to plant on off-spine branches (0 = none).</param>
internal sealed record ParsedGenerateOptions(
    uint Rows,
    uint Cols,
    uint StartRow,
    uint StartCol,
    uint FinishRow,
    uint FinishCol,
    uint MinSolutionLength,
    uint DoorCount,
    uint SpareDoors,
    uint SpareKeys);

/// <summary>
/// Parses the string inputs from <see cref="Views.GenerateMazePopup"/>
/// into a validated <see cref="ParsedGenerateOptions"/>. The popup uses
/// this helper so the validation chain — rows/cols, cell-count cap,
/// start/finish range, start ≠ finish, min-solution-length, per-field
/// key/door bounds, and the combined key+door cap — can be unit-tested without
/// spinning up MAUI.
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
    /// <param name="doorCountText">Doors entry text (number of real path doors; 0 = none).</param>
    /// <param name="spareDoorsText">Spare Doors entry text (number of decoy doors; 0 = none).</param>
    /// <param name="spareKeysText">Spare Keys entry text (number of spare keys; 0 = none).</param>
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
        string? doorCountText,
        string? spareDoorsText,
        string? spareKeysText,
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

        // Key/door fields default to 0 when the entry text is null/empty (matches React's defaultsFromGrid).
        if (!TryParseFeatureField(doorCountText, "Doors", out uint doorCount, out error)) return false;
        if (!TryParseFeatureField(spareDoorsText, "Spare Doors", out uint spareDoors, out error)) return false;
        if (!TryParseFeatureField(spareKeysText, "Spare Keys", out uint spareKeys, out error)) return false;

        // Combined key+door cap. Each real door contributes one key AND one door to the
        // grid, so the formula counts doors twice. Mirrors React's
        // exceedsGenerateFeatureCap in utils/validation.ts.
        if (ApiMaze.ExceedsGenerateFeatureCap(doorCount, spareDoors, spareKeys))
        {
            uint total = (2 * doorCount) + spareDoors + spareKeys;
            error =
                $"Total keys + doors ({total}) exceeds the limit of {ApiMaze.MaxTotalFeatures}. " +
                "Each door brings a key, so the count is 2·Doors + Spare Doors + Spare Keys.";
            return false;
        }

        error = string.Empty;
        parsed = new ParsedGenerateOptions(
            Rows: rows,
            Cols: cols,
            StartRow: startRow,
            StartCol: startCol,
            FinishRow: finishRow,
            FinishCol: finishCol,
            MinSolutionLength: minSolutionLength,
            DoorCount: doorCount,
            SpareDoors: spareDoors,
            SpareKeys: spareKeys);
        return true;
    }

    /// <summary>
    /// Parses one of the three key/door number fields against the shared
    /// <see cref="ApiMaze.MaxDoorCount"/> per-field bound. A null/empty entry
    /// is treated as 0 (the React modal seeds spare fields to "0" by
    /// default, but the user can also clear them).
    /// </summary>
    private static bool TryParseFeatureField(string? text, string fieldName, out uint value, out string error)
    {
        string trimmed = text?.Trim() ?? string.Empty;
        if (trimmed.Length == 0)
        {
            value = 0;
            error = string.Empty;
            return true;
        }
        if (!uint.TryParse(trimmed, out value) || value > ApiMaze.MaxDoorCount)
        {
            error = $"{fieldName} must be a whole number between 0 and {ApiMaze.MaxDoorCount}.";
            return false;
        }
        error = string.Empty;
        return true;
    }
}
