using Maze.Maui.App.Utils;
using Xunit;

namespace Maze.Maui.App.Tests.Utils
{
    /// <summary>
    /// Tests for <see cref="MazeCellCap.Exceeds"/>, the helper that
    /// <see cref="Maze.Maui.App.Views.GenerateMazePopup"/> uses to validate
    /// rows × cols against the server-reported <c>AppFeatures.MaxMazeCells</c>.
    /// </summary>
    public class MazeCellCapTests
    {
        [Fact]
        public void Exceeds_returns_false_when_cap_is_null()
        {
            Assert.False(MazeCellCap.Exceeds(200u, 200u, null));
        }

        [Fact]
        public void Exceeds_returns_false_when_at_cap()
        {
            // 60 × 60 = 3,600 = cap
            Assert.False(MazeCellCap.Exceeds(60u, 60u, 3_600));
        }

        [Fact]
        public void Exceeds_returns_false_when_just_under_cap()
        {
            // 60 × 59 = 3,540 < 3,600
            Assert.False(MazeCellCap.Exceeds(60u, 59u, 3_600));
        }

        [Fact]
        public void Exceeds_returns_true_when_just_over_cap()
        {
            // 61 × 60 = 3,660 > 3,600
            Assert.True(MazeCellCap.Exceeds(61u, 60u, 3_600));
        }

        [Fact]
        public void Exceeds_returns_true_for_pathologically_large_inputs()
        {
            // (uint × uint) overflows the 32-bit space; the helper widens to
            // ulong so the comparison stays correct rather than wrapping.
            Assert.True(MazeCellCap.Exceeds(uint.MaxValue, 2u, 3_600));
        }
    }
}
