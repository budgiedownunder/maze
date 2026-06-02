using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Api;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Microsoft.Maui.Controls;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>One heart in the HP HUD. <see cref="Opacity"/> dims an empty heart.</summary>
    /// <param name="Filled">Whether this heart represents remaining HP.</param>
    public readonly record struct HeartState(bool Filled)
    {
        /// <summary>Render opacity — full for a filled heart, dimmed for an empty one.</summary>
        public double Opacity => Filled ? 1.0 : 0.25;
    }

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
        // Tracks each enemy's last-known cell by id so EnemyMoved events (which carry
        // only the new cell) can tell the grid which cell to vacate.
        private readonly Dictionary<uint, (int row, int col)> _enemyCells = new();

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
        /// True while the pause popup is being displayed.
        /// Used by the page to suppress lifecycle-driven cleanup during popup show/hide.
        /// </summary>
        public bool IsShowingPausePopup { get; private set; }

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

        /// <summary>The player's current HP. Drives the heart-row HUD.</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(Hearts))]
        private uint hp;

        /// <summary>The player's maximum HP — the number of hearts in the HUD.</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(Hearts))]
        private uint maxHp;

        /// <summary>
        /// One heart per <see cref="MaxHp"/>, full or dimmed according to <see cref="Hp"/>.
        /// Drives the heart-row HUD via a bindable layout.
        /// </summary>
        public IReadOnlyList<HeartState> Hearts =>
            Enumerable.Range(0, (int)MaxHp).Select(i => new HeartState(i < Hp)).ToList();

        /// <summary>
        /// Raised when the player takes damage. The page hooks this to flash the
        /// damage overlay.
        /// </summary>
        public event Action? DamageFlashRequested;

        /// <summary>
        /// Raised when a player move started a door unlocking. The page hooks
        /// this to start its tick-timer; the timer repeatedly calls
        /// <see cref="Tick(double)"/> until it returns <c>false</c>.
        /// </summary>
        public event Action? TickStartRequested;

        /// <summary>
        /// Whether the game is paused. While paused the tick loop is stopped and
        /// moves are ignored; the pause popup is shown.
        /// </summary>
        [ObservableProperty]
        private bool isPaused;

        /// <summary>
        /// Raised when the game is paused. The page hooks this to stop its
        /// tick timer (resume re-arms it via <see cref="TickStartRequested"/>).
        /// </summary>
        public event Action? PauseRequested;

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
            IsPaused = false;
            LoseReason = LoseReason.None;
            _enemyCells.Clear();

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
                Hp = _game.Hp;
                MaxHp = _game.MaxHp;
                gameGrid.BeginGameRuntime();
                foreach (var enemy in _game.Enemies)
                    _enemyCells[enemy.Id] = ((int)enemy.Row, (int)enemy.Column);
                gameGrid.SetPlayerAt(_game.PlayerRow, _game.PlayerCol, _game.PlayerDirection);
                // Enemies move on a fixed cadence — run the tick loop continuously
                // while any exist (the page's timer keeps firing while Tick() returns true).
                if (_game.Enemies.Count > 0)
                    TickStartRequested?.Invoke();
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
                || _game.IsComplete || _game.IsLost || IsPaused)
                return;

            int prevRow = _game.PlayerRow;
            int prevCol = _game.PlayerCol;

            var result = _game.MovePlayer(direction);

            if (result == MazeGameMoveResult.Moved
                || result == MazeGameMoveResult.Complete
                || result == MazeGameMoveResult.Stranded
                || result == MazeGameMoveResult.Killed)
            {
                _gameGrid.SetVisitedDotAt(prevRow, prevCol);
                _gameGrid.SetPlayerAt(_game.PlayerRow, _game.PlayerCol, _game.PlayerDirection);
                // Flush events the move queued synchronously: KeyCollected from
                // walking onto a key, PlayerDamaged from stepping into an enemy,
                // PlayerHealed / PlayerNotHealed from stepping onto a health
                // pickup. (No enemy advances on a 0ms tick.)
                ProcessTickEvents(_game.Tick(0));
                Hp = _game.Hp;
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

            if (result == MazeGameMoveResult.Killed)
            {
                IsLost = true;
                LoseReason = _game.LoseReason;
                await ShowResultPopup("You died!", won: false);
            }
            else if (result == MazeGameMoveResult.Stranded)
            {
                IsLost = true;
                LoseReason = _game.LoseReason;
                await ShowResultPopup("You're stranded!!", won: false);
            }
            else if (result == MazeGameMoveResult.Complete)
            {
                _gameGrid.SetPlayerCelebrate(_game.PlayerRow, _game.PlayerCol);
                await ShowResultPopup("You win!", won: true);
            }
        }

        /// <summary>
        /// Pauses the game and shows the pause menu (Resume / Restart). Triggered
        /// by the centre D-pad button or the Space / Esc keys. No-op once the
        /// game is won/lost or already paused.
        /// </summary>
        [RelayCommand]
        public async Task Pause()
        {
            if (_game is null || _gameGrid is null || IsPaused || _game.IsComplete || _game.IsLost)
                return;
            // Capture the grid now: showing the popup drives the page's
            // navigation lifecycle, and the IsShowingPausePopup guard keeps it
            // from running cleanup — but a local keeps Restart safe regardless.
            IMazeGridView grid = _gameGrid;
            IsPaused = true;
            PauseRequested?.Invoke(); // page stops the tick timer
            PauseMenuResult action;
            IsShowingPausePopup = true;
            try
            {
                action = await _dialogService.ShowPauseMenu();
            }
            finally
            {
                IsShowingPausePopup = false;
            }
            if (action == PauseMenuResult.Restart)
            {
                // Full reset reuses the same grid view; it clears IsPaused and
                // re-arms the tick loop.
                StartGame(grid);
            }
            else
            {
                IsPaused = false;
                // Re-arm the tick loop. OnTickStartRequested reseeds the dt
                // baseline to now, so the paused span isn't counted as elapsed
                // time (otherwise enemies lurch forward on resume).
                TickStartRequested?.Invoke();
            }
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
            if (_game.IsComplete || _game.IsLost) return false;
            ProcessTickEvents(_game.Tick(dtMs));
            Hp = _game.Hp;
            // An enemy stepping onto the player drops HP with no player move; if that
            // is fatal, surface the death here (a subsequent player Move would
            // otherwise be the first to report it).
            if (_game.IsLost && !IsLost)
            {
                IsLost = true;
                LoseReason = _game.LoseReason;
                _ = ShowResultPopup("You died!", won: false);
                return false;
            }
            // Keep ticking while enemies exist (fixed-cadence movement) or a door is
            // still opening; stop otherwise.
            return _game.Enemies.Count > 0 || _game.Doors.Any(d => d.State == DoorState.Opening);
        }

        /// <summary>
        /// Dispatches a batch of tick events to the grid view and HUD: door opens,
        /// enemy moves, damage flashes, and consumed health pickups.
        /// </summary>
        private void ProcessTickEvents(GameEvent[] events)
        {
            if (_gameGrid is null || _game is null) return;
            foreach (var evt in events)
            {
                switch (evt.Kind)
                {
                    case GameEventKind.DoorOpened:
                        _gameGrid.SetDoorRuntimeState((int)evt.Row, (int)evt.Column, DoorState.Open);
                        break;
                    case GameEventKind.EnemyMoved:
                        uint id = evt.Payload;
                        (int oldRow, int oldCol) = _enemyCells.TryGetValue(id, out var pos) ? pos : (-1, -1);
                        _gameGrid.SetEnemyCell(oldRow, oldCol, (int)evt.Row, (int)evt.Column, id);
                        _enemyCells[id] = ((int)evt.Row, (int)evt.Column);
                        break;
                    case GameEventKind.PlayerDamaged:
                        DamageFlashRequested?.Invoke();
                        break;
                    case GameEventKind.PlayerHealed:
                        _gameGrid.MarkHealthCollected((int)evt.Row, (int)evt.Column);
                        break;
                    case GameEventKind.PlayerNotHealed:
                        // Pickup spared (player already at max HP) — nothing to render.
                        break;
                    case GameEventKind.KeyCollected:
                        // A key was auto-collected on walk-over: clear its grid
                        // visual and refresh the bag so the new key shows.
                        _gameGrid.MarkKeyCollected((int)evt.Row, (int)evt.Column);
                        Bag = _game.Bag;
                        break;
                }
            }
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
        }

        private async Task ShowResultPopup(string message, bool won)
        {
            IsShowingResultPopup = true;
            bool playAgain;
            try
            {
                playAgain = await _dialogService.ShowGameResult(message, won);
            }
            finally
            {
                IsShowingResultPopup = false;
            }
            // Play Again restarts the current maze from the beginning, reusing the
            // same grid view the session was started with.
            if (playAgain && _gameGrid is not null)
                StartGame(_gameGrid);
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
