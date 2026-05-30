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

        // ---- Pickup / Bag / Tick / Stranded flow (game session injected) ---
        //
        // Tests in this section drive a stub MazeGame injected via the
        // MazeGameViewModel.GameFactory test seam. The stub is the one in
        // Stubs/MazeGame.cs — settable properties + one-shot NextX hooks.
        // Each test sets up MazeItem + a stub MazeGame, calls StartGame to
        // install the session, then exercises the view-model surface.

        private static MazeItem ItemWithDefinition() =>
            new MazeItem { Name = "Test", Definition = new Maze.Api.Maze(3, 3) };

        private static MazeGame InstallGame(MazeGameViewModel vm, IMazeGridView grid, MazeItem item)
        {
            vm.MazeItem = item;
            MazeGame stub = MazeGame.CreateForTests();
            MazeGameViewModel.GameFactory = _ => stub;
            try
            {
                vm.StartGame(grid);
            }
            finally
            {
                MazeGameViewModel.GameFactory = null;
            }
            return stub;
        }

        [Fact]
        public void Pickup_OnUncollectedKey_AddsToBagAndMarksGridCollected()
        {
            var (vm, _, grid) = BuildVm();
            var item = ItemWithDefinition();
            MazeGame stub = InstallGame(vm, grid.Object, item);

            // Place the player on a key cell (0,1) with a single uncollected key.
            stub.PlayerRow = 0;
            stub.PlayerCol = 1;
            stub.Keys = new List<KeyInfo> { new KeyInfo(0, 1, 42) };
            stub.NextPickupItem = new BagItem(BagItemKind.Key, 42);

            // PickupCommand.Execute bypasses CanExecute, which is fine for a unit
            // test — we're verifying the orchestration that runs once Pickup() is
            // invoked, independent of how the page enables the button.
            vm.PickupCommand.Execute(null);

            Assert.Single(vm.Bag);
            Assert.Equal(new BagItem(BagItemKind.Key, 42), vm.Bag[0]);
            grid.Verify(g => g.MarkKeyCollected(0, 1), Times.Once);
        }

        [Fact]
        public void Move_ResultStartedUnlocking_RaisesTickStartRequestedAndMarksDoorOpening()
        {
            var (vm, _, grid) = BuildVm();
            var item = ItemWithDefinition();
            MazeGame stub = InstallGame(vm, grid.Object, item);
            stub.PlayerRow = 0;
            stub.PlayerCol = 1;
            stub.NextMoveResult = MazeGameMoveResult.StartedUnlocking;
            bool tickRequested = false;
            vm.TickStartRequested += () => tickRequested = true;

            vm.Move(MazeGameDirection.Right);

            Assert.True(tickRequested);
            // Door is one cell to the right of the player (0,1) → (0,2).
            grid.Verify(g => g.SetDoorRuntimeState(0, 2, DoorState.Opening), Times.Once);
        }

        [Fact]
        public void Tick_EmitsDoorOpenedEvent_FlipsDoorToOpenOnGrid_ReturnsFalse_WhenNoneOpening()
        {
            var (vm, _, grid) = BuildVm();
            var item = ItemWithDefinition();
            MazeGame stub = InstallGame(vm, grid.Object, item);
            stub.Doors = new List<DoorInfo> { new DoorInfo(0, 2, DoorState.Opening) };
            stub.NextTickEvents = new[] { new GameEvent(GameEventKind.DoorOpened, 0, 2, 0) };

            bool keepTicking = vm.Tick(1000.0);

            grid.Verify(g => g.SetDoorRuntimeState(0, 2, DoorState.Open), Times.Once);
            Assert.False(keepTicking); // door went Open → no more Opening doors
        }

        [Fact]
        public void Move_ResultStranded_FlipsIsLostAndShowsStrandedPopup()
        {
            var (vm, dialog, grid) = BuildVm();
            var item = ItemWithDefinition();
            MazeGame stub = InstallGame(vm, grid.Object, item);
            // Move's pre-guard reads stub.IsLost — leave it false here; production
            // flips vm.IsLost on the Stranded result, reading LoseReason from the stub.
            stub.NextMoveResult = MazeGameMoveResult.Stranded;
            stub.LoseReason = LoseReason.Stranded;

            vm.Move(MazeGameDirection.Down);

            Assert.True(vm.IsLost);
            Assert.Equal(LoseReason.Stranded, vm.LoseReason);
            dialog.Verify(d => d.ShowGameResult("You're stranded!!"), Times.Once);
        }

        [Fact]
        public void Move_AfterIsLost_DoesNothing()
        {
            var (vm, dialog, grid) = BuildVm();
            var item = ItemWithDefinition();
            MazeGame stub = InstallGame(vm, grid.Object, item);
            // Pre-existing lost state on the underlying game session is what the
            // guard checks — verifies subsequent moves are no-ops.
            stub.IsLost = true;
            stub.LoseReason = LoseReason.Stranded;

            vm.Move(MazeGameDirection.Right);

            grid.Verify(g => g.SetVisitedDotAt(It.IsAny<int>(), It.IsAny<int>()), Times.Never);
            dialog.Verify(d => d.ShowGameResult(It.IsAny<string>()), Times.Never);
        }

        // ---- Enemies / HP -------------------------------------------------

        // Installs a game with the stub pre-configured (HP / enemies) BEFORE
        // StartGame so the view-model seeds from them.
        private static MazeGame InstallGamePreconfigured(MazeGameViewModel vm, IMazeGridView grid, MazeItem item, Action<MazeGame> configure)
        {
            vm.MazeItem = item;
            MazeGame stub = MazeGame.CreateForTests();
            configure(stub);
            MazeGameViewModel.GameFactory = _ => stub;
            try { vm.StartGame(grid); }
            finally { MazeGameViewModel.GameFactory = null; }
            return stub;
        }

        [Fact]
        public void StartGame_SeedsHpAndMaxHp_AndBeginsGameRuntime()
        {
            var (vm, _, grid) = BuildVm();
            MazeGame stub = InstallGamePreconfigured(vm, grid.Object, ItemWithDefinition(), s =>
            {
                s.Hp = 3;
                s.MaxHp = 3;
                s.Enemies = new List<EnemyInfo> { new EnemyInfo(0, 1, 0) };
            });
            _ = stub;

            Assert.Equal(3u, vm.Hp);
            Assert.Equal(3u, vm.MaxHp);
            Assert.Equal(3, vm.Hearts.Count);
            Assert.All(vm.Hearts, h => Assert.True(h.Filled));
            grid.Verify(g => g.BeginGameRuntime(), Times.Once);
        }

        [Fact]
        public void Move_IntoEnemy_DecrementsHpAndFlashesDamage()
        {
            var (vm, _, grid) = BuildVm();
            MazeGame stub = InstallGamePreconfigured(vm, grid.Object, ItemWithDefinition(), s => { s.Hp = 3; s.MaxHp = 3; });
            stub.NextMoveResult = MazeGameMoveResult.Moved;
            // The move queues a PlayerDamaged event (HP after = 2) flushed by the 0ms tick.
            stub.NextTickEvents = new[] { new GameEvent(GameEventKind.PlayerDamaged, 0, 0, 2) };
            stub.Hp = 2;
            bool flashed = false;
            vm.DamageFlashRequested += () => flashed = true;

            vm.Move(MazeGameDirection.Right);

            Assert.Equal(2u, vm.Hp);
            Assert.True(flashed);
        }

        [Fact]
        public void Move_OntoHealthPickup_IncrementsHpAndMarksPickupCollected()
        {
            var (vm, _, grid) = BuildVm();
            MazeGame stub = InstallGamePreconfigured(vm, grid.Object, ItemWithDefinition(), s => { s.Hp = 2; s.MaxHp = 3; });
            stub.NextMoveResult = MazeGameMoveResult.Moved;
            // The move queues a PlayerHealed event at the consumed cell (0,2), HP after = 3.
            stub.NextTickEvents = new[] { new GameEvent(GameEventKind.PlayerHealed, 0, 2, 3) };
            stub.Hp = 3;

            vm.Move(MazeGameDirection.Right);

            Assert.Equal(3u, vm.Hp);
            grid.Verify(g => g.MarkHealthCollected(0, 2), Times.Once);
        }

        [Fact]
        public void Tick_EnemyMovedEvent_NotifiesGridOfNewEnemyCell()
        {
            var (vm, _, grid) = BuildVm();
            MazeGame stub = InstallGamePreconfigured(vm, grid.Object, ItemWithDefinition(), s =>
            {
                s.Hp = 3;
                s.MaxHp = 3;
                s.Enemies = new List<EnemyInfo> { new EnemyInfo(0, 1, 0) };
            });
            // Enemy id 0 advances from its spawn (0,1) to (0,0).
            stub.NextTickEvents = new[] { new GameEvent(GameEventKind.EnemyMoved, 0, 0, 0) };

            bool keepTicking = vm.Tick(1500.0);

            grid.Verify(g => g.SetEnemyCell(0, 1, 0, 0, 0), Times.Once);
            Assert.True(keepTicking); // an enemy still exists → keep ticking
        }

        [Fact]
        public void Move_ResultKilled_FlipsIsLostAndShowsDeathPopup()
        {
            var (vm, dialog, grid) = BuildVm();
            MazeGame stub = InstallGamePreconfigured(vm, grid.Object, ItemWithDefinition(), s => { s.Hp = 1; s.MaxHp = 3; });
            stub.NextMoveResult = MazeGameMoveResult.Killed;
            stub.LoseReason = LoseReason.Killed;
            stub.Hp = 0;

            vm.Move(MazeGameDirection.Right);

            Assert.True(vm.IsLost);
            Assert.Equal(LoseReason.Killed, vm.LoseReason);
            dialog.Verify(d => d.ShowGameResult("You died!"), Times.Once);
        }
    }
}
