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
/// <param name="EnemyCount">Number of enemies to auto-place at random passable cells (0 = none).</param>
/// <param name="HealthCount">Number of health pickups to auto-place at random passable cells (0 = none).</param>
/// <param name="TreasureCount">Number of treasure cells to auto-place (0 = none).</param>
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
    uint SpareKeys,
    uint EnemyCount,
    uint HealthCount,
    uint TreasureCount);

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
    /// <param name="enemyCountText">Enemies entry text (number of enemies to auto-place; 0 = none).</param>
    /// <param name="healthCountText">Health entry text (number of health pickups to auto-place; 0 = none).</param>
    /// <param name="treasureCountText">Treasure entry text (number of treasure cells to auto-place; 0 = none).</param>
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
        string? enemyCountText,
        string? healthCountText,
        string? treasureCountText,
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
        if (!TryParseFeatureField(doorCountText, "Doors", ApiMaze.MaxDoorCount, out uint doorCount, out error)) return false;
        if (!TryParseFeatureField(spareDoorsText, "Spare Doors", ApiMaze.MaxDoorCount, out uint spareDoors, out error)) return false;
        if (!TryParseFeatureField(spareKeysText, "Spare Keys", ApiMaze.MaxDoorCount, out uint spareKeys, out error)) return false;

        // Enemies / Health default to 0 like the key/door fields and are bounded by their own caps.
        // They don't participate in the combined key+door feature cap — they map to empty passages
        // for the solver, so they don't affect the key-aware solve's feature budget.
        if (!TryParseFeatureField(enemyCountText, "Enemies", ApiMaze.MaxEnemyCount, out uint enemyCount, out error)) return false;
        if (!TryParseFeatureField(healthCountText, "Health", ApiMaze.MaxHealthCount, out uint healthCount, out error)) return false;
        // Treasure auto-places like enemies/health (its own cap, no key+door budget impact).
        if (!TryParseFeatureField(treasureCountText, "Treasure", ApiMaze.MaxTreasureCount, out uint treasureCount, out error)) return false;

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
            SpareKeys: spareKeys,
            EnemyCount: enemyCount,
            HealthCount: healthCount,
            TreasureCount: treasureCount);
        return true;
    }

    /// <summary>
    /// Parses one of the generator count fields against its per-field
    /// <paramref name="maxValue"/> bound. A null/empty entry is treated as 0
    /// (the React modal seeds these fields to "0" by default, but the user
    /// can also clear them).
    /// </summary>
    private static bool TryParseFeatureField(string? text, string fieldName, uint maxValue, out uint value, out string error)
    {
        string trimmed = text?.Trim() ?? string.Empty;
        if (trimmed.Length == 0)
        {
            value = 0;
            error = string.Empty;
            return true;
        }
        if (!uint.TryParse(trimmed, out value) || value > maxValue)
        {
            error = $"{fieldName} must be a whole number between 0 and {maxValue}.";
            return false;
        }
        error = string.Empty;
        return true;
    }
}
