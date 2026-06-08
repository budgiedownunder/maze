using System.Linq;
using Maze.Api;
using Maze.Maui.App;
using Xunit;

namespace Maze.Maui.App.Tests.Controls
{
    /// <summary>
    /// Tests for <see cref="CellOverrides"/>, the editor's per-cell override store —
    /// in particular that structural edits keep override keys aligned with the grid
    /// (mirrors the web editor's remap rules).
    /// </summary>
    public class CellOverridesTests
    {
        // An enemy override carrying `damage` as a unique marker, so a test can track
        // which override lands where after rows/columns shift.
        private static EnemyCellEntity Marked(uint damage) => new() { Damage = damage };

        private static uint? MarkerAt(CellOverrides overrides, int row, int col) =>
            (overrides.Get(row, col) as EnemyCellEntity)?.Damage;

        [Fact]
        public void Get_Set_Has_Remove_round_trip()
        {
            CellOverrides overrides = new();
            Assert.Null(overrides.Get(1, 2));
            Assert.False(overrides.Has(1, 2));

            overrides.Set(1, 2, Marked(5));
            Assert.True(overrides.Has(1, 2));
            Assert.Equal(5u, MarkerAt(overrides, 1, 2));
            Assert.Equal(1, overrides.Count);

            overrides.Remove(1, 2);
            Assert.False(overrides.Has(1, 2));
            Assert.Equal(0, overrides.Count);
        }

        [Fact]
        public void InsertRows_shifts_overrides_at_or_below_the_point()
        {
            CellOverrides overrides = new();
            overrides.Set(0, 0, Marked(1)); // above the insert point
            overrides.Set(2, 1, Marked(2)); // at/below it
            overrides.InsertRows(1, 2);
            Assert.Equal(1u, MarkerAt(overrides, 0, 0)); // above stays put
            Assert.Equal(2u, MarkerAt(overrides, 4, 1)); // shifts down by count
            Assert.Equal(2, overrides.Count);
        }

        [Fact]
        public void DeleteRows_drops_the_band_and_shifts_the_rest_up()
        {
            CellOverrides overrides = new();
            overrides.Set(0, 0, Marked(1)); // above the band
            overrides.Set(1, 0, Marked(2)); // in deleted band [1, 3)
            overrides.Set(2, 1, Marked(3)); // in deleted band
            overrides.Set(4, 1, Marked(4)); // below the band
            overrides.DeleteRows(1, 2);
            Assert.Equal(1u, MarkerAt(overrides, 0, 0));
            Assert.Equal(4u, MarkerAt(overrides, 2, 1)); // below the band shifts up by count
            Assert.Equal(2, overrides.Count); // the two in the band are dropped
        }

        [Fact]
        public void InsertCols_shifts_overrides_at_or_right_of_the_point()
        {
            CellOverrides overrides = new();
            overrides.Set(0, 0, Marked(1)); // left of the insert point
            overrides.Set(1, 2, Marked(2)); // at/right of it
            overrides.InsertCols(1, 2);
            Assert.Equal(1u, MarkerAt(overrides, 0, 0));
            Assert.Equal(2u, MarkerAt(overrides, 1, 4)); // shifts right by count
            Assert.Equal(2, overrides.Count);
        }

        [Fact]
        public void DeleteCols_drops_the_band_and_shifts_the_rest_left()
        {
            CellOverrides overrides = new();
            overrides.Set(0, 0, Marked(1)); // left of the band
            overrides.Set(0, 1, Marked(2)); // in deleted band [1, 3)
            overrides.Set(1, 2, Marked(3)); // in deleted band
            overrides.Set(1, 4, Marked(4)); // right of the band
            overrides.DeleteCols(1, 2);
            Assert.Equal(1u, MarkerAt(overrides, 0, 0));
            Assert.Equal(4u, MarkerAt(overrides, 1, 2)); // right of the band shifts left by count
            Assert.Equal(2, overrides.Count);
        }

        [Fact]
        public void Entries_yields_every_override()
        {
            CellOverrides overrides = new();
            overrides.Set(0, 0, Marked(1));
            overrides.Set(3, 2, Marked(2));
            List<(int, int)> cells = overrides.Entries
                .Select(entry => (entry.Key.Row, entry.Key.Col))
                .OrderBy(cell => cell)
                .ToList();
            Assert.Equal(new[] { (0, 0), (3, 2) }, cells);
        }

        [Fact]
        public void Remap_is_a_no_op_with_no_overrides()
        {
            CellOverrides overrides = new();
            overrides.InsertRows(0, 3);
            overrides.DeleteCols(1, 2);
            Assert.Equal(0, overrides.Count);
        }
    }
}
