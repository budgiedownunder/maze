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
        private const string ValidDoorCount = "0";
        private const string ValidSpareDoors = "0";
        private const string ValidSpareKeys = "0";
        private const string ValidEnemyCount = "0";
        private const string ValidHealthCount = "0";
        private const string ValidTreasureCount = "0";

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
            string? minSolutionLength = ValidMinSolutionLength,
            string? doorCount = ValidDoorCount,
            string? spareDoors = ValidSpareDoors,
            string? spareKeys = ValidSpareKeys,
            string? enemyCount = ValidEnemyCount,
            string? healthCount = ValidHealthCount,
            string? treasureCount = ValidTreasureCount)
            => GenerateMazeOptionsParser.TryParse(
                rows, cols, startRow, startCol, finishRow, finishCol,
                minSolutionLength, doorCount, spareDoors, spareKeys,
                enemyCount, healthCount, treasureCount,
                cap, out parsed, out error);

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
            Assert.Equal(0u, parsed.DoorCount);
            Assert.Equal(0u, parsed.SpareDoors);
            Assert.Equal(0u, parsed.SpareKeys);
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

        // ── Doors / Spare Doors / Spare Keys per-field bounds ──────────

        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("  ")]
        public void TryParse_treats_empty_door_count_as_zero(string? text)
        {
            bool ok = TryParseBaseline(cap: null, out var parsed, out _, doorCount: text);
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(0u, parsed!.DoorCount);
        }

        [Theory]
        [InlineData("0")]
        [InlineData("8")]
        public void TryParse_accepts_door_count_in_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out var parsed, out _, doorCount: text);
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(uint.Parse(text), parsed!.DoorCount);
        }

        [Theory]
        [InlineData("9")]
        [InlineData("100")]
        [InlineData("abc")]
        [InlineData("-1")]
        public void TryParse_rejects_door_count_out_of_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, doorCount: text);
            Assert.False(ok);
            Assert.Equal("Doors must be a whole number between 0 and 8.", error);
        }

        [Theory]
        [InlineData("9")]
        [InlineData("100")]
        [InlineData("abc")]
        public void TryParse_rejects_spare_doors_out_of_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, spareDoors: text);
            Assert.False(ok);
            Assert.Equal("Spare Doors must be a whole number between 0 and 8.", error);
        }

        [Theory]
        [InlineData("9")]
        [InlineData("100")]
        [InlineData("abc")]
        public void TryParse_rejects_spare_keys_out_of_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, spareKeys: text);
            Assert.False(ok);
            Assert.Equal("Spare Keys must be a whole number between 0 and 8.", error);
        }

        // ── Cross-field K + D cap (2*Doors + SpareDoors + SpareKeys <= 16) ──

        [Fact]
        public void TryParse_accepts_combination_at_the_K_plus_D_cap()
        {
            // 2*8 + 0 + 0 = 16 — exactly at cap.
            bool ok = TryParseBaseline(
                cap: null, out var parsed, out _,
                doorCount: "8", spareDoors: "0", spareKeys: "0");
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(8u, parsed!.DoorCount);
        }

        [Fact]
        public void TryParse_accepts_split_combination_at_the_K_plus_D_cap()
        {
            // 2*4 + 4 + 4 = 16 — exactly at cap, split.
            bool ok = TryParseBaseline(
                cap: null, out var parsed, out _,
                doorCount: "4", spareDoors: "4", spareKeys: "4");
            Assert.True(ok);
            Assert.NotNull(parsed);
        }

        [Fact]
        public void TryParse_rejects_combination_just_over_the_K_plus_D_cap()
        {
            // 2*7 + 1 + 2 = 17 — just over.
            bool ok = TryParseBaseline(
                cap: null, out _, out var error,
                doorCount: "7", spareDoors: "1", spareKeys: "2");
            Assert.False(ok);
            Assert.Equal(
                "Total keys + doors (17) exceeds the limit of 16. " +
                "Each door brings a key, so the count is 2·Doors + Spare Doors + Spare Keys.",
                error);
        }

        // ── Enemies / Health per-field bounds ──────────────────────────

        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("  ")]
        public void TryParse_treats_empty_enemy_and_health_as_zero(string? text)
        {
            bool ok = TryParseBaseline(cap: null, out var parsed, out _, enemyCount: text, healthCount: text);
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(0u, parsed!.EnemyCount);
            Assert.Equal(0u, parsed.HealthCount);
        }

        [Theory]
        [InlineData("0")]
        [InlineData("8")]
        public void TryParse_accepts_enemy_count_in_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out var parsed, out _, enemyCount: text);
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(uint.Parse(text), parsed!.EnemyCount);
        }

        [Theory]
        [InlineData("0")]
        [InlineData("8")]
        public void TryParse_accepts_health_count_in_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out var parsed, out _, healthCount: text);
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(uint.Parse(text), parsed!.HealthCount);
        }

        [Theory]
        [InlineData("9")]
        [InlineData("100")]
        [InlineData("abc")]
        [InlineData("-1")]
        public void TryParse_rejects_enemy_count_out_of_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, enemyCount: text);
            Assert.False(ok);
            Assert.Equal("Enemies must be a whole number between 0 and 8.", error);
        }

        [Theory]
        [InlineData("9")]
        [InlineData("100")]
        [InlineData("abc")]
        public void TryParse_rejects_health_count_out_of_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, healthCount: text);
            Assert.False(ok);
            Assert.Equal("Health must be a whole number between 0 and 8.", error);
        }

        [Fact]
        public void TryParse_enemy_and_health_do_not_count_against_the_K_plus_D_cap()
        {
            // Doors at the K+D cap (2*8 = 16) plus the maximum enemies + health
            // still parses — enemies / health are solver-empty and carry no
            // feature budget.
            bool ok = TryParseBaseline(
                cap: null, out var parsed, out _,
                doorCount: "8", spareDoors: "0", spareKeys: "0",
                enemyCount: "8", healthCount: "8");
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(8u, parsed!.EnemyCount);
            Assert.Equal(8u, parsed.HealthCount);
        }

        // ── Treasure per-field bounds (cap 12) ─────────────────────────

        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("  ")]
        public void TryParse_treats_empty_treasure_as_zero(string? text)
        {
            bool ok = TryParseBaseline(cap: null, out var parsed, out _, treasureCount: text);
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(0u, parsed!.TreasureCount);
        }

        [Theory]
        [InlineData("0")]
        [InlineData("12")]
        public void TryParse_accepts_treasure_count_in_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out var parsed, out _, treasureCount: text);
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(uint.Parse(text), parsed!.TreasureCount);
        }

        [Theory]
        [InlineData("13")]
        [InlineData("100")]
        [InlineData("abc")]
        [InlineData("-1")]
        public void TryParse_rejects_treasure_count_out_of_range(string text)
        {
            bool ok = TryParseBaseline(cap: null, out _, out var error, treasureCount: text);
            Assert.False(ok);
            Assert.Equal("Treasure must be a whole number between 0 and 12.", error);
        }

        [Fact]
        public void TryParse_treasure_does_not_count_against_the_K_plus_D_cap()
        {
            // Doors at the K+D cap (2*8 = 16) plus the maximum treasure still
            // parses — treasure is solver-empty and carries no feature budget.
            bool ok = TryParseBaseline(
                cap: null, out var parsed, out _,
                doorCount: "8", spareDoors: "0", spareKeys: "0",
                treasureCount: "12");
            Assert.True(ok);
            Assert.NotNull(parsed);
            Assert.Equal(12u, parsed!.TreasureCount);
        }
    }
}
