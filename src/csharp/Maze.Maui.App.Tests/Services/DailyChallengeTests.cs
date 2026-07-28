using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Xunit;

namespace Maze.Maui.App.Tests.Services
{
    /// <summary>
    /// Tests for the pure daily-challenge resolver in <see cref="DailyChallenge"/> —
    /// finding the curated "Daily Challenges" collection and picking its daily member.
    /// </summary>
    public class DailyChallengeTests
    {
        private static FeaturedGameItem Collection(string id, string name) =>
            new() { Kind = "collection", Collection = new GameCollection { Id = id, Name = name } };

        private static FeaturedGameItem Definition(string id) =>
            new() { Kind = "definition", Definition = new GameDefinition { Id = id } };

        private static GameDefinition Def(string id, string rotation = "static") =>
            new() { Id = id, Rotation = rotation };

        [Fact]
        public void FindCollection_ReturnsTheDailyChallengesCollection()
        {
            var items = new List<FeaturedGameItem>
            {
                Definition("d1"),
                Collection("c-other", "Something Else"),
                Collection("c-daily", "Daily Challenges"),
            };

            Assert.Equal("c-daily", DailyChallenge.FindCollection(items)?.Id);
        }

        [Fact]
        public void FindCollection_NoneMatching_ReturnsNull()
        {
            var items = new List<FeaturedGameItem> { Definition("d1"), Collection("c-other", "Something Else") };

            Assert.Null(DailyChallenge.FindCollection(items));
        }

        [Fact]
        public void PickDaily_PrefersTheDailyRotationMember()
        {
            var members = new List<GameDefinition> { Def("g-static"), Def("g-daily", "daily") };

            Assert.Equal("g-daily", DailyChallenge.PickDaily(members)?.Id);
        }

        [Fact]
        public void PickDaily_NoDailyMember_FallsBackToFirst()
        {
            var members = new List<GameDefinition> { Def("g-a"), Def("g-b") };

            Assert.Equal("g-a", DailyChallenge.PickDaily(members)?.Id);
        }

        [Fact]
        public void PickDaily_Empty_ReturnsNull()
        {
            Assert.Null(DailyChallenge.PickDaily(new List<GameDefinition>()));
        }
    }
}
