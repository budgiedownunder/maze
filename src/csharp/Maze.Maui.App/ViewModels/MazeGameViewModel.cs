using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Api;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Microsoft.Maui.Controls;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// View model for the maze game page. Receives a <see cref="MazeItem"/> via Shell navigation,
    /// then drives a <see cref="MazeGame"/> session when <see cref="StartGame"/> is called by the page.
    /// </summary>
    [QueryProperty("MazeItem", "MazeItem")]
    public partial class MazeGameViewModel : BaseViewModel
    {
        private readonly IDialogService _dialogService;
        private MazeItem? _mazeItem;
        private MazeGame? _game;
        private IMazeGridView? _gameGrid;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="dialogService">Injected dialog service</param>
        public MazeGameViewModel(IDialogService dialogService)
        {
            _dialogService = dialogService;
        }

        /// <summary>
        /// The maze item to play. Set via Shell navigation query property.
        /// </summary>
        public MazeItem? MazeItem
        {
            get => _mazeItem;
            set
            {
                _mazeItem = value;
                Title = value?.Name ?? "";
            }
        }

        /// <summary>
        /// True while the game result popup is being displayed.
        /// Used by the page to suppress lifecycle-driven cleanup during popup show/hide.
        /// </summary>
        public bool IsShowingResultPopup { get; private set; }

        /// <summary>
        /// Status message shown on error. Empty when no message.
        /// </summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(HasLoadStatus))]
        private string loadStatus = "";

        /// <summary>
        /// Whether a load status message is currently set.
        /// </summary>
        public bool HasLoadStatus => !string.IsNullOrEmpty(LoadStatus);

        /// <summary>
        /// Current bag contents (collected keys). Refreshed after each Pickup.
        /// </summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(BagCount))]
        [NotifyPropertyChangedFor(nameof(IsBagEmpty))]
        private IReadOnlyList<BagItem> bag = [];

        /// <summary>Number of items currently in the bag — convenience for bindings.</summary>
        public int BagCount => Bag.Count;

        /// <summary>Whether the bag is currently empty — convenience for bindings.</summary>
        public bool IsBagEmpty => Bag.Count == 0;

        /// <summary>
        /// Whether the game is in a lost state (e.g. player is stranded).
        /// </summary>
        [ObservableProperty]
        private bool isLost;

        /// <summary>
        /// The reason the game ended in a loss, or <see cref="LoseReason.None"/> when not lost.
        /// </summary>
        [ObservableProperty]
        private LoseReason loseReason = LoseReason.None;

        /// <summary>
        /// Whether the Pickup command should be enabled — true while the player
        /// is standing on an uncollected key cell and the game is still in play.
        /// </summary>
        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(PickupCommand))]
        private bool canPickup;

        /// <summary>
        /// Raised when a player move started a door unlocking. The page hooks
        /// this to start its tick-timer; the timer repeatedly calls
        /// <see cref="Tick(double)"/> until it returns <c>false</c>.
        /// </summary>
        public event Action? TickStartRequested;

        /// <summary>
        /// Test seam — overrides the default <see cref="MazeGame.Create"/>
        /// factory <see cref="StartGame"/> uses to spin up a game session.
        /// Production code never sets this; left at <c>null</c>,
        /// <see cref="StartGame"/> falls back to the real native interop
        /// path. Tests use this to inject a stub <see cref="MazeGame"/>.
        /// </summary>
        internal static Func<string, MazeGame>? GameFactory { get; set; }

        /// <summary>
        /// Initializes the grid with the maze definition, creates the game session,
        /// and places the player sprite at the start cell.
        /// Called by the page from <c>OnNavigatedTo</c> after the query property is set.
        /// </summary>
        /// <param name="gameGrid">The grid view (production: <see cref="MazeGrid"/>) to initialize.</param>
        public void StartGame(IMazeGridView gameGrid)
        {
            LoadStatus = "";
            _gameGrid = gameGrid;
            Bag = [];
            IsLost = false;
            LoseReason = LoseReason.None;
            CanPickup = false;

            if (_mazeItem?.Definition is null)
            {
                LoadStatus = "Maze not available.";
                return;
            }

            _game?.Dispose();
            _game = null;

            gameGrid.Initialize(false, _mazeItem);
            gameGrid.IsInteractionLocked = true;

            try
            {
                _game = (GameFactory ?? MazeGame.Create)(_mazeItem.Definition.DefinitionToJson());
                gameGrid.SetPlayerAt(_game.PlayerRow, _game.PlayerCol, _game.PlayerDirection);
                RefreshPickupAvailability();
            }
            catch (Exception ex)
            {
                LoadStatus = $"Unable to start game: {ex.Message}";
            }
        }

        /// <summary>
        /// Attempts to move the player in the given direction.
        /// </summary>
        /// <param name="direction">Direction to move</param>
        public async void Move(MazeGameDirection direction)
        {
            if (_game is null || _gameGrid is null || direction == MazeGameDirection.None
                || _game.IsComplete || _game.IsLost)
                return;

            int prevRow = _game.PlayerRow;
            int prevCol = _game.PlayerCol;

            var result = _game.MovePlayer(direction);

            if (result == MazeGameMoveResult.Moved
                || result == MazeGameMoveResult.Complete
                || result == MazeGameMoveResult.Stranded)
            {
                _gameGrid.SetVisitedDotAt(prevRow, prevCol);
                _gameGrid.SetPlayerAt(_game.PlayerRow, _game.PlayerCol, _game.PlayerDirection);
                RefreshPickupAvailability();
            }

            if (result == MazeGameMoveResult.StartedUnlocking)
            {
                // The door is at the cell the player tried to enter — not their new cell, which
                // is unchanged because the move was blocked-pending-unlock.
                (int doorRow, int doorCol) = NeighbourCell(prevRow, prevCol, direction);
                _gameGrid.SetDoorRuntimeState(doorRow, doorCol, DoorState.Opening);
                // The unlock attempt consumed a key from the bag — refresh so the UI shrinks.
                Bag = _game.Bag;
                TickStartRequested?.Invoke();
            }

            if (result == MazeGameMoveResult.Stranded)
            {
                IsLost = true;
                LoseReason = _game.LoseReason;
                await ShowResultPopup("You're stranded!!");
            }
            else if (result == MazeGameMoveResult.Complete)
            {
                _gameGrid.SetPlayerCelebrate(_game.PlayerRow, _game.PlayerCol);
                await ShowResultPopup("You win!");
            }
        }

        /// <summary>
        /// Picks up the collectible at the player's current cell. No-op when no
        /// collectible is present (<see cref="CanPickup"/> is then false).
        /// </summary>
        [RelayCommand(CanExecute = nameof(CanPickup))]
        public void Pickup()
        {
            if (_game is null || _gameGrid is null) return;
            BagItem? picked = _game.Pickup();
            if (picked is null) return;
            // The picked key was at the player's current cell.
            _gameGrid.MarkKeyCollected(_game.PlayerRow, _game.PlayerCol);
            Bag = _game.Bag;
            RefreshPickupAvailability();
        }

        /// <summary>
        /// Advances time-based game state by <paramref name="dtMs"/> milliseconds.
        /// Called by the page's tick timer while a door is opening. Returns
        /// <c>true</c> while any door is still <see cref="DoorState.Opening"/>,
        /// signalling the timer should fire again; <c>false</c> when the timer
        /// can stop.
        /// </summary>
        /// <param name="dtMs">Elapsed milliseconds since the previous tick.</param>
        /// <returns>Whether ticking should continue.</returns>
        public bool Tick(double dtMs)
        {
            if (_game is null || _gameGrid is null) return false;
            GameEvent[] events = _game.Tick(dtMs);
            foreach (var evt in events)
            {
                if (evt.Kind == GameEventKind.DoorOpened)
                {
                    _gameGrid.SetDoorRuntimeState((int)evt.Row, (int)evt.Column, DoorState.Open);
                }
            }
            return _game.Doors.Any(d => d.State == DoorState.Opening);
        }

        /// <summary>
        /// Disposes the active game session. Called by the page when navigating away.
        /// </summary>
        public void Cleanup()
        {
            _game?.Dispose();
            _game = null;
            _gameGrid?.IsInteractionLocked = false;
            _gameGrid = null;
            Bag = [];
            CanPickup = false;
        }

        private async Task ShowResultPopup(string message)
        {
            IsShowingResultPopup = true;
            try
            {
                await _dialogService.ShowGameResult(message);
            }
            finally
            {
                IsShowingResultPopup = false;
            }
        }

        private void RefreshPickupAvailability()
        {
            if (_game is null || _game.IsComplete || _game.IsLost)
            {
                CanPickup = false;
                return;
            }
            int r = _game.PlayerRow;
            int c = _game.PlayerCol;
            CanPickup = _game.Keys.Any(k => k.Row == (uint)r && k.Column == (uint)c);
        }

        private static (int row, int col) NeighbourCell(int row, int col, MazeGameDirection direction) => direction switch
        {
            MazeGameDirection.Up => (row - 1, col),
            MazeGameDirection.Down => (row + 1, col),
            MazeGameDirection.Left => (row, col - 1),
            MazeGameDirection.Right => (row, col + 1),
            _ => (row, col),
        };
    }
}
