using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the Leaderboards page logic: subject discovery (paging, dedup,
    /// page cap), default selection, maze-name resolution (exact + basename),
    /// the Game Type → Game cascade, curated-seed resolution + caching, board
    /// paging/append, the metric toggle, the Play-3D-only Player/highlight
    /// gating, the include-usernames flag, and the empty state.
    /// </summary>
    public class LeaderboardsViewModelTests
    {
        private static (LeaderboardsViewModel vm, Mock<IScoresService> scores, Mock<IGameConfigService> config, Mock<IMazeService> mazes, Mock<IAuthService> auth)
            BuildVm()
        {
            var scores = new Mock<IScoresService>();
            var config = new Mock<IGameConfigService>();
            var mazes = new Mock<IMazeService>();
            var auth = new Mock<IAuthService>();

            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(new UserProfile { Id = "me", Username = "Me" });
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem>());
            scores.Setup(s => s.GetScoreHistoryAsync(It.IsAny<int?>(), It.IsAny<int?>())).ReturnsAsync(EmptyBoard());
            config.Setup(c => c.GetPlay3dConfigAsync(It.IsAny<Difficulty>()))
                  .ReturnsAsync(new Play3dConfig { Difficulty = "easy", Seed = 42 });
            scores.Setup(s => s.GetLeaderboardAsync(
                It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(),
                It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>())).ReturnsAsync(EmptyBoard());

            var vm = new LeaderboardsViewModel(scores.Object, config.Object, mazes.Object, auth.Object);
            return (vm, scores, config, mazes, auth);
        }

        private static ScoreboardResponse EmptyBoard() =>
            new() { Scores = new List<ScoreEntry>(), Limit = 20, Offset = 0, HasMore = false };

        private static ScoreboardResponse Board(IEnumerable<ScoreEntry> scores, bool hasMore) =>
            new() { Scores = scores.ToList(), Limit = 20, Offset = 0, HasMore = hasMore };

        private static ScoreEntry MazeRow(string mazeId, string userId = "me", string? username = null, ulong score = 1) =>
            new() { Id = Guid.NewGuid().ToString(), UserId = userId, MazeId = mazeId, Challenge = null, Score = score, ElapsedMs = 1000, RecordedAt = DateTimeOffset.UnixEpoch, Username = username };

        private static ScoreEntry ChallengeRow(string challenge, string userId = "me", string? username = null, ulong score = 1) =>
            new() { Id = Guid.NewGuid().ToString(), UserId = userId, MazeId = null, Challenge = challenge, Score = score, ElapsedMs = 1000, RecordedAt = DateTimeOffset.UnixEpoch, Username = username };

        private static MazeItem MazeItem(string id, string name) => new() { ID = id, Name = name };

        // ---- discovery + default selection ----------------------------------

        [Fact]
        public async Task Initialize_DefaultsToMostRecentMaze_MyMazesNoUsernames()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(100, 0))
                  .ReturnsAsync(Board(new[] { MazeRow("m1"), MazeRow("m2") }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem>
            {
                MazeItem("m1", "Beta"),
                MazeItem("m2", "Alpha"),
            });
            scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { MazeRow("m1") }, false));

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Equal(LeaderboardGameType.MyMazes, vm.SelectedGameType!.Kind);
            Assert.Equal("m1", vm.SelectedGame!.MazeId);
            Assert.False(vm.ShowPlayerColumn);
            string[] expectedGames = { "Alpha", "Beta" };
            Assert.Equal(expectedGames, vm.Games.Select(g => g.Label).ToArray());
            Assert.Single(vm.Rows);
            scores.Verify(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), 0, false), Times.Once);
        }

        [Fact]
        public async Task Initialize_DefaultsToPlay3d_WhenMostRecentIsChallenge()
        {
            var (vm, scores, config, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(100, 0))
                  .ReturnsAsync(Board(new[] { ChallengeRow("tricky:99") }, false));

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Equal(LeaderboardGameType.Play3d, vm.SelectedGameType!.Kind);
            Assert.Equal(Difficulty.Tricky, vm.SelectedGame!.Difficulty);
            Assert.True(vm.ShowPlayerColumn);
            config.Verify(c => c.GetPlay3dConfigAsync(Difficulty.Tricky), Times.Once);
            scores.Verify(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge == "tricky:42"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), true), Times.Once);
        }

        [Fact]
        public async Task Initialize_DefaultsToPlay3dEasy_WhenNoHistory()
        {
            var (vm, _, _, _, _) = BuildVm();

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Equal(LeaderboardGameType.Play3d, vm.SelectedGameType!.Kind);
            Assert.Equal(Difficulty.Easy, vm.SelectedGame!.Difficulty);
            string[] expectedGames = { "Easy", "Tricky", "Hard" };
            Assert.Equal(expectedGames, vm.Games.Select(g => g.Label).ToArray());
        }

        [Fact]
        public async Task Discovery_PagesUntilHasMoreFalse_AndDedups()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(100, 0))
                  .ReturnsAsync(Board(new[] { MazeRow("m1"), MazeRow("m1"), MazeRow("m2") }, true));
            scores.Setup(s => s.GetScoreHistoryAsync(100, 100))
                  .ReturnsAsync(Board(new[] { MazeRow("m2"), MazeRow("m3") }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem>
            {
                MazeItem("m1", "One"), MazeItem("m2", "Two"), MazeItem("m3", "Three"),
            });

            await vm.InitializeCommand.ExecuteAsync(null);

            string[] expectedGames = { "One", "Three", "Two" };
            Assert.Equal(expectedGames, vm.Games.Select(g => g.Label).ToArray());
            scores.Verify(s => s.GetScoreHistoryAsync(100, 0), Times.Once);
            scores.Verify(s => s.GetScoreHistoryAsync(100, 100), Times.Once);
        }

        [Fact]
        public async Task Discovery_StopsAtPageCap()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            // Always-more history would loop forever without the 25-page cap.
            scores.Setup(s => s.GetScoreHistoryAsync(It.IsAny<int?>(), It.IsAny<int?>()))
                  .ReturnsAsync(Board(new[] { MazeRow("m1") }, true));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });

            await vm.InitializeCommand.ExecuteAsync(null);

            scores.Verify(s => s.GetScoreHistoryAsync(It.IsAny<int?>(), It.IsAny<int?>()), Times.Exactly(25));
        }

        [Fact]
        public async Task MazeNames_ResolveByExactId_BasenameFallback_AndJsonStrip()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(100, 0)).ReturnsAsync(Board(new[]
            {
                MazeRow("id-exact"),
                MazeRow(@"C:\other\Shared.json"),   // basename matches a different stored path
                MazeRow(@"C:\x\Lonely.json"),       // no match → basename, .json stripped
            }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem>
            {
                MazeItem("id-exact", "Exact"),
                MazeItem(@"C:\data\Shared.json", "SharedName"),
            });

            await vm.InitializeCommand.ExecuteAsync(null);

            string[] expectedGames = { "Exact", "Lonely", "SharedName" };
            Assert.Equal(expectedGames, vm.Games.Select(g => g.Label).ToArray());
        }

        // ---- cascade + seed resolution --------------------------------------

        [Fact]
        public async Task Cascade_ChangingGameType_RepopulatesGames_AndResetsSelection()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(100, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            await vm.InitializeCommand.ExecuteAsync(null);

            vm.SelectedGameType = vm.GameTypes.First(t => t.Kind == LeaderboardGameType.Play3d);
            await vm.ReloadBoardCommand.ExecuteAsync(null);

            string[] expectedGames = { "Easy", "Tricky", "Hard" };
            Assert.Equal(expectedGames, vm.Games.Select(g => g.Label).ToArray());
            Assert.Equal(Difficulty.Easy, vm.SelectedGame!.Difficulty);
            Assert.True(vm.ShowPlayerColumn);
        }

        [Fact]
        public async Task SeedResolution_IsCachedPerDifficulty()
        {
            var (vm, _, config, _, _) = BuildVm();
            await vm.InitializeCommand.ExecuteAsync(null);   // play3d easy → one config call

            await vm.SelectScoreMetricCommand.ExecuteAsync(null);   // reload (same subject)
            await vm.SelectTimeMetricCommand.ExecuteAsync(null);    // reload (same subject)

            config.Verify(c => c.GetPlay3dConfigAsync(Difficulty.Easy), Times.Once);
        }

        // ---- board paging + metric ------------------------------------------

        [Fact]
        public async Task LoadMore_AppendsRows_AndAdvancesOffset()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(100, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            var firstPage = Enumerable.Range(0, 20).Select(_ => MazeRow("m1"));
            var secondPage = Enumerable.Range(0, 5).Select(_ => MazeRow("m1"));
            scores.Setup(s => s.GetLeaderboardAsync(It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(),
                It.IsAny<SortDirection?>(), It.IsAny<int?>(), 0, It.IsAny<bool?>())).ReturnsAsync(Board(firstPage, true));
            scores.Setup(s => s.GetLeaderboardAsync(It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(),
                It.IsAny<SortDirection?>(), It.IsAny<int?>(), 20, It.IsAny<bool?>())).ReturnsAsync(Board(secondPage, false));

            await vm.InitializeCommand.ExecuteAsync(null);
            Assert.True(vm.HasMore);
            Assert.Equal(20, vm.Rows.Count);

            await vm.LoadMoreCommand.ExecuteAsync(null);

            Assert.Equal(25, vm.Rows.Count);
            Assert.False(vm.HasMore);
            Assert.Equal(25, vm.Rows[24].Rank);   // ranks continue across pages
            scores.Verify(s => s.GetLeaderboardAsync(It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(),
                It.IsAny<SortDirection?>(), It.IsAny<int?>(), 20, It.IsAny<bool?>()), Times.Once);
        }

        [Fact]
        public async Task MetricSwitch_ReloadsWithNewMetric()
        {
            var (vm, scores, _, _, _) = BuildVm();
            await vm.InitializeCommand.ExecuteAsync(null);   // play3d easy, time metric

            await vm.SelectScoreMetricCommand.ExecuteAsync(null);

            Assert.True(vm.IsScoreMetricSelected);
            Assert.False(vm.IsTimeMetricSelected);
            scores.Verify(s => s.GetLeaderboardAsync(It.IsAny<ScoreSubject>(), ScoreMetric.Score,
                It.IsAny<SortDirection?>(), It.IsAny<int?>(), 0, It.IsAny<bool?>()), Times.Once);
        }

        // ---- highlight / player gating + empty state ------------------------

        [Fact]
        public async Task Highlight_OnPlay3dBoard_OnlyForCallerRows()
        {
            var (vm, scores, _, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(100, 0)).ReturnsAsync(Board(new[] { ChallengeRow("easy:42") }, false));
            scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge != null),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[]
                  {
                      ChallengeRow("easy:42", userId: "other", username: "Rival"),
                      ChallengeRow("easy:42", userId: "me", username: "Me"),
                  }, false));

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.ShowPlayerColumn);
            Assert.False(vm.Rows[0].IsHighlighted);
            Assert.Equal("Rival", vm.Rows[0].Player);
            Assert.True(vm.Rows[1].IsHighlighted);
        }

        [Fact]
        public async Task Highlight_NeverOnMyMazesBoard_EvenForCaller()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(100, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { MazeRow("m1", userId: "me") }, false));

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.False(vm.ShowPlayerColumn);
            Assert.False(vm.Rows[0].IsHighlighted);
        }

        [Fact]
        public async Task EmptyBoard_ShowsNoWinningScoresMessage()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(100, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            // leaderboard returns empty (default EmptyBoard)

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Empty(vm.Rows);
            Assert.True(vm.ShowStatusMessage);
            Assert.Equal("No winning scores yet", vm.StatusMessage);
        }
    }
}
