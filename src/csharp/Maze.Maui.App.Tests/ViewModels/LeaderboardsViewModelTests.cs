using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the Leaderboards page logic: subject discovery (all mazes listed,
    /// most-recent default), default selection (incl. basename resolution), the
    /// Game Type → Game cascade, curated-seed resolution + caching, board
    /// paging/append, the metric toggle, the Play-3D-only Player/highlight gating,
    /// the empty state, and the Play button (launch, enablement, Play/Play-Again
    /// label).
    /// </summary>
    public class LeaderboardsViewModelTests
    {
        private static (LeaderboardsViewModel vm, Mock<IScoresService> scores, Mock<IGameConfigService> config, Mock<IMazeService> mazes, Mock<IAuthService> auth, Mock<INavigationService> nav)
            BuildVm()
        {
            var scores = new Mock<IScoresService>();
            var config = new Mock<IGameConfigService>();
            var mazes = new Mock<IMazeService>();
            var auth = new Mock<IAuthService>();
            var nav = new Mock<INavigationService>();
            var avatar = new Mock<IAvatarService>();

            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(new UserProfile { Id = "me", Username = "Me" });
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem>());
            scores.Setup(s => s.GetScoreHistoryAsync(It.IsAny<int?>(), It.IsAny<int?>())).ReturnsAsync(EmptyBoard());
            config.Setup(c => c.GetPlay3dConfigAsync(It.IsAny<Difficulty>()))
                  .ReturnsAsync(new Play3dConfig { Difficulty = "easy", Seed = 42 });
            scores.Setup(s => s.GetLeaderboardAsync(
                It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(),
                It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>())).ReturnsAsync(EmptyBoard());

            var vm = new LeaderboardsViewModel(scores.Object, config.Object, mazes.Object, auth.Object, nav.Object, avatar.Object);
            return (vm, scores, config, mazes, auth, nav);
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
        public void GameTypes_AreLabelledMazesAndPlay3d()
        {
            var (vm, _, _, _, _, _) = BuildVm();
            string[] expected = { "Mazes", "Play 3D" };
            Assert.Equal(expected, vm.GameTypes.Select(t => t.Label).ToArray());
        }

        [Fact]
        public async Task LoadBoard_Play3d_ResolvesRowAvatarBytesPerPlayer()
        {
            var scores = new Mock<IScoresService>();
            var config = new Mock<IGameConfigService>();
            var mazes = new Mock<IMazeService>();
            var auth = new Mock<IAuthService>();
            var nav = new Mock<INavigationService>();
            var avatar = new Mock<IAvatarService>();

            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(new UserProfile { Id = "me", Username = "Me" });
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem>());
            config.Setup(c => c.GetPlay3dConfigAsync(It.IsAny<Difficulty>()))
                  .ReturnsAsync(new Play3dConfig { Difficulty = "easy", Seed = 42 });
            // Most-recent run is a curated challenge → defaults to a Play-3D board
            // (Player column shown), so player avatars are resolved.
            scores.Setup(s => s.GetScoreHistoryAsync(It.IsAny<int?>(), It.IsAny<int?>()))
                  .ReturnsAsync(Board(new[] { ChallengeRow("easy:42") }, false));

            var withAvatar = ChallengeRow("easy:42", userId: "alice", username: "alice");
            withAvatar.AvatarUpdatedAt = "2026-06-16T12:00:00Z";
            var withoutAvatar = ChallengeRow("easy:42", userId: "bob", username: "bob");
            scores.Setup(s => s.GetLeaderboardAsync(
                It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(),
                It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { withAvatar, withoutAvatar }, false));

            byte[] aliceBytes = { 9, 8, 7 };
            avatar.Setup(s => s.TryLoadAvatarBytesAsync("alice", "2026-06-16T12:00:00Z")).ReturnsAsync(aliceBytes);

            var vm = new LeaderboardsViewModel(scores.Object, config.Object, mazes.Object, auth.Object, nav.Object, avatar.Object);
            await vm.InitializeCommand.ExecuteAsync(null);

            LeaderboardRow aliceRow = vm.Rows.First(r => r.UserId == "alice");
            LeaderboardRow bobRow = vm.Rows.First(r => r.UserId == "bob");
            Assert.Same(aliceBytes, aliceRow.AvatarBytes);
            // No marker for bob → no fetch, placeholder shown (null bytes).
            Assert.Null(bobRow.AvatarBytes);
            avatar.Verify(s => s.TryLoadAvatarBytesAsync("bob", It.IsAny<string?>()), Times.Never);
        }

        [Fact]
        public async Task Initialize_DefaultsToMostRecentMaze_MyMazesNoUsernames()
        {
            var (vm, scores, _, mazes, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0))
                  .ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
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
            var (vm, scores, config, _, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0))
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
        public async Task Initialize_DefaultsToPlay3dEasy_WhenNoMazesNoHistory()
        {
            var (vm, _, _, _, _, _) = BuildVm();

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Equal(LeaderboardGameType.Play3d, vm.SelectedGameType!.Kind);
            Assert.Equal(Difficulty.Easy, vm.SelectedGame!.Difficulty);
            string[] expectedGames = { "Easy", "Tricky", "Hard" };
            Assert.Equal(expectedGames, vm.Games.Select(g => g.Label).ToArray());
        }

        [Fact]
        public async Task Games_ListAllMazes_IncludingUnplayed()
        {
            var (vm, _, _, mazes, _, _) = BuildVm();
            // No history (no plays at all), but the player has mazes → all listed,
            // defaulting to the first maze.
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem>
            {
                MazeItem("m1", "Beta"), MazeItem("m2", "Alpha"),
            });

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Equal(LeaderboardGameType.MyMazes, vm.SelectedGameType!.Kind);
            string[] expectedGames = { "Alpha", "Beta" };
            Assert.Equal(expectedGames, vm.Games.Select(g => g.Label).ToArray());
        }

        [Fact]
        public async Task DefaultSelection_ResolvesMostRecentMazeByBasename()
        {
            var (vm, scores, _, mazes, _, _) = BuildVm();
            // The most-recent run's maze_id is a path differing from the stored id
            // (FileStore) — it should still resolve to that maze by filename.
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0))
                  .ReturnsAsync(Board(new[] { MazeRow(@"C:\other\Shared.json") }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem>
            {
                MazeItem(@"C:\data\Shared.json", "SharedName"),
            });

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Equal(LeaderboardGameType.MyMazes, vm.SelectedGameType!.Kind);
            Assert.Equal(@"C:\data\Shared.json", vm.SelectedGame!.MazeId);
        }

        // ---- cascade + seed resolution --------------------------------------

        [Fact]
        public async Task Cascade_ChangingGameType_RepopulatesGames_AndResetsSelection()
        {
            var (vm, scores, _, mazes, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
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
            var (vm, _, config, _, _, _) = BuildVm();
            await vm.InitializeCommand.ExecuteAsync(null);   // play3d easy → one config call

            await vm.SelectScoreMetricCommand.ExecuteAsync(null);   // reload (same subject)
            await vm.SelectTimeMetricCommand.ExecuteAsync(null);    // reload (same subject)

            config.Verify(c => c.GetPlay3dConfigAsync(Difficulty.Easy), Times.Once);
        }

        // ---- board paging + metric ------------------------------------------

        [Fact]
        public async Task LoadMore_AppendsRows_AndAdvancesOffset()
        {
            var (vm, scores, _, mazes, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
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
        public async Task Refresh_RefetchesCurrentBoard()
        {
            var (vm, scores, _, _, _, _) = BuildVm();
            await vm.InitializeCommand.ExecuteAsync(null);   // play3d easy → board loaded once

            await vm.RefreshCommand.ExecuteAsync(null);

            // Same subject/metric, but force-reloaded → the page-1 fetch runs again.
            scores.Verify(s => s.GetLeaderboardAsync(It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(),
                It.IsAny<SortDirection?>(), It.IsAny<int?>(), 0, It.IsAny<bool?>()), Times.Exactly(2));
        }

        [Fact]
        public async Task MetricSwitch_ReloadsWithNewMetric()
        {
            var (vm, scores, _, _, _, _) = BuildVm();
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
            var (vm, scores, _, _, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("easy:42") }, false));
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
            var (vm, scores, _, mazes, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
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
            var (vm, scores, _, mazes, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            // leaderboard returns empty (default EmptyBoard)

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Empty(vm.Rows);
            Assert.True(vm.ShowStatusMessage);
            Assert.Equal("No winning scores yet", vm.StatusMessage);
        }

        // ---- Play button -----------------------------------------------------

        [Fact]
        public async Task Play_Play3d_NavigatesWithDifficulty()
        {
            var (vm, scores, _, _, _, nav) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("easy:42") }, false));
            await vm.InitializeCommand.ExecuteAsync(null);   // play3d easy

            await vm.PlayCommand.ExecuteAsync(null);

            nav.Verify(n => n.GoToAsync("Play3dGamePage",
                It.Is<IDictionary<string, object>?>(d => d != null && (string)d["difficulty"] == "easy")), Times.Once);
        }

        [Fact]
        public async Task Play_MyMazes_LoadsMazeAndNavigatesWithSettings()
        {
            var (vm, scores, _, mazes, _, nav) = BuildVm();
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            var full = new MazeItem { ID = "m1", Name = "One", GameSettings = new MazeGameSettings() };
            mazes.Setup(m => m.GetMazeItem("m1")).ReturnsAsync(full);
            await vm.InitializeCommand.ExecuteAsync(null);   // my-mazes m1 (first maze)

            await vm.PlayCommand.ExecuteAsync(null);

            mazes.Verify(m => m.GetMazeItem("m1"), Times.Once);
            nav.Verify(n => n.GoToAsync("Play3dGamePage",
                It.Is<IDictionary<string, object>?>(d => d != null
                    && ReferenceEquals(d["MazeItem"], full) && d.ContainsKey("LaunchSettings"))), Times.Once);
        }

        [Fact]
        public async Task CanPlay_FalseWhenMazesSelectedButNone()
        {
            var (vm, _, _, _, _, _) = BuildVm();
            await vm.InitializeCommand.ExecuteAsync(null);   // play3d easy → playable
            Assert.True(vm.CanPlay);
            Assert.True(vm.PlayCommand.CanExecute(null));

            // Switch to Mazes with none → no selectable game → Play disabled.
            vm.SelectedGameType = vm.GameTypes.First(t => t.Kind == LeaderboardGameType.MyMazes);

            Assert.Null(vm.SelectedGame);
            Assert.False(vm.CanPlay);
            Assert.False(vm.PlayCommand.CanExecute(null));
        }

        [Fact]
        public async Task PlayLabel_PlayAgainWhenCallerHasRunOnBoard()
        {
            var (vm, scores, _, mazes, _, _) = BuildVm();
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { MazeRow("m1", userId: "me") }, false));

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.HasPlayed);
            Assert.Equal("↻ Play Again", vm.PlayLabel);
        }

        [Fact]
        public async Task PlayLabel_PlayWhenCallerHasNoRun()
        {
            var (vm, scores, _, mazes, _, _) = BuildVm();
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            scores.Setup(s => s.GetLeaderboardAsync(It.IsAny<ScoreSubject>(),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { MazeRow("m1", userId: "other") }, false));

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.False(vm.HasPlayed);
            Assert.Equal("▶ Play", vm.PlayLabel);
        }
    }
}
