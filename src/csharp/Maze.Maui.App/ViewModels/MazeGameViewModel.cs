using CommunityToolkit.Mvvm.ComponentModel;
using Maze.Api;
using Maze.Maui.App.Models;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// View model for the maze game page. Receives a <see cref="MazeItem"/> via Shell navigation,
    /// then drives a <see cref="MazeGame"/> session when <see cref="StartGame"/> is called by the page.
    /// </summary>
    [QueryProperty("MazeItem", "MazeItem")]
    public partial class MazeGameViewModel : BaseViewModel
    {
        private MazeItem? _mazeItem;
        private MazeGame? _game;

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
        /// <param name="gameGrid">The <see cref="MazeGrid"/> to initialize.</param>
        public void StartGame(MazeGrid gameGrid)
        {
            LoadStatus = "";

            if (_mazeItem?.Definition is null)
            {
                LoadStatus = "Maze not available.";
                return;
            }

            _game?.Dispose();
            _game = null;

            gameGrid.Initialize(false, _mazeItem);

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
        /// Disposes the active game session. Called by the page when navigating away.
        /// </summary>
        public void Cleanup()
        {
            _game?.Dispose();
            _game = null;
        }
    }
}
