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
    /// Reusable base for a Play 3D browse list — a name-ordered, paged
    /// (<see cref="LoadMoreCommand"/>) card list with pull-to-refresh, per-id
    /// thumbnail loading, and the shared Play / Leaderboard actions. Subclasses
    /// supply one page fetch (<see cref="FetchPageAsync"/>); everything else —
    /// paging, image cache, launch routing — is common. A game (or a single-member
    /// collection) launches straight into the host page via <see cref="Play3dLauncher"/>;
    /// a multi-member collection is guarded until the Arcade / Campaign pickers exist.
    /// </summary>
    public abstract partial class Play3dListViewModel : BaseViewModel
    {
        /// <summary>Page size for both the first page and each Load-more.</summary>
        protected const int PageSize = 20;

        private readonly INavigationService _navigationService;
        private readonly IDialogService _dialogService;

        // Per-entity thumbnail cache (key = kind + id); null is cached too, so a
        // game with no image is fetched at most once.
        private readonly Dictionary<string, byte[]?> _imageCache = new();

        /// <summary>The game-library reads (list fetch, collection detail, images).</summary>
        protected IGameLibraryService GameLibrary { get; }

        /// <summary>The cards currently loaded, appended in place across pages.</summary>
        public ObservableCollection<Play3dCardItem> Rows { get; } = new();

        /// <summary>Whether a further page exists beyond what is loaded.</summary>
        [ObservableProperty]
        private bool hasMore;

        /// <summary>Bound to the <c>RefreshView</c> (pull-to-refresh spinner).</summary>
        [ObservableProperty]
        private bool isRefreshing;

        /// <summary>True while a Load-more is in flight (guards re-entry).</summary>
        [ObservableProperty]
        private bool isLoadingMore;

        /// <summary>Empty/error text shown when the list has no rows.</summary>
        [ObservableProperty]
        private string statusMessage = "";

        /// <summary>Whether <see cref="StatusMessage"/> should be shown.</summary>
        [ObservableProperty]
        private bool showStatusMessage;

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

        /// <summary>Fetches one page of cards at the given offset.</summary>
        /// <param name="offset">Row offset to start at</param>
        /// <param name="limit">Page size</param>
        /// <returns>The page of cards plus whether more remain</returns>
        protected abstract Task<Play3dCardPage> FetchPageAsync(int offset, int limit);

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
                HasMore = false;
                Play3dCardPage page = await FetchPageAsync(0, PageSize);
                AppendCards(page);
                if (Rows.Count == 0)
                    SetStatus("Nothing here yet.");
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
        private async Task PlayAsync(Play3dCardItem? card)
        {
            if (card is null)
                return;

            if (!card.IsCollection)
            {
                await Play3dLauncher.LaunchDefinitionAsync(_navigationService, card.Id);
                return;
            }

            // Collection: resolve the access-filtered members before launching, so a
            // collection whose only member is inaccessible guards instead of 404ing.
            try
            {
                GameCollectionDetailResponse detail = await GameLibrary.GetGameCollectionAsync(card.Id);
                Play3dCollectionPlay play = Play3dCollectionLaunch.Resolve(detail.Definitions);
                switch (play.Kind)
                {
                    case Play3dCollectionPlayKind.LaunchSingle:
                        await Play3dLauncher.LaunchDefinitionAsync(_navigationService, play.DefinitionId!);
                        break;
                    case Play3dCollectionPlayKind.NoneAccessible:
                        await _dialogService.ShowAlert("Unavailable", "This collection has no games you can play.", "OK");
                        break;
                    default:
                        await _dialogService.ShowAlert("Coming soon", "Playing multi-game collections isn't available yet.", "OK");
                        break;
                }
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert("Error", ex.Message, "OK");
            }
        }

        [RelayCommand]
        private async Task ShowLeaderboardAsync(Play3dCardItem? card)
        {
            if (card is null || card.IsCollection)
                return;

            // Opens the Leaderboards page; preselecting this game's board is wired
            // when the board selector learns stored-game subjects.
            await _navigationService.GoToAsync("LeaderboardsPage");
        }

        private void AppendCards(Play3dCardPage page)
        {
            foreach (Play3dCardItem card in page.Items)
                Rows.Add(card);
            HasMore = page.HasMore;
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

        private void SetStatus(string message)
        {
            StatusMessage = message;
            ShowStatusMessage = !string.IsNullOrEmpty(message);
        }
    }
}
