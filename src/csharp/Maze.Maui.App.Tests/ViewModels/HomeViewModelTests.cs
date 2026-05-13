using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Maze.Maui.App.Views;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the Home view model: each tile command navigates to the
    /// expected route via INavigationService.
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
        public async Task PlayRandom3dCommand_NavigatesToPlay3dGamePage()
        {
            var (vm, nav) = BuildVm();

            await vm.PlayRandom3dCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync(nameof(Play3dGamePage), It.IsAny<IDictionary<string, object>?>()),
                Times.Once);
        }

        [Fact]
        public async Task GoToDesignAndPlayCommand_NavigatesToMazesPage()
        {
            var (vm, nav) = BuildVm();

            await vm.GoToDesignAndPlayCommand.ExecuteAsync(null);

            nav.Verify(
                n => n.GoToAsync("MazesPage", It.IsAny<IDictionary<string, object>?>()),
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
