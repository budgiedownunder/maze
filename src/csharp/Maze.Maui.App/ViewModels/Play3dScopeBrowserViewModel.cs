using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// A tabbed Play 3D browse surface for one ownership scope (My Games =
    /// <see cref="GameListScope.Mine"/>, Shared with me = <see cref="GameListScope.Shared"/>,
    /// Community = <see cref="GameListScope.Public"/>). Games and collections are two
    /// separate name-ordered, independently-paged lists behind a Games / Collections
    /// tab strip, so neither needs a cross-entity merge — each is a
    /// <see cref="Play3dListViewModel"/> with its own search box. This model only owns
    /// the tab selection and the shared sort (offered for the unbounded Community scope).
    /// </summary>
    public partial class Play3dScopeBrowserViewModel : BaseViewModel
    {
        private readonly GameListScope _scope;
        private bool _collectionsLoaded;

        /// <summary>The Games tab's list.</summary>
        public Play3dDefinitionsViewModel Games { get; }

        /// <summary>The Collections tab's list.</summary>
        public Play3dCollectionsViewModel Collections { get; }

        /// <summary>Whether the Games tab is showing (else the Collections tab).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(ShowingCollections))]
        private bool showingGames = true;

        /// <summary>Whether the Collections tab is showing.</summary>
        public bool ShowingCollections => !ShowingGames;

        /// <summary>Whether this scope offers a sort control (Community only).</summary>
        public bool CanSort => _scope == GameListScope.Public;

        /// <summary>Selected sort (Community only): 0 = A–Z by name, 1 = newest first.</summary>
        [ObservableProperty]
        private int sortIndex;

        /// <summary>Constructor</summary>
        /// <param name="gameLibrary">Injected game-library read service</param>
        /// <param name="navigationService">Injected navigation service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="scope">The ownership scope this page browses</param>
        /// <param name="title">The page title</param>
        public Play3dScopeBrowserViewModel(
            IGameLibraryService gameLibrary, INavigationService navigationService, IDialogService dialogService,
            GameListScope scope, string title)
        {
            _scope = scope;
            Title = title;
            Games = new Play3dDefinitionsViewModel(gameLibrary, navigationService, dialogService, scope);
            Collections = new Play3dCollectionsViewModel(gameLibrary, navigationService, dialogService, scope);
        }

        // Null unless this scope sorts (Community); Newest vs Name from the picker.
        private GameListSort? Sort => CanSort ? (SortIndex == 1 ? GameListSort.Newest : GameListSort.Name) : null;

        /// <summary>Loads the initial (Games) tab; the Collections tab loads lazily on first switch.</summary>
        /// <returns>Task</returns>
        public Task LoadAsync()
        {
            Games.SetSort(Sort);
            return Games.LoadFirstPageAsync();
        }

        [RelayCommand]
        private void ShowGames() => ShowingGames = true;

        [RelayCommand]
        private async Task ShowCollectionsAsync()
        {
            ShowingGames = false;
            if (!_collectionsLoaded)
            {
                _collectionsLoaded = true;
                Collections.SetSort(Sort);
                await Collections.LoadFirstPageAsync();
            }
        }

        partial void OnSortIndexChanged(int value) => _ = ApplySortAsync();

        // Apply the chosen sort to both loaded lists (server-ordered, Community only).
        private async Task ApplySortAsync()
        {
            Games.SetSort(Sort);
            await Games.LoadFirstPageAsync();
            if (_collectionsLoaded)
            {
                Collections.SetSort(Sort);
                await Collections.LoadFirstPageAsync();
            }
        }
    }
}
