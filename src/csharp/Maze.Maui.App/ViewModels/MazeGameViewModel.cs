using CommunityToolkit.Mvvm.ComponentModel;
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
        /// Initializes the grid with the maze definition, creates the game session,
        /// and places the player sprite at the start cell.
        /// Called by the page from <c>OnNavigatedTo</c> after the query property is set.
        /// </summary>
        /// <param name="gameGrid">The grid view (production: <see cref="MazeGrid"/>) to initialize.</param>
        public void StartGame(IMazeGridView gameGrid)
        {
            LoadStatus = "";
            _gameGrid = gameGrid;

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
                _game = MazeGame.Create(_mazeItem.Definition.DefinitionToJson());
                gameGrid.SetPlayerAt(_game.PlayerRow, _game.PlayerCol, _game.PlayerDirection);
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
            if (_game is null || _gameGrid is null || direction == MazeGameDirection.None || _game.IsComplete)
                return;

            int prevRow = _game.PlayerRow;
            int prevCol = _game.PlayerCol;

            var result = _game.MovePlayer(direction);

            if (result == MazeGameMoveResult.Moved || result == MazeGameMoveResult.Complete)
            {
                _gameGrid.SetVisitedDotAt(prevRow, prevCol);
                _gameGrid.SetPlayerAt(_game.PlayerRow, _game.PlayerCol, _game.PlayerDirection);
            }

            if (result == MazeGameMoveResult.Complete)
            {
                _gameGrid.SetPlayerCelebrate(_game.PlayerRow, _game.PlayerCol);
                IsShowingResultPopup = true;
                try
                {
                    await _dialogService.ShowGameResult("You win!");
                }
                finally
                {
                    IsShowingResultPopup = false;
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
        }
    }
}
