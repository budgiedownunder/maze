using Maze.Maui.App.Utils;
using Xunit;

namespace Maze.Maui.App.Tests.Utils
{
    /// <summary>
    /// Tests for <see cref="GenerateMazeOptionsParser.TryParse"/>, the helper
    /// that <see cref="Maze.Maui.App.Views.GenerateMazePopup"/> delegates to
    /// for parsing and validating the form entries. Each test feeds in a
    /// "valid baseline" and tweaks one field to exercise a single branch of
    /// the validation chain.
    /// </summary>
    public class GenerateMazeOptionsParserTests
    {
        // Baseline: valid inputs that produce a successful parse. Individual
        // tests override one field to trigger the failure they care about.
        private const string ValidRows = "10";
        private const string ValidCols = "10";
        private const string ValidStartRow = "1";
        private const string ValidStartCol = "1";
        private const string ValidFinishRow = "10";
        private const string ValidFinishCol = "10";
        private const string ValidMinSolutionLength = "5";

        private static bool TryParseBaseline(
            int? cap,
            out ParsedGenerateOptions? parsed,
            out string error,
            string? rows = ValidRows,
            string? cols = ValidCols,
            string? startRow = ValidStartRow,
            string? startCol = ValidStartCol,
            string? finishRow = ValidFinishRow,
            string? finishCol = ValidFinishCol,
            string? minSolutionLength = ValidMinSolutionLength)
            => GenerateMazeOptionsParser.TryParse(
                rows, cols, startRow, startCol, finishRow, finishCol,
                minSolutionLength, cap, out parsed, out error);

        // ── Happy path ─────────────────────────────────────────────────

        [Fact]
        public void TryParse_returns_true_for_valid_input_with_null_cap()
        {
            bool ok = TryParseBaseline(cap: null, out var parsed, out var error);
            Assert.True(ok);
            Assert.Equal(string.Empty, error);
            Assert.NotNull(parsed);
            Assert.Equal(10u, parsed!.Rows);
            Assert.Equal(10u, parsed.Cols);
            // 1-based input "1"/"1" → 0-based output 0/0
            Assert.Equal(0u, parsed.StartRow);
            Assert.Equal(0u, parsed.StartCol);
            // 1-based input "10"/"10" → 0-based output 9/9
            Assert.Equal(9u, parsed.FinishRow);
            Assert.Equal(9u, parsed.FinishCol);
            Assert.Equal(5u, parsed.MinSolutionLength);
        }

        [Fact]
        public void TryParse_returns_true_when_rows_x_cols_at_cap()
        {
            // 60 × 60 = 3,600 = cap
            bool ok = TryParseBaseline(
                cap: 3_600, out var parsed, out var error,
                rows: "60", cols: "60", finishRow: "60", finishCol: "60");
            Assert.True(ok);
            Assert.Equal(string.Empty, error);
            Assert.NotNull(parsed);
        }

        // ── Rows / Cols ────────────────────────────────────────────────

        [Theory]
        [InlineData("")]
        [InlineData("0")]
        [InlineData("2")]
        [InlineData("abc")]
        [InlineData(null)]
        public void TryParse_rejects_invalid_rows(string? rowsText)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, rows: rowsText);
            Assert.False(ok);
            Assert.Equal("Rows must be a whole number of 3 or more.", error);
        }

        [Theory]
        [InlineData("")]
        [InlineData("0")]
        [InlineData("2")]
        [InlineData("abc")]
        [InlineData(null)]
        public void TryParse_rejects_invalid_cols(string? colsText)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, cols: colsText);
            Assert.False(ok);
            Assert.Equal("Columns must be a whole number of 3 or more.", error);
        }

        // ── Cap ────────────────────────────────────────────────────────

        [Fact]
        public void TryParse_rejects_over_cap_with_message_naming_the_cap()
        {
            // 61 × 60 = 3,660 > 3,600
            bool ok = TryParseBaseline(
                cap: 3_600, out _, out var error,
                rows: "61", cols: "60", finishRow: "61", finishCol: "60");
            Assert.False(ok);
            Assert.Equal("Total cells (rows × columns) cannot exceed 3600.", error);
        }

        [Fact]
        public void TryParse_does_not_enforce_a_cap_when_max_is_null()
        {
            // 200 × 200 = 40,000 — would trip any practical cap; with cap=null
            // the parser must accept it.
            bool ok = TryParseBaseline(
                cap: null, out var parsed, out _,
                rows: "200", cols: "200", finishRow: "200", finishCol: "200");
            Assert.True(ok);
            Assert.NotNull(parsed);
        }

        // ── Start / Finish range ───────────────────────────────────────

        [Theory]
        [InlineData("0", "Start Row must be between 1 and 10.")]
        [InlineData("11", "Start Row must be between 1 and 10.")]
        [InlineData("", "Start Row must be between 1 and 10.")]
        [InlineData("abc", "Start Row must be between 1 and 10.")]
        public void TryParse_rejects_invalid_start_row(string text, string expected)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, startRow: text);
            Assert.False(ok);
            Assert.Equal(expected, error);
        }

        [Theory]
        [InlineData("0", "Start Column must be between 1 and 10.")]
        [InlineData("11", "Start Column must be between 1 and 10.")]
        public void TryParse_rejects_invalid_start_col(string text, string expected)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, startCol: text);
            Assert.False(ok);
            Assert.Equal(expected, error);
        }

        [Theory]
        [InlineData("0", "Finish Row must be between 1 and 10.")]
        [InlineData("11", "Finish Row must be between 1 and 10.")]
        public void TryParse_rejects_invalid_finish_row(string text, string expected)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, finishRow: text);
            Assert.False(ok);
            Assert.Equal(expected, error);
        }

        [Theory]
        [InlineData("0", "Finish Column must be between 1 and 10.")]
        [InlineData("11", "Finish Column must be between 1 and 10.")]
        public void TryParse_rejects_invalid_finish_col(string text, string expected)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, finishCol: text);
            Assert.False(ok);
            Assert.Equal(expected, error);
        }

        [Fact]
        public void TryParse_rejects_start_equal_to_finish()
        {
            bool ok = TryParseBaseline(
                cap: null, out _, out var error,
                startRow: "3", startCol: "3", finishRow: "3", finishCol: "3");
            Assert.False(ok);
            Assert.Equal("Start and Finish cells must be different.", error);
        }

        // ── Min solution length ────────────────────────────────────────

        [Theory]
        [InlineData("0")]
        [InlineData("")]
        [InlineData("abc")]
        [InlineData(null)]
        public void TryParse_rejects_invalid_min_solution_length(string? text)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, minSolutionLength: text);
            Assert.False(ok);
            Assert.Equal("Min Solution Length must be a whole number of 1 or more.", error);
        }

        // ── Trimming ───────────────────────────────────────────────────

        [Fact]
        public void TryParse_trims_whitespace_around_inputs()
        {
            bool ok = TryParseBaseline(
                cap: null, out var parsed, out _,
                rows: "  10  ", cols: " 10 ", minSolutionLength: "\t5\t");
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(10u, parsed!.Rows);
            Assert.Equal(10u, parsed.Cols);
            Assert.Equal(5u, parsed.MinSolutionLength);
        }
    }
}
