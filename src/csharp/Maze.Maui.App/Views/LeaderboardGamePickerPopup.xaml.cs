using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// The Leaderboards game picker — a scope-tabbed (Featured / My Games / Shared /
    /// Community), searchable, paged browser of stored games and expandable
    /// collections. Tapping a game (or a collection member) closes with that
    /// <see cref="GameDefinition"/>; tapping a collection expands/collapses its
    /// members; Cancel returns <c>null</c>. The caller keys the board on the chosen
    /// game's <c>def:&lt;id&gt;</c>.
    /// </summary>
    public partial class LeaderboardGamePickerPopup : Popup
    {
        private readonly LeaderboardGamePickerViewModel _viewModel;

        /// <summary>Constructor</summary>
        /// <param name="gameLibrary">Injected game-library read service</param>
        public LeaderboardGamePickerPopup(IGameLibraryService gameLibrary)
        {
            InitializeComponent();
            _viewModel = new LeaderboardGamePickerViewModel(gameLibrary);
            BindingContext = _viewModel;
            _ = _viewModel.LoadAsync();
        }

        // Tapping a game (or member) selects it; tapping a collection expands it.
        private async void OnRowTapped(object sender, TappedEventArgs e)
        {
            if (sender is not Element element || element.BindingContext is not GamePickerRow row)
                return;

            if (row.IsCollection)
            {
                await _viewModel.ToggleCollectionAsync(row);
                return;
            }

            await Navigation.ClosePopupAsync<GameDefinition?>(row.Game);
        }

        private async void OnCancelClicked(object sender, EventArgs e)
            => await Navigation.ClosePopupAsync<GameDefinition?>(null);
    }
}
