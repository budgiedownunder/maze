using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>Which slice of stored games the picker is browsing.</summary>
    public enum GamePickerScope
    {
        /// <summary>The admin-ordered featured catalogue (mixed games + collections).</summary>
        Featured,
        /// <summary>The caller's own games/collections.</summary>
        Mine,
        /// <summary>Games/collections shared with the caller.</summary>
        Shared,
        /// <summary>The public (Community) pool.</summary>
        Public,
    }

    /// <summary>
    /// A row in the game picker: a selectable game (optionally an indented collection
    /// member), or an expandable collection.
    /// </summary>
    public partial class GamePickerRow : ObservableObject
    {
        /// <summary>True when this row is an expandable collection (else a game).</summary>
        public bool IsCollection { get; init; }

        /// <summary>True when this row is a game shown indented inside its collection.</summary>
        public bool IsMember { get; init; }

        /// <summary>The game (game / member rows), or <c>null</c> for a collection row.</summary>
        public GameDefinition? Game { get; init; }

        /// <summary>The collection (collection rows), or <c>null</c> for a game row.</summary>
        public GameCollection? Collection { get; init; }

        /// <summary>Whether an expandable collection row is expanded.</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(Glyph))]
        private bool isExpanded;

        /// <summary>Display name.</summary>
        public string Name => IsCollection ? Collection!.Name : Game!.Name;

        /// <summary>The leading glyph: a disclosure caret for a collection, a bullet for a member game.</summary>
        public string Glyph => IsCollection ? (IsExpanded ? "▾" : "▸") : IsMember ? "•" : "";

        /// <summary>Left indent for member rows.</summary>
        public double LeftIndent => IsMember ? 28 : 12;
    }

    /// <summary>
    /// Drives the Leaderboards game picker popup: a scope-tabbed (Featured / My Games
    /// / Shared / Community), paged browser of stored games and expandable
    /// collections, with server-side search for the scopes that support it. The
    /// featured scope renders the single mixed catalogue list; the other scopes show
    /// their collections (each expandable to its member games) and their games as two
    /// independently-paged lists. Selecting a game is handled by the popup.
    /// </summary>
    public partial class LeaderboardGamePickerViewModel : BaseViewModel
    {
        private const int PageSize = 20;
        private const int SearchDebounceMs = 300;

        private readonly IGameLibraryService _gameLibrary;
        private CancellationTokenSource? _searchCts;

        /// <summary>The featured catalogue rows (Featured scope).</summary>
        public ObservableCollection<GamePickerRow> FeaturedRows { get; } = new();

        /// <summary>The collection rows (+ expanded members) for a non-featured scope.</summary>
        public ObservableCollection<GamePickerRow> CollectionRows { get; } = new();

        /// <summary>The game rows for a non-featured scope.</summary>
        public ObservableCollection<GamePickerRow> GameRows { get; } = new();

        /// <summary>The browsed scope.</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsFeatured))]
        [NotifyPropertyChangedFor(nameof(IsNotFeatured))]
        [NotifyPropertyChangedFor(nameof(IsMineTab))]
        [NotifyPropertyChangedFor(nameof(IsSharedTab))]
        [NotifyPropertyChangedFor(nameof(IsPublicTab))]
        [NotifyPropertyChangedFor(nameof(CanSearch))]
        private GamePickerScope scope = GamePickerScope.Featured;

        /// <summary>Search text (server-side, honoured for My Games / Community).</summary>
        [ObservableProperty]
        private string searchText = "";

        /// <summary>Whether a further featured page exists.</summary>
        [ObservableProperty]
        private bool featuredHasMore;

        /// <summary>Whether a further collections page exists.</summary>
        [ObservableProperty]
        private bool collectionsHasMore;

        /// <summary>Whether a further games page exists.</summary>
        [ObservableProperty]
        private bool gamesHasMore;

        /// <summary>Empty/error status text.</summary>
        [ObservableProperty]
        private string statusMessage = "";

        /// <summary>Whether the status text is shown.</summary>
        [ObservableProperty]
        private bool showStatusMessage;

        /// <summary>True for the Featured scope (single mixed list).</summary>
        public bool IsFeatured => Scope == GamePickerScope.Featured;

        /// <summary>True for a non-featured scope (Collections + Games sections).</summary>
        public bool IsNotFeatured => !IsFeatured;

        /// <summary>Whether the My Games tab is active.</summary>
        public bool IsMineTab => Scope == GamePickerScope.Mine;

        /// <summary>Whether the Shared tab is active.</summary>
        public bool IsSharedTab => Scope == GamePickerScope.Shared;

        /// <summary>Whether the Community tab is active.</summary>
        public bool IsPublicTab => Scope == GamePickerScope.Public;

        /// <summary>Whether this scope offers server-side search (My Games / Community).</summary>
        public bool CanSearch => Scope is GamePickerScope.Mine or GamePickerScope.Public;

        /// <summary>Constructor</summary>
        /// <param name="gameLibrary">Injected game-library read service</param>
        public LeaderboardGamePickerViewModel(IGameLibraryService gameLibrary)
        {
            _gameLibrary = gameLibrary;
        }

        /// <summary>Loads the initial (Featured) scope. Called by the popup on open.</summary>
        /// <returns>Task</returns>
        public Task LoadAsync() => ReloadAsync();

        [RelayCommand]
        private Task SelectFeatured() => SetScopeAsync(GamePickerScope.Featured);

        [RelayCommand]
        private Task SelectMine() => SetScopeAsync(GamePickerScope.Mine);

        [RelayCommand]
        private Task SelectShared() => SetScopeAsync(GamePickerScope.Shared);

        [RelayCommand]
        private Task SelectPublic() => SetScopeAsync(GamePickerScope.Public);

        [RelayCommand]
        private async Task LoadMoreFeatured()
        {
            FeaturedGameItemsListResponse resp = await _gameLibrary.GetFeaturedGameItemsAsync(PageSize, FeaturedRows.Count);
            foreach (FeaturedGameItem item in resp.Items)
                AddFeaturedItem(item);
            FeaturedHasMore = resp.HasMore;
        }

        [RelayCommand]
        private async Task LoadMoreCollections()
        {
            int offset = CollectionRows.Count(r => r.IsCollection);
            GameCollectionListResponse resp = await _gameLibrary.ListGameCollectionsAsync(ListScope, Query, null, PageSize, offset);
            foreach (GameCollection collection in resp.Collections)
                CollectionRows.Add(new GamePickerRow { IsCollection = true, Collection = collection });
            CollectionsHasMore = resp.HasMore;
        }

        [RelayCommand]
        private async Task LoadMoreGames()
        {
            GameDefinitionListResponse resp = await _gameLibrary.ListGameDefinitionsAsync(ListScope, Query, null, PageSize, GameRows.Count);
            foreach (GameDefinition game in resp.Definitions)
                GameRows.Add(new GamePickerRow { Game = game });
            GamesHasMore = resp.HasMore;
        }

        /// <summary>Expands or collapses a collection row, loading its members on first expand.</summary>
        /// <param name="row">The collection row</param>
        /// <returns>Task</returns>
        public async Task ToggleCollectionAsync(GamePickerRow row)
        {
            ObservableCollection<GamePickerRow> list = IsFeatured ? FeaturedRows : CollectionRows;
            int index = list.IndexOf(row);
            if (index < 0)
                return;

            if (row.IsExpanded)
            {
                // Collapse: remove the member rows that follow.
                while (index + 1 < list.Count && list[index + 1].IsMember)
                    list.RemoveAt(index + 1);
                row.IsExpanded = false;
                return;
            }

            try
            {
                GameCollectionDetailResponse detail = await _gameLibrary.GetGameCollectionAsync(row.Collection!.Id);
                int insertAt = index + 1;
                foreach (GameDefinition member in detail.Definitions)
                    list.Insert(insertAt++, new GamePickerRow { Game = member, IsMember = true });
                row.IsExpanded = true;
            }
            catch (Exception ex)
            {
                SetStatus(ex.Message);
            }
        }

        partial void OnSearchTextChanged(string value)
        {
            if (CanSearch)
                DebounceReload();
        }

        private void DebounceReload()
        {
            _searchCts?.Cancel();
            var cts = new CancellationTokenSource();
            _searchCts = cts;
            _ = DebounceAsync(cts.Token);
        }

        private async Task DebounceAsync(CancellationToken token)
        {
            try
            {
                await Task.Delay(SearchDebounceMs, token);
            }
            catch (TaskCanceledException)
            {
                return;
            }
            await ReloadAsync();
        }

        private async Task SetScopeAsync(GamePickerScope scope)
        {
            if (Scope == scope)
                return;
            Scope = scope;
            SearchText = "";
            await ReloadAsync();
        }

        // The server list scope for the current non-featured tab.
        private GameListScope ListScope => Scope switch
        {
            GamePickerScope.Mine => GameListScope.Mine,
            GamePickerScope.Shared => GameListScope.Shared,
            _ => GameListScope.Public,
        };

        // The server query (only the searchable scopes send one).
        private string? Query => CanSearch && !string.IsNullOrWhiteSpace(SearchText) ? SearchText.Trim() : null;

        private async Task ReloadAsync()
        {
            IsBusy = true;
            SetStatus("");
            FeaturedRows.Clear();
            CollectionRows.Clear();
            GameRows.Clear();
            FeaturedHasMore = false;
            CollectionsHasMore = false;
            GamesHasMore = false;
            try
            {
                if (IsFeatured)
                {
                    FeaturedGameItemsListResponse resp = await _gameLibrary.GetFeaturedGameItemsAsync(PageSize, 0);
                    foreach (FeaturedGameItem item in resp.Items)
                        AddFeaturedItem(item);
                    FeaturedHasMore = resp.HasMore;
                    if (FeaturedRows.Count == 0)
                        SetStatus("Nothing featured yet.");
                }
                else
                {
                    GameCollectionListResponse collections = await _gameLibrary.ListGameCollectionsAsync(ListScope, Query, null, PageSize, 0);
                    foreach (GameCollection collection in collections.Collections)
                        CollectionRows.Add(new GamePickerRow { IsCollection = true, Collection = collection });
                    CollectionsHasMore = collections.HasMore;

                    GameDefinitionListResponse games = await _gameLibrary.ListGameDefinitionsAsync(ListScope, Query, null, PageSize, 0);
                    foreach (GameDefinition game in games.Definitions)
                        GameRows.Add(new GamePickerRow { Game = game });
                    GamesHasMore = games.HasMore;

                    if (CollectionRows.Count == 0 && GameRows.Count == 0)
                        SetStatus("No games here.");
                }
            }
            catch (Exception ex)
            {
                SetStatus(ex.Message);
            }
            finally
            {
                IsBusy = false;
            }
        }

        private void AddFeaturedItem(FeaturedGameItem item)
        {
            if (item.Definition is not null)
                FeaturedRows.Add(new GamePickerRow { Game = item.Definition });
            else if (item.Collection is not null)
                FeaturedRows.Add(new GamePickerRow { IsCollection = true, Collection = item.Collection });
        }

        private void SetStatus(string message)
        {
            StatusMessage = message;
            ShowStatusMessage = !string.IsNullOrEmpty(message);
        }
    }
}
