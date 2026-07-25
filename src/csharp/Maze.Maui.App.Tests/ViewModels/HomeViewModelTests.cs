using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the Home view model: each tile navigates directly to its
    /// destination route.
    /// </summary>
    public class HomeViewModelTests
    {
        private static (HomeViewModel vm, Mock<INavigationService> nav) BuildVm()
        {
            var nav = new Mock<INavigationService>();
            var vm = new HomeViewModel(nav.Object);
            return (vm, nav);
        }

        [Fact]
        public async Task GoTo3dGamesCommand_NavigatesToHub()
        {
            var (vm, nav) = BuildVm();

            await vm.GoTo3dGamesCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("Play3dHubPage", It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public async Task GoToMazesCommand_NavigatesToMazesPage()
        {
            var (vm, nav) = BuildVm();

            await vm.GoToMazesCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("MazesPage", It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public async Task GoToLeaderboardsCommand_NavigatesToLeaderboardsPage()
        {
            var (vm, nav) = BuildVm();

            await vm.GoToLeaderboardsCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("LeaderboardsPage", It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public void Title_IsHome()
        {
            var (vm, _) = BuildVm();
            Assert.Equal("Home", vm.Title);
        }
    }
}
