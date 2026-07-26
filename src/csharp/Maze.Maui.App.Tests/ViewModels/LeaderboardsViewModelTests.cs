using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the Leaderboards page logic: subject discovery, default selection
    /// (maze basename resolution + most-recent stored-game via the play-fetch + a
    /// card preselect), the Mazes / 3D-Games selection, board paging/append, the
    /// metric toggle, the 3D-only Player/highlight gating, the empty state, the Play
    /// button, and the owner/admin-gated Reset.
    /// </summary>
    public class LeaderboardsViewModelTests
    {
        private sealed record Mocks(
            Mock<IScoresService> Scores,
            Mock<IGameLibraryService> GameLib,
            Mock<IMazeService> Mazes,
            Mock<IAuthService> Auth,
            Mock<INavigationService> Nav,
            Mock<IAvatarService> Avatar,
            Mock<IDialogService> Dialog);

        private static Mocks CreateMocks()
        {
            var scores = new Mock<IScoresService>();
            var gameLib = new Mock<IGameLibraryService>();
            var mazes = new Mock<IMazeService>();
            var auth = new Mock<IAuthService>();
            var nav = new Mock<INavigationService>();
            var avatar = new Mock<IAvatarService>();
            var dialog = new Mock<IDialogService>();

            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(new UserProfile { Id = "me", Username = "Me" });
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem>());
            scores.Setup(s => s.GetScoreHistoryAsync(It.IsAny<int?>(), It.IsAny<int?>())).ReturnsAsync(EmptyBoard());
            scores.Setup(s => s.GetLeaderboardAsync(
                It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(),
                It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>())).ReturnsAsync(EmptyBoard());
            scores.Setup(s => s.GetBoardDatesAsync(It.IsAny<string>()))
                  .ReturnsAsync(new BoardDatesResponse { Dates = new List<string>() });

            return new Mocks(scores, gameLib, mazes, auth, nav, avatar, dialog);
        }

        private static LeaderboardsViewModel NewVm(Mocks m) =>
            new(m.Scores.Object, m.GameLib.Object, m.Mazes.Object, m.Auth.Object, m.Nav.Object, m.Avatar.Object, m.Dialog.Object);

        private static (LeaderboardsViewModel vm, Mock<IScoresService> scores, Mock<IGameLibraryService> gameLib, Mock<IMazeService> mazes, Mock<INavigationService> nav)
            BuildVm()
        {
            Mocks m = CreateMocks();
            return (NewVm(m), m.Scores, m.GameLib, m.Mazes, m.Nav);
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

        private static GamePlayResponse GameDef(string id, string name = "My Game", string ownerId = "owner", string rotation = "static") =>
            new() { Id = id, Name = name, OwnerId = ownerId, Rotation = rotation };

        // ---- discovery + default selection ----------------------------------

        [Fact]
        public void GameTypes_AreLabelledMazesAnd3dGames()
        {
            var (vm, _, _, _, _) = BuildVm();
            string[] expected = { "Mazes", "3D Games" };
            Assert.Equal(expected, vm.GameTypes.Select(t => t.Label).ToArray());
        }

        [Fact]
        public async Task LoadBoard_3dGame_ResolvesRowAvatarBytesPerPlayer()
        {
            Mocks m = CreateMocks();
            // Most-recent run is a stored 3D game → defaults to its board (Player
            // column shown), so player avatars are resolved.
            m.Scores.Setup(s => s.GetScoreHistoryAsync(It.IsAny<int?>(), It.IsAny<int?>()))
                  .ReturnsAsync(Board(new[] { ChallengeRow("def:g1") }, false));
            m.GameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1"));

            var withAvatar = ChallengeRow("def:g1", userId: "alice", username: "alice");
            withAvatar.AvatarUpdatedAt = "2026-06-16T12:00:00Z";
            var withoutAvatar = ChallengeRow("def:g1", userId: "bob", username: "bob");
            m.Scores.Setup(s => s.GetLeaderboardAsync(
                It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(),
                It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { withAvatar, withoutAvatar }, false));

            byte[] aliceBytes = { 9, 8, 7 };
            m.Avatar.Setup(s => s.TryLoadAvatarBytesAsync("alice", "2026-06-16T12:00:00Z")).ReturnsAsync(aliceBytes);

            var vm = NewVm(m);
            await vm.InitializeCommand.ExecuteAsync(null);

            LeaderboardRow aliceRow = vm.Rows.First(r => r.UserId == "alice");
            LeaderboardRow bobRow = vm.Rows.First(r => r.UserId == "bob");
            Assert.Same(aliceBytes, aliceRow.AvatarBytes);
            Assert.Null(bobRow.AvatarBytes);
            m.Avatar.Verify(s => s.TryLoadAvatarBytesAsync("bob", It.IsAny<string?>()), Times.Never);
        }

        [Fact]
        public async Task Initialize_DefaultsToMostRecentMaze_MyMazesNoUsernames()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
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
        public async Task Initialize_DefaultsTo3dGame_WhenMostRecentIsDefChallenge()
        {
            var (vm, scores, gameLib, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0))
                  .ReturnsAsync(Board(new[] { ChallengeRow("def:g9") }, false));
            gameLib.Setup(g => g.GetGameDefinitionAsync("g9")).ReturnsAsync(GameDef("g9", name: "Nine"));

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Equal(LeaderboardGameType.Play3d, vm.SelectedGameType!.Kind);
            Assert.Equal("g9", vm.PickedGame!.Id);
            Assert.Equal("Nine", vm.PickedGameLabel);
            Assert.True(vm.ShowPlayerColumn);
            gameLib.Verify(g => g.GetGameDefinitionAsync("g9"), Times.Once);
            scores.Verify(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge == "def:g9"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), true), Times.Once);
        }

        [Fact]
        public async Task Initialize_PreselectGame_WinsOverMostRecent()
        {
            var (vm, scores, gameLib, _, _) = BuildVm();
            // Most-recent is a maze, but a card preselected a 3D game → the game wins.
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            gameLib.Setup(g => g.GetGameDefinitionAsync("card")).ReturnsAsync(GameDef("card", name: "From Card"));
            vm.SetPreselectGame("card");

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Equal(LeaderboardGameType.Play3d, vm.SelectedGameType!.Kind);
            Assert.Equal("card", vm.PickedGame!.Id);
        }

        [Fact]
        public async Task Initialize_DefaultsTo3dGamesEmpty_WhenNoMazesNoHistory()
        {
            var (vm, _, _, _, _) = BuildVm();

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Equal(LeaderboardGameType.Play3d, vm.SelectedGameType!.Kind);
            Assert.Null(vm.PickedGame);
            Assert.False(vm.CanPlay);
            Assert.Equal("Choose a game to see its leaderboard.", vm.StatusMessage);
        }

        [Fact]
        public async Task Games_ListAllMazes_IncludingUnplayed()
        {
            var (vm, _, _, mazes, _) = BuildVm();
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
            var (vm, scores, _, mazes, _) = BuildVm();
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

        [Fact]
        public async Task Cascade_SwitchingTo3dGames_ClearsMazesAndDisablesPlayUntilPicked()
        {
            var (vm, _, _, mazes, _) = BuildVm();
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            await vm.InitializeCommand.ExecuteAsync(null);   // defaults to Mazes m1

            vm.SelectedGameType = vm.GameTypes.First(t => t.Kind == LeaderboardGameType.Play3d);

            Assert.Empty(vm.Games);
            Assert.Null(vm.PickedGame);
            Assert.False(vm.CanPlay);
            Assert.True(vm.Is3dType);
        }

        // ---- board paging + metric ------------------------------------------

        [Fact]
        public async Task LoadMore_AppendsRows_AndAdvancesOffset()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
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
            Assert.Equal(25, vm.Rows[24].Rank);
            scores.Verify(s => s.GetLeaderboardAsync(It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(),
                It.IsAny<SortDirection?>(), It.IsAny<int?>(), 20, It.IsAny<bool?>()), Times.Once);
        }

        [Fact]
        public async Task Refresh_RefetchesCurrentBoard()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            await vm.InitializeCommand.ExecuteAsync(null);   // Mazes m1 → board loaded once

            await vm.RefreshCommand.ExecuteAsync(null);

            scores.Verify(s => s.GetLeaderboardAsync(It.IsAny<ScoreSubject>(), It.IsAny<ScoreMetric?>(),
                It.IsAny<SortDirection?>(), It.IsAny<int?>(), 0, It.IsAny<bool?>()), Times.Exactly(2));
        }

        [Fact]
        public async Task MetricSwitch_ReloadsWithNewMetric()
        {
            var (vm, scores, _, mazes, _) = BuildVm();
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            await vm.InitializeCommand.ExecuteAsync(null);   // Mazes m1, time metric

            await vm.SelectScoreMetricCommand.ExecuteAsync(null);

            Assert.True(vm.IsScoreMetricSelected);
            Assert.False(vm.IsTimeMetricSelected);
            scores.Verify(s => s.GetLeaderboardAsync(It.IsAny<ScoreSubject>(), ScoreMetric.Score,
                It.IsAny<SortDirection?>(), It.IsAny<int?>(), 0, It.IsAny<bool?>()), Times.Once);
        }

        // ---- highlight / player gating + empty state ------------------------

        [Fact]
        public async Task Highlight_On3dGameBoard_OnlyForCallerRows()
        {
            var (vm, scores, gameLib, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("def:g1") }, false));
            gameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1"));
            scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge != null),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[]
                  {
                      ChallengeRow("def:g1", userId: "other", username: "Rival"),
                      ChallengeRow("def:g1", userId: "me", username: "Me"),
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
            var (vm, scores, _, mazes, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Empty(vm.Rows);
            Assert.True(vm.ShowStatusMessage);
            Assert.Equal("No winning scores yet", vm.StatusMessage);
        }

        // ---- Play button -----------------------------------------------------

        [Fact]
        public async Task Play_3dGame_NavigatesWithDef()
        {
            var (vm, scores, gameLib, _, nav) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("def:g1") }, false));
            gameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1"));
            await vm.InitializeCommand.ExecuteAsync(null);   // 3D game g1

            await vm.PlayCommand.ExecuteAsync(null);

            nav.Verify(n => n.GoToAsync("Play3dGamePage",
                It.Is<IDictionary<string, object>?>(d => d != null && (string)d["def"] == "g1")), Times.Once);
        }

        [Fact]
        public async Task Play_MyMazes_LoadsMazeAndNavigatesWithSettings()
        {
            var (vm, scores, _, mazes, nav) = BuildVm();
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            var full = new MazeItem { ID = "m1", Name = "One", GameSettings = new MazeGameSettings() };
            mazes.Setup(m => m.GetMazeItem("m1")).ReturnsAsync(full);
            await vm.InitializeCommand.ExecuteAsync(null);

            await vm.PlayCommand.ExecuteAsync(null);

            mazes.Verify(m => m.GetMazeItem("m1"), Times.Once);
            nav.Verify(n => n.GoToAsync("Play3dGamePage",
                It.Is<IDictionary<string, object>?>(d => d != null
                    && ReferenceEquals(d["MazeItem"], full) && d.ContainsKey("LaunchSettings"))), Times.Once);
        }

        [Fact]
        public async Task Play_MyMazes_UnplayableMaze_ShowsAlertAndDoesNotNavigate()
        {
            Mocks m = CreateMocks();
            m.Mazes.Setup(x => x.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            var unplayable = new MazeItem { ID = "m1", Name = "One", Definition = new Api.Maze(3, 3) };
            m.Mazes.Setup(x => x.GetMazeItem("m1")).ReturnsAsync(unplayable);
            var vm = NewVm(m);
            await vm.InitializeCommand.ExecuteAsync(null);

            await vm.PlayCommand.ExecuteAsync(null);

            m.Dialog.Verify(d => d.ShowAlert("MAZE", It.Is<string>(s => s.Contains("Cannot play maze")), "OK"), Times.Once);
            m.Nav.Verify(n => n.GoToAsync(It.IsAny<string>(), It.IsAny<IDictionary<string, object>?>()), Times.Never);
        }

        [Fact]
        public async Task CanPlay_FalseWhenMazesSelectedButNone()
        {
            var (vm, scores, gameLib, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("def:g1") }, false));
            gameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1"));
            await vm.InitializeCommand.ExecuteAsync(null);   // 3D game → playable
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
            var (vm, scores, _, mazes, _) = BuildVm();
            mazes.Setup(m => m.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { MazeRow("m1", userId: "me") }, false));

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.HasPlayed);
            Assert.Equal("↻ Play Again", vm.PlayLabel);
        }

        // ---- Daily board-date picker ----------------------------------------

        [Fact]
        public async Task Daily_OffersTodayThenPastDays_DefaultsToMostRecentWithRuns()
        {
            var (vm, scores, gameLib, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("def:g1:2020-06-20") }, false));
            gameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1", rotation: "daily"));
            scores.Setup(s => s.GetBoardDatesAsync("g1"))
                  .ReturnsAsync(new BoardDatesResponse { Dates = new List<string> { "2020-06-20", "2020-06-12" } });

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.IsDailyGame);
            // Today pinned first, then the days with boards, most-recent first.
            string[] expected = { "Today", "20 Jun 2020", "12 Jun 2020" };
            Assert.Equal(expected, vm.BoardDates.Select(o => o.Label).ToArray());
            // Default = the most-recent day that has runs (not Today, which has none).
            Assert.Equal("2020-06-20", vm.SelectedBoardDate!.DateUtc);
            scores.Verify(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge == "def:g1:2020-06-20"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), true), Times.Once);
        }

        [Fact]
        public async Task Daily_TodayHasRuns_PinnedOnce_AndIsTheDefault()
        {
            string today = GameChallenge.TodayUtc();
            var (vm, scores, gameLib, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow($"def:g1:{today}") }, false));
            gameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1", rotation: "daily"));
            scores.Setup(s => s.GetBoardDatesAsync("g1"))
                  .ReturnsAsync(new BoardDatesResponse { Dates = new List<string> { today, "2020-06-12" } });

            await vm.InitializeCommand.ExecuteAsync(null);

            // Today isn't duplicated — the pin covers it — and it's the default.
            string[] expected = { "Today", "12 Jun 2020" };
            Assert.Equal(expected, vm.BoardDates.Select(o => o.Label).ToArray());
            Assert.Equal("Today", vm.SelectedBoardDate!.Label);
            Assert.Equal(today, vm.SelectedBoardDate.DateUtc);
        }

        [Fact]
        public async Task Daily_NoRuns_OffersOnlyTodayAndDefaultsToIt()
        {
            string today = GameChallenge.TodayUtc();
            var (vm, scores, gameLib, _, _) = BuildVm();
            gameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1", rotation: "daily"));
            scores.Setup(s => s.GetBoardDatesAsync("g1")).ReturnsAsync(new BoardDatesResponse { Dates = new List<string>() });
            vm.SetPreselectGame("g1");

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.IsDailyGame);
            Assert.Single(vm.BoardDates);
            Assert.Equal("Today", vm.SelectedBoardDate!.Label);
            Assert.Equal(today, vm.SelectedBoardDate.DateUtc);
        }

        [Fact]
        public async Task Daily_SelectingAnEarlierDay_ReloadsThatDaysBoard()
        {
            var (vm, scores, gameLib, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("def:g1:2020-06-20") }, false));
            gameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1", rotation: "daily"));
            scores.Setup(s => s.GetBoardDatesAsync("g1"))
                  .ReturnsAsync(new BoardDatesResponse { Dates = new List<string> { "2020-06-20", "2020-06-12" } });
            await vm.InitializeCommand.ExecuteAsync(null);

            vm.SelectedBoardDate = vm.BoardDates.First(o => o.DateUtc == "2020-06-12");

            scores.Verify(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge == "def:g1:2020-06-12"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), true), Times.Once);
        }

        [Fact]
        public async Task Static3dGame_HasNoBoardDatePicker()
        {
            var (vm, scores, gameLib, _, _) = BuildVm();
            scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("def:g1") }, false));
            gameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1", rotation: "static"));

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.Is3dType);
            Assert.False(vm.IsDailyGame);
            Assert.Empty(vm.BoardDates);
            Assert.Null(vm.SelectedBoardDate);
            scores.Verify(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge == "def:g1"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), true), Times.Once);
        }

        // ---- Reset leaderboard ----------------------------------------------

        [Fact]
        public async Task Reset_MyMazesBoardWithRows_OfferedToOwnerEvenWhenNotAdmin()
        {
            Mocks m = CreateMocks();
            m.Scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            m.Mazes.Setup(x => x.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            m.Scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            var vm = NewVm(m);

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.ShowReset);
            Assert.True(vm.ResetBoardCommand.CanExecute(null));
        }

        [Fact]
        public async Task Reset_EmptyBoard_NotOffered()
        {
            Mocks m = CreateMocks();
            m.Scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            m.Mazes.Setup(x => x.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            var vm = NewVm(m);

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.Empty(vm.Rows);
            Assert.False(vm.ShowReset);
            Assert.False(vm.ResetBoardCommand.CanExecute(null));
        }

        [Fact]
        public async Task Reset_3dGameBoard_NotOfferedToNonAdminNonOwner()
        {
            Mocks m = CreateMocks();   // non-admin "me"; the game is owned by "other"
            m.Scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("def:g1") }, false));
            m.GameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1", ownerId: "other"));
            m.Scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge != null),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { ChallengeRow("def:g1", userId: "other", username: "Rival") }, false));
            var vm = NewVm(m);

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.ShowPlayerColumn);
            Assert.False(vm.ShowReset);
        }

        [Fact]
        public async Task Reset_3dGameBoard_OfferedToOwner()
        {
            Mocks m = CreateMocks();   // non-admin "me", but "me" owns the game
            m.Scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("def:g1") }, false));
            m.GameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1", ownerId: "me"));
            m.Scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge != null),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { ChallengeRow("def:g1", userId: "me", username: "Me") }, false));
            var vm = NewVm(m);

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.ShowReset);
        }

        [Fact]
        public async Task Reset_3dGameBoard_OfferedToAdmin()
        {
            Mocks m = CreateMocks();
            m.Auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(new UserProfile { Id = "me", Username = "Me", IsAdmin = true });
            m.Scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { ChallengeRow("def:g1") }, false));
            m.GameLib.Setup(g => g.GetGameDefinitionAsync("g1")).ReturnsAsync(GameDef("g1", ownerId: "other"));
            m.Scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.Challenge != null),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { ChallengeRow("def:g1", userId: "other", username: "Rival") }, false));
            var vm = NewVm(m);

            await vm.InitializeCommand.ExecuteAsync(null);

            Assert.True(vm.ShowReset);
        }

        [Fact]
        public async Task Reset_Confirmed_ClearsSubjectAndReloadsEmpty()
        {
            Mocks m = CreateMocks();
            m.Scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            m.Mazes.Setup(x => x.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            m.Scores.SetupSequence(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { MazeRow("m1") }, false))
                  .ReturnsAsync(EmptyBoard());
            m.Dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(true);
            m.Scores.Setup(s => s.ClearLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1"))).ReturnsAsync(1L);
            var vm = NewVm(m);
            await vm.InitializeCommand.ExecuteAsync(null);
            Assert.True(vm.ShowReset);

            await vm.ResetBoardCommand.ExecuteAsync(null);

            m.Dialog.Verify(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(), "Reset", "Cancel", true), Times.Once);
            m.Scores.Verify(s => s.ClearLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1")), Times.Once);
            Assert.Empty(vm.Rows);
            Assert.False(vm.ShowReset);
        }

        [Fact]
        public async Task Reset_Cancelled_DoesNotClear()
        {
            Mocks m = CreateMocks();
            m.Scores.Setup(s => s.GetScoreHistoryAsync(1, 0)).ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            m.Mazes.Setup(x => x.GetMazeItems(false)).ReturnsAsync(new List<MazeItem> { MazeItem("m1", "One") });
            m.Scores.Setup(s => s.GetLeaderboardAsync(It.Is<ScoreSubject>(x => x.MazeId == "m1"),
                It.IsAny<ScoreMetric?>(), It.IsAny<SortDirection?>(), It.IsAny<int?>(), It.IsAny<int?>(), It.IsAny<bool?>()))
                  .ReturnsAsync(Board(new[] { MazeRow("m1") }, false));
            m.Dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(false);
            var vm = NewVm(m);
            await vm.InitializeCommand.ExecuteAsync(null);

            await vm.ResetBoardCommand.ExecuteAsync(null);

            m.Scores.Verify(s => s.ClearLeaderboardAsync(It.IsAny<ScoreSubject>()), Times.Never);
            Assert.Single(vm.Rows);
        }
    }
}
