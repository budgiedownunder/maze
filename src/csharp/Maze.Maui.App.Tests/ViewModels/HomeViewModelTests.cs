using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Maze.Maui.App.Views;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the Home view model: the Design and Play tile navigates
    /// directly; the Play 3D tile first prompts for a difficulty and only
    /// navigates (with that difficulty) once the user confirms.
    /// </summary>
    public class HomeViewModelTests
    {
        private static (HomeViewModel vm, Mock<INavigationService> nav, Mock<IDialogService> dialog) BuildVm()
        {
            var nav = new Mock<INavigationService>();
            var dialog = new Mock<IDialogService>();
            var vm = new HomeViewModel(nav.Object, dialog.Object);
            return (vm, nav, dialog);
        }

        [Fact]
        public async Task PlayRandom3dCommand_PromptsForDifficultyThenNavigatesWithIt()
        {
            var (vm, nav, dialog) = BuildVm();
            dialog.Setup(d => d.ShowPlay3dDifficultyAsync()).ReturnsAsync(Difficulty.Hard);

            await vm.PlayRandom3dCommand.ExecuteAsync(null);

            dialog.Verify(d => d.ShowPlay3dDifficultyAsync(), Times.Once);
            nav.Verify(
                n => n.GoToAsync(
                    nameof(Play3dGamePage),
                    It.Is<IDictionary<string, object>>(p =>
                        p.ContainsKey("difficulty") && (string)p["difficulty"] == "hard")),
                Times.Once);
        }

        [Fact]
        public async Task PlayRandom3dCommand_UserCancelsPicker_DoesNotNavigate()
        {
            var (vm, nav, dialog) = BuildVm();
            dialog.Setup(d => d.ShowPlay3dDifficultyAsync()).ReturnsAsync((Difficulty?)null);

            await vm.PlayRandom3dCommand.ExecuteAsync(null);

            dialog.Verify(d => d.ShowPlay3dDifficultyAsync(), Times.Once);
            nav.Verify(
                n => n.GoToAsync(It.IsAny<string>(), It.IsAny<IDictionary<string, object>?>()),
                Times.Never);
        }

        [Fact]
        public async Task GoTo3dGamesCommand_NavigatesToHub()
        {
            var (vm, nav, _) = BuildVm();

            await vm.GoTo3dGamesCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("Play3dHubPage", It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public async Task GoToMazesCommand_NavigatesToMazesPage()
        {
            var (vm, nav, _) = BuildVm();

            await vm.GoToMazesCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("MazesPage", It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public async Task GoToLeaderboardsCommand_NavigatesToLeaderboardsPage()
        {
            var (vm, nav, _) = BuildVm();

            await vm.GoToLeaderboardsCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("LeaderboardsPage", It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public void Title_IsHome()
        {
            var (vm, _, _) = BuildVm();
            Assert.Equal("Home", vm.Title);
        }
    }
}
