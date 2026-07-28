using Maze.Maui.App.Models;
using Xunit;

namespace Maze.Maui.App.Tests.Models
{
    /// <summary>
    /// Tests for the stored-game leaderboard challenge-key convention in
    /// <see cref="GameChallenge"/> (the C# mirror of the web client's
    /// <c>gameChallengeKey</c> / <c>gameIdFromChallenge</c>).
    /// </summary>
    public class GameChallengeTests
    {
        [Fact]
        public void For_StaticGame_IsBareDefKey()
        {
            Assert.Equal("def:g1", GameChallenge.For("g1", GameVocabulary.Rotation.Static));
        }

        [Fact]
        public void For_DailyGame_AppendsGivenDate()
        {
            Assert.Equal(
                "def:g1:2026-07-25",
                GameChallenge.For("g1", GameVocabulary.Rotation.Daily, "2026-07-25"));
        }

        [Fact]
        public void For_DailyGame_NoDate_UsesTodayUtc()
        {
            Assert.Equal(
                $"def:g1:{GameChallenge.TodayUtc()}",
                GameChallenge.For("g1", GameVocabulary.Rotation.Daily));
        }

        [Fact]
        public void DefinitionIdFromChallenge_StaticKey_ReturnsId()
        {
            Assert.Equal("g1", GameChallenge.DefinitionIdFromChallenge("def:g1"));
        }

        [Fact]
        public void DefinitionIdFromChallenge_DailyKey_StripsDateSuffix()
        {
            Assert.Equal("g1", GameChallenge.DefinitionIdFromChallenge("def:g1:2026-07-25"));
        }

        [Fact]
        public void DefinitionIdFromChallenge_NonDefChallenge_ReturnsNull()
        {
            Assert.Null(GameChallenge.DefinitionIdFromChallenge("easy:42"));
        }
    }
}
