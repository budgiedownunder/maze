using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the Home view model: each tile navigates directly to its
    /// destination route, and Today's Challenge resolves + launches the daily game.
    /// </summary>
    public class HomeViewModelTests
    {
        private static (HomeViewModel vm, Mock<INavigationService> nav, Mock<IGameLibraryService> gameLib, Mock<IDialogService> dialog) BuildVm()
        {
            var nav = new Mock<INavigationService>();
            var gameLib = new Mock<IGameLibraryService>();
            var dialog = new Mock<IDialogService>();
            var vm = new HomeViewModel(nav.Object, gameLib.Object, dialog.Object);
            return (vm, nav, gameLib, dialog);
        }

        private static FeaturedGameItemsListResponse Featured(params FeaturedGameItem[] items) =>
            new() { Items = items.ToList(), HasMore = false };

        private static FeaturedGameItem CollectionItem(string id, string name) =>
            new() { Kind = "collection", Collection = new GameCollection { Id = id, Name = name } };

        private static GameDefinition Def(string id, string rotation = "static") =>
            new() { Id = id, Name = id, Rotation = rotation };

        [Fact]
        public async Task GoTo3dGamesCommand_NavigatesToHub()
        {
            var (vm, nav, _, _) = BuildVm();

            await vm.GoTo3dGamesCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("Play3dHubPage", It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public async Task GoToMazesCommand_NavigatesToMazesPage()
        {
            var (vm, nav, _, _) = BuildVm();

            await vm.GoToMazesCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("MazesPage", It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public async Task GoToLeaderboardsCommand_NavigatesToLeaderboardsPage()
        {
            var (vm, nav, _, _) = BuildVm();

            await vm.GoToLeaderboardsCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("LeaderboardsPage", It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public void Title_IsHome()
        {
            var (vm, _, _, _) = BuildVm();
            Assert.Equal("Home", vm.Title);
        }

        [Fact]
        public async Task PlayTodaysChallenge_LaunchesTheDailyMember()
        {
            var (vm, nav, gameLib, _) = BuildVm();
            gameLib.Setup(g => g.GetFeaturedGameItemsAsync(It.IsAny<int?>(), It.IsAny<int?>()))
                   .ReturnsAsync(Featured(CollectionItem("col-daily", "Daily Challenges")));
            gameLib.Setup(g => g.GetGameCollectionAsync("col-daily"))
                   .ReturnsAsync(new GameCollectionDetailResponse { Definitions = { Def("g-static"), Def("g-daily", "daily") } });

            await vm.PlayTodaysChallengeCommand.ExecuteAsync(null);

            // Launches the daily-rotation member, not the first (static) one.
            nav.Verify(n => n.GoToAsync("Play3dGamePage",
                It.Is<IDictionary<string, object>?>(d => d != null && (string)d["def"] == "g-daily")), Times.Once);
        }

        [Fact]
        public async Task PlayTodaysChallenge_NoDailyChallengesCollection_Alerts()
        {
            var (vm, nav, gameLib, dialog) = BuildVm();
            gameLib.Setup(g => g.GetFeaturedGameItemsAsync(It.IsAny<int?>(), It.IsAny<int?>()))
                   .ReturnsAsync(Featured(CollectionItem("col-other", "Something Else")));

            await vm.PlayTodaysChallengeCommand.ExecuteAsync(null);

            dialog.Verify(d => d.ShowAlert("Daily Challenge", It.IsAny<string>(), "OK"), Times.Once);
            nav.Verify(n => n.GoToAsync(It.IsAny<string>(), It.IsAny<IDictionary<string, object>?>()), Times.Never);
        }

        [Fact]
        public async Task PlayTodaysChallenge_EmptyCollection_Alerts()
        {
            var (vm, nav, gameLib, dialog) = BuildVm();
            gameLib.Setup(g => g.GetFeaturedGameItemsAsync(It.IsAny<int?>(), It.IsAny<int?>()))
                   .ReturnsAsync(Featured(CollectionItem("col-daily", "Daily Challenges")));
            gameLib.Setup(g => g.GetGameCollectionAsync("col-daily"))
                   .ReturnsAsync(new GameCollectionDetailResponse());

            await vm.PlayTodaysChallengeCommand.ExecuteAsync(null);

            dialog.Verify(d => d.ShowAlert("Daily Challenge", It.IsAny<string>(), "OK"), Times.Once);
            nav.Verify(n => n.GoToAsync(It.IsAny<string>(), It.IsAny<IDictionary<string, object>?>()), Times.Never);
        }

        [Fact]
        public async Task PlayTodaysChallenge_LookupFails_Alerts()
        {
            var (vm, nav, gameLib, dialog) = BuildVm();
            gameLib.Setup(g => g.GetFeaturedGameItemsAsync(It.IsAny<int?>(), It.IsAny<int?>()))
                   .ThrowsAsync(new HttpRequestException("network"));

            await vm.PlayTodaysChallengeCommand.ExecuteAsync(null);

            dialog.Verify(d => d.ShowAlert("Daily Challenge", It.IsAny<string>(), "OK"), Times.Once);
            nav.Verify(n => n.GoToAsync(It.IsAny<string>(), It.IsAny<IDictionary<string, object>?>()), Times.Never);
        }
    }
}
