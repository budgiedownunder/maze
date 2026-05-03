using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Maze.Maui.App.Views;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the maze-list page: load + sort, name uniqueness checks,
    /// add/remove/invalidate, the rename / duplicate / delete dialog flows
    /// (including retry-on-conflict loops), the cross-VM message handlers,
    /// and the IsBusy double-tap guard.
    /// </summary>
    public class MazesViewModelTests
    {
        private static (MazesViewModel vm, Mock<IDialogService> dialog, Mock<IMazeService> service, Mock<INavigationService> nav)
            BuildVm()
        {
            var dialog = new Mock<IDialogService>();
            var service = new Mock<IMazeService>();
            var nav = new Mock<INavigationService>();
            var vm = new MazesViewModel(dialog.Object, service.Object, nav.Object);
            return (vm, dialog, service, nav);
        }

        private static MazeItem MakeItem(string id, string name) =>
            new() { ID = id, Name = name };

        // ---- GetMazes -------------------------------------------------------

        [Fact]
        public async Task GetMazesAsync_PopulatesAndSortsByName()
        {
            var (vm, _, service, _) = BuildVm();
            service.Setup(s => s.GetMazeItems(true)).ReturnsAsync(new List<MazeItem>
            {
                MakeItem("1", "Charlie"),
                MakeItem("2", "Alpha"),
                MakeItem("3", "Bravo"),
            });

            await vm.GetMazesCommand.ExecuteAsync(null);

            Assert.True(vm.IsDataLoaded);
            Assert.Equal("Alpha,Bravo,Charlie", string.Join(",", vm.MazeItems.Select(i => i.Name)));
            Assert.Equal("", vm.LoadStatus);
        }

        [Fact]
        public async Task GetMazesAsync_EmptyResult_SetsNoMazesFoundStatus()
        {
            var (vm, _, service, _) = BuildVm();
            service.Setup(s => s.GetMazeItems(true)).ReturnsAsync(new List<MazeItem>());

            await vm.GetMazesCommand.ExecuteAsync(null);

            Assert.True(vm.IsDataLoaded);
            Assert.Empty(vm.MazeItems);
            Assert.Equal("No mazes found", vm.LoadStatus);
        }

        [Fact]
        public async Task GetMazesAsync_ServiceFails_AlertsUser()
        {
            var (vm, dialog, service, _) = BuildVm();
            service.Setup(s => s.GetMazeItems(true)).ThrowsAsync(new HttpRequestException("network"));

            await vm.GetMazesCommand.ExecuteAsync(null);

            dialog.Verify(d => d.ShowAlert("Error", It.Is<string>(m => m.Contains("Unable to load mazes")), "OK"), Times.Once);
            Assert.False(vm.IsDataLoaded);
        }

        // ---- IsBusy double-tap guard ---------------------------------------

        [Fact]
        public async Task GetMazesAsync_WhileBusy_ReturnsImmediately()
        {
            var (vm, _, service, _) = BuildVm();
            vm.IsBusy = true;

            await vm.GetMazesCommand.ExecuteAsync(null);

            service.Verify(s => s.GetMazeItems(It.IsAny<bool>()), Times.Never);
        }

        // ---- Collection helpers --------------------------------------------

        [Fact]
        public void AddNewItem_KeepsCollectionSortedByName()
        {
            var (vm, _, _, _) = BuildVm();
            vm.MazeItems.Add(MakeItem("1", "Alpha"));
            vm.MazeItems.Add(MakeItem("2", "Charlie"));

            vm.AddNewItem(MakeItem("3", "Bravo"));

            Assert.Equal("Alpha,Bravo,Charlie", string.Join(",", vm.MazeItems.Select(i => i.Name)));
        }

        [Fact]
        public void RemoveItem_RemovesByReference()
        {
            var (vm, _, _, _) = BuildVm();
            var target = MakeItem("1", "Alpha");
            vm.MazeItems.Add(target);
            vm.MazeItems.Add(MakeItem("2", "Bravo"));

            vm.RemoveItem(target);

            Assert.Equal("Bravo", string.Join(",", vm.MazeItems.Select(i => i.Name)));
        }

        [Fact]
        public void InvalidateData_ClearsCollectionAndResetsLoadedFlag()
        {
            var (vm, _, _, _) = BuildVm();
            vm.MazeItems.Add(MakeItem("1", "Alpha"));

            vm.InvalidateData();

            Assert.Empty(vm.MazeItems);
            Assert.False(vm.IsDataLoaded);
            Assert.Equal("No mazes found", vm.LoadStatus);
        }

        // ---- Rename ---------------------------------------------------------

        [Fact]
        public async Task RenameAsync_UserCancels_DoesNothing()
        {
            var (vm, dialog, service, _) = BuildVm();
            var item = MakeItem("1", "Alpha");
            vm.MazeItems.Add(item);
            dialog.Setup(d => d.DisplayPrompt(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(),
                                              It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string?>(),
                                              It.IsAny<int>(), It.IsAny<Keyboard?>(), It.IsAny<string?>(),
                                              It.IsAny<bool>(), It.IsAny<bool>()))
                  .Returns(Task.FromResult<string>(null!));

            await vm.RenameCommand.ExecuteAsync(item);

            service.Verify(s => s.RenameMazeItem(It.IsAny<MazeItem>(), It.IsAny<string>()), Times.Never);
        }

        [Fact]
        public async Task RenameAsync_HappyPath_CallsServiceAndKeepsCollectionSorted()
        {
            var (vm, dialog, service, _) = BuildVm();
            var alpha = MakeItem("1", "Alpha");
            var charlie = MakeItem("2", "Charlie");
            vm.MazeItems.Add(alpha);
            vm.MazeItems.Add(charlie);
            dialog.Setup(d => d.DisplayPrompt(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(),
                                              It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string?>(),
                                              It.IsAny<int>(), It.IsAny<Keyboard?>(), It.IsAny<string?>(),
                                              It.IsAny<bool>(), It.IsAny<bool>()))
                  .ReturnsAsync("Delta");
            // Simulate the service applying the rename to the item before returning.
            service.Setup(s => s.RenameMazeItem(alpha, "Delta"))
                   .Callback(() => alpha.Name = "Delta")
                   .Returns(Task.CompletedTask);

            await vm.RenameCommand.ExecuteAsync(alpha);

            service.Verify(s => s.RenameMazeItem(alpha, "Delta"), Times.Once);
            Assert.Equal("Charlie,Delta", string.Join(",", vm.MazeItems.Select(i => i.Name)));
        }

        [Fact]
        public async Task RenameAsync_NameAlreadyInUse_ShowsAlertAndDoesNotCallService()
        {
            var (vm, dialog, service, _) = BuildVm();
            var alpha = MakeItem("1", "Alpha");
            var bravo = MakeItem("2", "Bravo");
            vm.MazeItems.Add(alpha);
            vm.MazeItems.Add(bravo);
            // First prompt returns "Bravo" (taken), second returns null (cancel)
            // to break the retry loop. The production interface declares the
            // result as non-nullable Task<string> but returns null on cancel —
            // null! suppresses the nullability mismatch on the test side.
            dialog.SetupSequence(d => d.DisplayPrompt(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(),
                                                       It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string?>(),
                                                       It.IsAny<int>(), It.IsAny<Keyboard?>(), It.IsAny<string?>(),
                                                       It.IsAny<bool>(), It.IsAny<bool>()))
                  .ReturnsAsync("Bravo")
                  .Returns(Task.FromResult<string>(null!));

            await vm.RenameCommand.ExecuteAsync(alpha);

            dialog.Verify(d => d.ShowAlert("Error", It.Is<string>(m => m.Contains("already in use")), "OK"), Times.Once);
            service.Verify(s => s.RenameMazeItem(It.IsAny<MazeItem>(), It.IsAny<string>()), Times.Never);
        }

        // ---- Duplicate ------------------------------------------------------

        [Fact]
        public async Task DuplicateAsync_UserCancels_DoesNothing()
        {
            var (vm, dialog, service, _) = BuildVm();
            var item = MakeItem("1", "Alpha");
            vm.MazeItems.Add(item);
            dialog.Setup(d => d.DisplayPrompt(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(),
                                              It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string?>(),
                                              It.IsAny<int>(), It.IsAny<Keyboard?>(), It.IsAny<string?>(),
                                              It.IsAny<bool>(), It.IsAny<bool>()))
                  .Returns(Task.FromResult<string>(null!));

            await vm.DuplicateCommand.ExecuteAsync(item);

            service.Verify(s => s.CreateMazeItem(It.IsAny<MazeItem>()), Times.Never);
        }

        [Fact]
        public async Task DuplicateAsync_HappyPath_CallsCreateAndAddsToCollection()
        {
            var (vm, dialog, service, _) = BuildVm();
            var alpha = MakeItem("1", "Alpha");
            vm.MazeItems.Add(alpha);
            dialog.Setup(d => d.DisplayPrompt(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(),
                                              It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string?>(),
                                              It.IsAny<int>(), It.IsAny<Keyboard?>(), It.IsAny<string?>(),
                                              It.IsAny<bool>(), It.IsAny<bool>()))
                  .ReturnsAsync("Alpha 2");

            await vm.DuplicateCommand.ExecuteAsync(alpha);

            service.Verify(s => s.CreateMazeItem(It.Is<MazeItem>(i => i.Name == "Alpha 2")), Times.Once);
            Assert.Equal("Alpha,Alpha 2", string.Join(",", vm.MazeItems.Select(i => i.Name)));
        }

        [Fact]
        public async Task DuplicateAsync_NameAlreadyInUse_ShowsAlertAndDoesNotCallService()
        {
            var (vm, dialog, service, _) = BuildVm();
            var alpha = MakeItem("1", "Alpha");
            var copyOfAlpha = MakeItem("2", "Copy of Alpha");
            vm.MazeItems.Add(alpha);
            vm.MazeItems.Add(copyOfAlpha);
            // First default-fill returns the suggested "Copy of Alpha" (taken),
            // second prompt returns null to cancel and break the retry loop.
            dialog.SetupSequence(d => d.DisplayPrompt(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(),
                                                       It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string?>(),
                                                       It.IsAny<int>(), It.IsAny<Keyboard?>(), It.IsAny<string?>(),
                                                       It.IsAny<bool>(), It.IsAny<bool>()))
                  .ReturnsAsync("Copy of Alpha")
                  .Returns(Task.FromResult<string>(null!));

            await vm.DuplicateCommand.ExecuteAsync(alpha);

            dialog.Verify(d => d.ShowAlert("Error", It.Is<string>(m => m.Contains("already in use")), "OK"), Times.Once);
            service.Verify(s => s.CreateMazeItem(It.IsAny<MazeItem>()), Times.Never);
        }

        // ---- Delete ---------------------------------------------------------

        [Fact]
        public async Task DeleteAsync_UserCancels_DoesNothing()
        {
            var (vm, dialog, service, _) = BuildVm();
            var item = MakeItem("1", "Alpha");
            vm.MazeItems.Add(item);
            dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(),
                                                  It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(false);

            await vm.DeleteCommand.ExecuteAsync(item);

            service.Verify(s => s.DeleteMazeItem(It.IsAny<string>()), Times.Never);
            Assert.Single(vm.MazeItems);
        }

        [Fact]
        public async Task DeleteAsync_HappyPath_DeletesAndRemovesFromCollection()
        {
            var (vm, dialog, service, _) = BuildVm();
            var item = MakeItem("1", "Alpha");
            vm.MazeItems.Add(item);
            dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(),
                                                  It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(true);

            await vm.DeleteCommand.ExecuteAsync(item);

            service.Verify(s => s.DeleteMazeItem("1"), Times.Once);
            Assert.Empty(vm.MazeItems);
        }

        [Fact]
        public async Task DeleteAsync_ServiceFails_LeavesItemInCollectionAndAlerts()
        {
            var (vm, dialog, service, _) = BuildVm();
            var item = MakeItem("1", "Alpha");
            vm.MazeItems.Add(item);
            dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(),
                                                  It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(true);
            service.Setup(s => s.DeleteMazeItem(It.IsAny<string>())).ThrowsAsync(new HttpRequestException("forbidden"));

            await vm.DeleteCommand.ExecuteAsync(item);

            Assert.Single(vm.MazeItems);
            dialog.Verify(d => d.ShowAlert("Error", It.Is<string>(m => m.Contains("Failed to delete maze")), "OK"), Times.Once);
        }

        // ---- Message handlers -----------------------------------------------

        [Fact]
        public void Receive_MazesInvalidatedMessage_ClearsCollection()
        {
            var (vm, _, _, _) = BuildVm();
            vm.MazeItems.Add(MakeItem("1", "Alpha"));

            WeakReferenceMessenger.Default.Send(new MazesInvalidatedMessage());

            Assert.Empty(vm.MazeItems);
            Assert.False(vm.IsDataLoaded);
        }

        [Fact]
        public void Receive_NewMazeItemMessage_AddsAndSorts()
        {
            var (vm, _, _, _) = BuildVm();
            vm.MazeItems.Add(MakeItem("1", "Charlie"));

            WeakReferenceMessenger.Default.Send(new NewMazeItemMessage(MakeItem("2", "Alpha")));

            Assert.Equal("Alpha,Charlie", string.Join(",", vm.MazeItems.Select(i => i.Name)));
        }

        // ---- Navigation routing --------------------------------------------

        [Fact]
        public async Task GoToDesignAsync_RoutesViaNavigationServiceWithItem()
        {
            var (vm, _, _, nav) = BuildVm();
            var item = MakeItem("1", "Alpha");

            await vm.GoToDesignCommand.ExecuteAsync(item);

            nav.Verify(n => n.GoToAsync(
                nameof(MazePage),
                It.Is<IDictionary<string, object>>(d => ReferenceEquals(d["MazeItem"], item))),
                Times.Once);
        }

        [Fact]
        public async Task NewAsync_NavigatesToDesignWithFreshMazeItem()
        {
            var (vm, _, _, nav) = BuildVm();

            await vm.NewCommand.ExecuteAsync(null);

            nav.Verify(n => n.GoToAsync(nameof(MazePage), It.IsAny<IDictionary<string, object>>()), Times.Once);
        }

        [Fact]
        public async Task GoToPlayAsync_NoDefinition_ReturnsImmediately()
        {
            var (vm, _, _, nav) = BuildVm();
            // MazeItem with null Definition — the early-return guard fires.
            var item = MakeItem("1", "Alpha");

            await vm.GoToPlayCommand.ExecuteAsync(item);

            nav.VerifyNoOtherCalls();
        }
    }
}
