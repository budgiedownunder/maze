using Maze.Api;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the maze-game view model. The production
    /// <c>Maze.Api.MazeGame</c> is sealed with a private constructor and
    /// loads native maze_wasm/maze_c libraries inside its
    /// <c>Create</c> factory — neither can run inside this non-MAUI
    /// test host. Coverage is therefore limited to the
    /// <c>MazeItem</c> setter, the <c>LoadStatus</c> /
    /// <c>HasLoadStatus</c> pair, the early-return guards in
    /// <c>StartGame</c> and <c>Move</c>, and <c>Cleanup</c>'s
    /// safe-when-uninitialized behaviour.
    /// </summary>
    public class MazeGameViewModelTests
    {
        private static (MazeGameViewModel vm, Mock<IDialogService> dialog, Mock<IMazeGridView> grid)
            BuildVm()
        {
            var dialog = new Mock<IDialogService>();
            var grid = new Mock<IMazeGridView>();
            var vm = new MazeGameViewModel(dialog.Object);
            return (vm, dialog, grid);
        }

        // ---- MazeItem setter -----------------------------------------------

        [Fact]
        public void MazeItemSetter_UpdatesTitleToItemName()
        {
            var (vm, _, _) = BuildVm();

            vm.MazeItem = new MazeItem { Name = "Dungeon" };

            Assert.Equal("Dungeon", vm.Title);
        }

        [Fact]
        public void MazeItemSetter_NullItem_ClearsTitle()
        {
            var (vm, _, _) = BuildVm();
            vm.MazeItem = new MazeItem { Name = "Dungeon" };

            vm.MazeItem = null;

            Assert.Equal("", vm.Title);
        }

        // ---- LoadStatus / HasLoadStatus ------------------------------------

        [Fact]
        public void HasLoadStatus_FalseWhenLoadStatusEmpty()
        {
            var (vm, _, _) = BuildVm();

            Assert.Equal("", vm.LoadStatus);
            Assert.False(vm.HasLoadStatus);
        }

        [Fact]
        public void LoadStatusChange_RaisesPropertyChangedForHasLoadStatus()
        {
            var (vm, _, _) = BuildVm();
            int hasLoadStatusChanges = 0;
            vm.PropertyChanged += (_, e) =>
            {
                if (e.PropertyName == nameof(MazeGameViewModel.HasLoadStatus)) hasLoadStatusChanges++;
            };

            vm.LoadStatus = "Loading...";

            Assert.True(vm.HasLoadStatus);
            Assert.True(hasLoadStatusChanges >= 1);
        }

        // ---- StartGame guard branches --------------------------------------

        [Fact]
        public void StartGame_NullMazeItem_SetsLoadStatusAndExitsEarly()
        {
            var (vm, _, grid) = BuildVm();
            // MazeItem is null by default — StartGame must not touch the grid.

            vm.StartGame(grid.Object);

            Assert.Equal("Maze not available.", vm.LoadStatus);
            Assert.True(vm.HasLoadStatus);
            grid.Verify(g => g.Initialize(It.IsAny<bool>(), It.IsAny<MazeItem?>()), Times.Never);
            grid.VerifySet(g => g.IsInteractionLocked = It.IsAny<bool>(), Times.Never);
        }

        [Fact]
        public void StartGame_MazeItemWithoutDefinition_SetsLoadStatusAndExitsEarly()
        {
            var (vm, _, grid) = BuildVm();
            vm.MazeItem = new MazeItem { Name = "Dungeon" }; // Definition still null.

            vm.StartGame(grid.Object);

            Assert.Equal("Maze not available.", vm.LoadStatus);
            grid.Verify(g => g.Initialize(It.IsAny<bool>(), It.IsAny<MazeItem?>()), Times.Never);
        }

        // ---- Move guard branches -------------------------------------------

        [Fact]
        public void Move_BeforeStartGame_DoesNothing()
        {
            // Without StartGame the internal _game is null — Move's first
            // guard returns immediately. Mocked grid must see no calls.
            var (vm, _, grid) = BuildVm();

            vm.Move(MazeGameDirection.Up);

            grid.Verify(g => g.SetVisitedDotAt(It.IsAny<int>(), It.IsAny<int>()), Times.Never);
            grid.Verify(g => g.SetPlayerAt(It.IsAny<int>(), It.IsAny<int>(), It.IsAny<MazeGameDirection>()), Times.Never);
            grid.Verify(g => g.SetPlayerCelebrate(It.IsAny<int>(), It.IsAny<int>()), Times.Never);
        }

        [Fact]
        public void Move_NoneDirection_DoesNothing()
        {
            var (vm, _, grid) = BuildVm();

            vm.Move(MazeGameDirection.None);

            grid.Verify(g => g.SetVisitedDotAt(It.IsAny<int>(), It.IsAny<int>()), Times.Never);
        }

        // ---- Cleanup safe when never started --------------------------------

        [Fact]
        public void Cleanup_BeforeStartGame_DoesNotThrow()
        {
            var (vm, _, _) = BuildVm();

            // No game session yet — Cleanup must be safe.
            var ex = Record.Exception(() => vm.Cleanup());

            Assert.Null(ex);
        }

        // ---- IsShowingResultPopup default ----------------------------------

        [Fact]
        public void IsShowingResultPopup_DefaultsToFalse()
        {
            var (vm, _, _) = BuildVm();

            Assert.False(vm.IsShowingResultPopup);
        }
    }
}
