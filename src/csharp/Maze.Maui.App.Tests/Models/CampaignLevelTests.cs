using Maze.Maui.App.Models;
using Xunit;

namespace Maze.Maui.App.Tests.Models
{
    /// <summary>
    /// Tests for the pure campaign progression logic in <see cref="CampaignLevel"/>:
    /// completion is global (any score = complete, always shown, even out of order),
    /// the current level is the first with no score, and everything unscored after it
    /// is locked.
    /// </summary>
    public class CampaignLevelTests
    {
        private static GameDefinition Def(string id) => new() { Id = id, Name = id };

        [Fact]
        public void ChallengeKey_IsDefPrefixed()
        {
            Assert.Equal("def:g1", CampaignLevel.ChallengeKey(Def("g1")));
        }

        [Fact]
        public void Build_NoneCompleted_FirstIsCurrentRestLocked()
        {
            IReadOnlyList<CampaignLevel> levels = CampaignLevel.Build([Def("g1"), Def("g2"), Def("g3")], []);

            Assert.Equal(3, levels.Count);
            Assert.Equal(1, levels[0].Number);
            Assert.Equal(3, levels[2].Number);
            Assert.Equal(CampaignLevelState.Current, levels[0].State);
            Assert.Equal(CampaignLevelState.Locked, levels[1].State);
            Assert.Equal(CampaignLevelState.Locked, levels[2].State);
        }

        [Fact]
        public void Build_GlobalCompletionOutOfOrder_KeepsLaterCompletedButLocksUnscoredAfterCurrent()
        {
            // g1 + g3 scored (globally); g2 is the first unscored → current; g3 stays
            // Completed even though it sits after the current level; g4 is locked.
            IReadOnlyList<CampaignLevel> levels = CampaignLevel.Build(
                [Def("g1"), Def("g2"), Def("g3"), Def("g4")],
                ["def:g1", "def:g3"]);

            Assert.Equal(CampaignLevelState.Completed, levels[0].State);
            Assert.Equal(CampaignLevelState.Current, levels[1].State);
            Assert.Equal(CampaignLevelState.Completed, levels[2].State);
            Assert.Equal(CampaignLevelState.Locked, levels[3].State);
        }

        [Fact]
        public void Build_AllCompleted_NoCurrentOrLocked()
        {
            IReadOnlyList<CampaignLevel> levels = CampaignLevel.Build(
                [Def("g1"), Def("g2")], ["def:g1", "def:g2"]);

            Assert.All(levels, l => Assert.Equal(CampaignLevelState.Completed, l.State));
        }

        [Theory]
        [InlineData(CampaignLevelState.Completed, "✓ Completed")]
        [InlineData(CampaignLevelState.Current, "Play")]
        [InlineData(CampaignLevelState.Locked, "Locked")]
        public void StatusText_MatchesState(CampaignLevelState state, string expected)
        {
            var level = new CampaignLevel { Number = 1, Definition = Def("g1"), State = state };
            Assert.Equal(expected, level.StatusText);
        }
    }
}
