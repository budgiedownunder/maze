using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Maze.Maui.Services;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the maze-editor view model. Most editor logic (cell
    /// selection, range mode, insert/delete row/col) lives in the
    /// InteractiveGrid view, not the VM — so this surface is narrow:
    /// SaveMaze (new vs existing), RefreshMaze confirmation/cancellation,
    /// dirty-state propagation through CanSave/CanRefresh, and
    /// command-to-event routing.
    /// </summary>
    public class MazeViewModelTests
    {
        private static (MazeViewModel vm, Mock<IDeviceTypeService> device, Mock<IDialogService> dialog, Mock<IMazeService> service)
            BuildVm()
        {
            var device = new Mock<IDeviceTypeService>();
            var dialog = new Mock<IDialogService>();
            var service = new Mock<IMazeService>();
            var vm = new MazeViewModel(device.Object, dialog.Object, service.Object);
            return (vm, device, dialog, service);
        }

        // ---- SaveMaze (draft / unstored) ------------------------------------

        [Fact]
        public async Task SaveMaze_Draft_PromptsForNameCallsCreateAndPublishesMessage()
        {
            var (vm, _, dialog, service) = BuildVm();
            vm.IsStored = false;
            vm.MazeItem = new MazeItem();
            dialog.Setup(d => d.DisplayPrompt(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(),
                                              It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string?>(),
                                              It.IsAny<int>(), It.IsAny<Keyboard?>(), It.IsAny<string?>(),
                                              It.IsAny<bool>(), It.IsAny<bool>()))
                  .ReturnsAsync("My Maze");
            int newMessages = 0;
            object recipient = new();
            WeakReferenceMessenger.Default.Register<NewMazeItemMessage>(recipient, (_, _) => newMessages++);
            try
            {
                var result = await vm.SaveMaze(new Api.Maze(3, 3));

                Assert.True(result);
            }
            finally
            {
                WeakReferenceMessenger.Default.Unregister<NewMazeItemMessage>(recipient);
            }

            service.Verify(s => s.CreateMazeItem(It.Is<MazeItem>(i => i.Name == "My Maze")), Times.Once);
            service.Verify(s => s.UpdateMazeItem(It.IsAny<MazeItem>()), Times.Never);
            Assert.Equal(1, newMessages);
            Assert.False(vm.IsDirty);
        }

        [Fact]
        public async Task SaveMaze_Draft_UserCancelsNamePrompt_ReturnsFalseWithoutServiceCall()
        {
            var (vm, _, dialog, service) = BuildVm();
            vm.IsStored = false;
            dialog.Setup(d => d.DisplayPrompt(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(),
                                              It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string?>(),
                                              It.IsAny<int>(), It.IsAny<Keyboard?>(), It.IsAny<string?>(),
                                              It.IsAny<bool>(), It.IsAny<bool>()))
                  .Returns(Task.FromResult<string>(null!));

            var result = await vm.SaveMaze(new Api.Maze(3, 3));

            Assert.False(result);
            service.Verify(s => s.CreateMazeItem(It.IsAny<MazeItem>()), Times.Never);
        }

        // ---- SaveMaze (stored) ----------------------------------------------

        [Fact]
        public async Task SaveMaze_Stored_CallsUpdateAndDoesNotPublishMessage()
        {
            var (vm, _, _, service) = BuildVm();
            vm.IsStored = true;
            vm.MazeItem = new MazeItem { ID = "abc", Name = "My Maze" };
            int newMessages = 0;
            object recipient = new();
            WeakReferenceMessenger.Default.Register<NewMazeItemMessage>(recipient, (_, _) => newMessages++);
            try
            {
                var result = await vm.SaveMaze(new Api.Maze(3, 3));

                Assert.True(result);
            }
            finally
            {
                WeakReferenceMessenger.Default.Unregister<NewMazeItemMessage>(recipient);
            }

            service.Verify(s => s.UpdateMazeItem(It.Is<MazeItem>(i => i.ID == "abc" && i.Name == "My Maze")), Times.Once);
            service.Verify(s => s.CreateMazeItem(It.IsAny<MazeItem>()), Times.Never);
            Assert.Equal(0, newMessages);
            Assert.False(vm.IsDirty);
        }

        [Fact]
        public async Task SaveMaze_ServiceFails_AlertsAndReturnsFalse()
        {
            var (vm, _, dialog, service) = BuildVm();
            vm.IsStored = true;
            vm.MazeItem = new MazeItem { ID = "abc", Name = "My Maze" };
            service.Setup(s => s.UpdateMazeItem(It.IsAny<MazeItem>())).ThrowsAsync(new HttpRequestException("network"));

            var result = await vm.SaveMaze(new Api.Maze(3, 3));

            Assert.False(result);
            dialog.Verify(d => d.ShowAlert("Error", It.Is<string>(m => m.Contains("Failed to save maze")), "OK"), Times.Once);
        }

        // ---- RefreshMaze ----------------------------------------------------

        [Fact]
        public async Task RefreshMaze_UserCancels_DoesNotCallService()
        {
            var (vm, _, dialog, service) = BuildVm();
            vm.MazeItem = new MazeItem { ID = "abc" };
            dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(),
                                                  It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(false);

            var result = await vm.RefreshMaze();

            Assert.False(result);
            service.Verify(s => s.GetMazeItem(It.IsAny<string>()), Times.Never);
        }

        [Fact]
        public async Task RefreshMaze_UserConfirms_FetchesItemAndClearsDirty()
        {
            var (vm, _, dialog, service) = BuildVm();
            var fresh = new Api.Maze(5, 5);
            vm.IsStored = true;
            vm.MazeItem = new MazeItem { ID = "abc", Name = "Old" };
            vm.NotifyMazeChanged(); // mark dirty before refresh
            Assert.True(vm.IsDirty);
            dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(),
                                                  It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(true);
            service.Setup(s => s.GetMazeItem("abc"))
                   .ReturnsAsync(new MazeItem { ID = "abc", Name = "Fresh", Definition = fresh });

            var result = await vm.RefreshMaze();

            Assert.True(result);
            Assert.False(vm.IsDirty);
            Assert.Equal("Fresh", vm.MazeItem.Name);
            Assert.Same(fresh, vm.MazeItem.Definition);
        }

        [Fact]
        public async Task RefreshMaze_ServiceFails_AlertsAndReturnsFalse()
        {
            var (vm, _, dialog, service) = BuildVm();
            vm.MazeItem = new MazeItem { ID = "abc" };
            dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(),
                                                  It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(true);
            service.Setup(s => s.GetMazeItem(It.IsAny<string>())).ThrowsAsync(new HttpRequestException("network"));

            var result = await vm.RefreshMaze();

            Assert.False(result);
            dialog.Verify(d => d.ShowAlert("Error", It.Is<string>(m => m.Contains("Failed to refresh maze")), "OK"), Times.Once);
        }

        // ---- NotifyMazeChanged + dirty state -------------------------------

        [Fact]
        public void NotifyMazeChanged_StoredMaze_FlipsDirtyAndEnablesSaveAndRefresh()
        {
            var (vm, _, _, _) = BuildVm();
            vm.IsStored = true;

            vm.NotifyMazeChanged();

            Assert.True(vm.IsDirty);
            Assert.True(vm.CanSave);
            Assert.True(vm.CanRefresh);
        }

        [Fact]
        public void NotifyMazeChanged_UnstoredMaze_FlipsDirtyAndEnablesSaveOnly()
        {
            var (vm, _, _, _) = BuildVm();
            vm.IsStored = false;
            vm.CanRefresh = false; // baseline: a draft has nothing to refresh from

            vm.NotifyMazeChanged();

            Assert.True(vm.IsDirty);
            Assert.True(vm.CanSave);
            // CanRefresh is only updated when IsStored — a draft stays at its baseline.
            Assert.False(vm.CanRefresh);
        }

        // ---- Command-to-event routing --------------------------------------

        [Fact]
        public async Task EditingCommands_RaiseCorrespondingEventsAndMarkDirty()
        {
            // A representative slice of the ~14 *Requested events. Each
            // editing command also flips IsDirty true via UpdateCanSaveRefresh.
            var (vm, _, _, _) = BuildVm();
            vm.IsStored = true;

            int insertRows = 0, setWall = 0, clear = 0;
            vm.InsertRowsRequested += (_, _) => insertRows++;
            vm.SetWallRequested += (_, _) => setWall++;
            vm.ClearRequested += (_, _) => clear++;

            await vm.InsertRowsCommand.ExecuteAsync(null);
            await vm.SetWallCommand.ExecuteAsync(null);
            await vm.ClearCommand.ExecuteAsync(null);

            Assert.Equal(1, insertRows);
            Assert.Equal(1, setWall);
            Assert.Equal(1, clear);
            Assert.True(vm.IsDirty);
        }

        [Fact]
        public async Task NonEditingCommands_RaiseCorrespondingEvents_WithoutMarkingDirty()
        {
            // Solve / WalkSolution / Generate / SelectRange / Done / ClearSolution
            // raise events but do NOT call UpdateCanSaveRefresh — they are
            // view-only operations that shouldn't dirty the maze.
            var (vm, _, _, _) = BuildVm();

            int solve = 0, walk = 0, selectRange = 0, generate = 0;
            vm.SolveRequested += (_, _) => solve++;
            vm.WalkSolutionRequested += (_, _) => walk++;
            vm.SelectRangeRequested += (_, _) => selectRange++;
            vm.GenerateRequested += (_, _) => generate++;

            await vm.SolveCommand.ExecuteAsync(null);
            await vm.WalkSolutionCommand.ExecuteAsync(null);
            await vm.SelectRangeCommand.ExecuteAsync(null);
            await vm.GenerateCommand.ExecuteAsync(null);

            Assert.Equal(1, solve);
            Assert.Equal(1, walk);
            Assert.Equal(1, selectRange);
            Assert.Equal(1, generate);
            Assert.False(vm.IsDirty);
        }

        [Fact]
        public async Task SaveCommand_RaisesSaveRequested()
        {
            var (vm, _, _, _) = BuildVm();
            int saves = 0;
            vm.SaveRequested += (_, _) => saves++;

            await vm.SaveCommand.ExecuteAsync(null);

            Assert.Equal(1, saves);
        }

        [Fact]
        public async Task RefreshCommand_RaisesRefreshRequested()
        {
            var (vm, _, _, _) = BuildVm();
            int refreshes = 0;
            vm.RefreshRequested += (_, _) => refreshes++;

            await vm.RefreshCommand.ExecuteAsync(null);

            Assert.Equal(1, refreshes);
        }

        // ---- IsTouchOnlyDevice delegates to service -------------------------

        [Fact]
        public void IsTouchOnlyDevice_DelegatesToDeviceService()
        {
            var (vm, device, _, _) = BuildVm();
            device.Setup(d => d.IsTouchOnlyDevice()).Returns(true);

            Assert.True(vm.IsTouchOnlyDevice);

            device.Verify(d => d.IsTouchOnlyDevice(), Times.AtLeastOnce);
        }
    }
}
