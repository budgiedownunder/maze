using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>One fetched page of already-mapped cards plus whether more remain.</summary>
    /// <param name="Items">The page of cards</param>
    /// <param name="HasMore">Whether a further page exists</param>
    public sealed record Play3dCardPage(IReadOnlyList<Play3dCardItem> Items, bool HasMore);

    /// <summary>
    /// Reusable base for a single Play 3D browse list — a name-ordered, paged
    /// (<see cref="LoadMoreCommand"/>) card list with pull-to-refresh, per-id
    /// thumbnail loading, a search box, and the shared Play / Leaderboard actions.
    /// Subclasses supply one page fetch (<see cref="FetchPageAsync"/>); everything
    /// else — paging, image cache, search, launch routing — is common, so the
    /// Featured list and each scope tab reuse it (mirroring the web client's single
    /// list body). Search is <b>client-side</b> by default (filters the loaded pages
    /// by name); a subclass whose scope is unbounded overrides <see cref="UsesServerSearch"/>
    /// so the query goes to the server instead.
    /// </summary>
    public abstract partial class Play3dListViewModel : BaseViewModel
    {
        /// <summary>Page size for both the first page and each Load-more.</summary>
        protected const int PageSize = 20;

        private const int SearchDebounceMs = 300;

        private readonly INavigationService _navigationService;
        private readonly IDialogService _dialogService;

        // Per-entity thumbnail cache (key = kind + id); null is cached too, so a
        // game with no image is fetched at most once.
        private readonly Dictionary<string, byte[]?> _imageCache = new();

        private CancellationTokenSource? _searchCts;
        private GameListSort? _sort;

        /// <summary>The game-library reads (list fetch, collection detail, images).</summary>
        protected IGameLibraryService GameLibrary { get; }

        /// <summary>Every card loaded from the server, across all fetched pages.</summary>
        public ObservableCollection<Play3dCardItem> Rows { get; } = new();

        /// <summary>
        /// The cards the list actually shows: <see cref="Rows"/> filtered by
        /// <see cref="FilterText"/> when this list searches client-side, or identical
        /// to <see cref="Rows"/> when it searches server-side (the server already
        /// filtered). The <c>CollectionView</c> binds to this.
        /// </summary>
        public ObservableCollection<Play3dCardItem> VisibleRows { get; } = new();

        /// <summary>Whether a further page exists beyond what is loaded.</summary>
        [ObservableProperty]
        private bool hasMore;

        /// <summary>Bound to the <c>RefreshView</c> (pull-to-refresh spinner).</summary>
        [ObservableProperty]
        private bool isRefreshing;

        /// <summary>True while a Load-more is in flight (guards re-entry).</summary>
        [ObservableProperty]
        private bool isLoadingMore;

        /// <summary>Empty/error/no-match text shown when the list has no visible rows.</summary>
        [ObservableProperty]
        private string statusMessage = "";

        /// <summary>Whether <see cref="StatusMessage"/> should be shown.</summary>
        [ObservableProperty]
        private bool showStatusMessage;

        /// <summary>The search box text (client filter or server query per <see cref="UsesServerSearch"/>).</summary>
        [ObservableProperty]
        private string filterText = "";

        /// <summary>Constructor</summary>
        /// <param name="gameLibrary">Injected game-library read service</param>
        /// <param name="navigationService">Injected navigation service</param>
        /// <param name="dialogService">Injected dialog service (launch guards)</param>
        protected Play3dListViewModel(IGameLibraryService gameLibrary, INavigationService navigationService, IDialogService dialogService)
        {
            GameLibrary = gameLibrary;
            _navigationService = navigationService;
            _dialogService = dialogService;
        }

        /// <summary>The placeholder shown in this list's search box.</summary>
        public virtual string SearchPlaceholder => "Filter…";

        /// <summary>The message shown when the list loads empty; subclasses tailor it per scope.</summary>
        protected virtual string EmptyMessage => "Nothing here yet.";

        /// <summary>
        /// True when the search text is sent to the server (an unbounded scope like
        /// Community, where a match may not be in the loaded pages) rather than
        /// filtering the already-loaded pages client-side.
        /// </summary>
        protected virtual bool UsesServerSearch => false;

        /// <summary>The query forwarded to the server — server-search lists only, else <c>null</c>.</summary>
        protected string? ServerQuery => UsesServerSearch && !string.IsNullOrWhiteSpace(FilterText) ? FilterText.Trim() : null;

        /// <summary>The sort forwarded to the server — server-search lists only, else <c>null</c>.</summary>
        protected GameListSort? SortOrder => _sort;

        /// <summary>Fetches one page of cards at the given offset.</summary>
        /// <param name="offset">Row offset to start at</param>
        /// <param name="limit">Page size</param>
        /// <returns>The page of cards plus whether more remain</returns>
        protected abstract Task<Play3dCardPage> FetchPageAsync(int offset, int limit);

        /// <summary>Stores the server sort for the next load (does not itself reload).</summary>
        /// <param name="sort">The sort, or <c>null</c> for the server default</param>
        public void SetSort(GameListSort? sort) => _sort = sort;

        /// <summary>
        /// Loads the first page from scratch — clears the list, fetches offset 0,
        /// and shows an empty-state message when nothing comes back. Called on first
        /// appear and by <see cref="RefreshCommand"/>.
        /// </summary>
        /// <returns>Task</returns>
        public async Task LoadFirstPageAsync()
        {
            IsBusy = true;
            SetStatus("");
            try
            {
                Rows.Clear();
                VisibleRows.Clear();
                HasMore = false;
                Play3dCardPage page = await FetchPageAsync(0, PageSize);
                AppendCards(page);
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

        [RelayCommand]
        private async Task RefreshAsync()
        {
            IsRefreshing = true;
            try
            {
                await LoadFirstPageAsync();
            }
            finally
            {
                IsRefreshing = false;
            }
        }

        [RelayCommand]
        private async Task LoadMoreAsync()
        {
            if (IsLoadingMore || !HasMore)
                return;

            IsLoadingMore = true;
            try
            {
                Play3dCardPage page = await FetchPageAsync(Rows.Count, PageSize);
                AppendCards(page);
            }
            catch (Exception ex)
            {
                SetStatus(ex.Message);
            }
            finally
            {
                IsLoadingMore = false;
            }
        }

        [RelayCommand]
        private Task PlayAsync(Play3dCardItem? card)
            => card is null ? Task.CompletedTask : Play3dCardActions.PlayAsync(card, _navigationService, GameLibrary, _dialogService);

        [RelayCommand]
        private Task ShowLeaderboardAsync(Play3dCardItem? card)
            => card is null ? Task.CompletedTask : Play3dCardActions.ShowLeaderboardAsync(card, _navigationService);

        partial void OnFilterTextChanged(string value)
        {
            if (UsesServerSearch)
                DebounceServerReload();
            else
                ApplyClientFilter();
        }

        // Coalesce rapid keystrokes into one server reload, mirroring the web client.
        private void DebounceServerReload()
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
            await LoadFirstPageAsync();
        }

        // Rebuild the shown rows from the loaded pages (client-search lists only).
        private void ApplyClientFilter()
        {
            VisibleRows.Clear();
            foreach (Play3dCardItem card in Rows)
                if (Matches(card))
                    VisibleRows.Add(card);
            UpdateStatus();
        }

        private bool Matches(Play3dCardItem card)
            => UsesServerSearch
               || string.IsNullOrWhiteSpace(FilterText)
               || card.Name.Contains(FilterText.Trim(), StringComparison.CurrentCultureIgnoreCase);

        private void AppendCards(Play3dCardPage page)
        {
            foreach (Play3dCardItem card in page.Items)
            {
                Rows.Add(card);
                if (Matches(card))
                    VisibleRows.Add(card);
            }
            HasMore = page.HasMore;
            UpdateStatus();
            _ = LoadImagesAsync(page.Items);
        }

        private async Task LoadImagesAsync(IReadOnlyList<Play3dCardItem> cards)
        {
            foreach (Play3dCardItem card in cards)
            {
                if (string.IsNullOrEmpty(card.ImageUpdatedAt))
                    continue;

                string key = $"{card.Kind}:{card.Id}";
                if (!_imageCache.TryGetValue(key, out byte[]? bytes))
                {
                    bytes = await GameLibrary.GetGameImageAsync(card.Kind, card.Id, card.ImageUpdatedAt);
                    _imageCache[key] = bytes;
                }

                if (bytes is not null)
                    card.ImageBytes = bytes;
            }
        }

        // Cleared when something is shown; "No matches." when a search (client or
        // server) yields nothing; otherwise the scope's empty-state message.
        private void UpdateStatus()
        {
            if (VisibleRows.Count > 0)
                SetStatus("");
            else
                SetStatus(string.IsNullOrWhiteSpace(FilterText) ? EmptyMessage : "No matches.");
        }

        private void SetStatus(string message)
        {
            StatusMessage = message;
            ShowStatusMessage = !string.IsNullOrEmpty(message);
        }
    }
}
