using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Extensions;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Drives the Leaderboards page: discovers the player's played subjects, resolves
    /// a Game Type → Game selection into a board subject (a stored maze, or a stored
    /// 3D game chosen from the scoped game picker), and loads that subject's paged
    /// leaderboard (metric toggle + load-more). Reads only through the service
    /// interfaces — no MAUI runtime types — so it is unit-testable in isolation.
    /// </summary>
    public partial class LeaderboardsViewModel : BaseViewModel
    {
        private const int BoardPageSize = 20;
        private const string EmptyMessage = "No winning scores yet";
        private const string ChooseGameMessage = "Choose a game to see its leaderboard.";

        private readonly IScoresService _scoresService;
        private readonly IGameLibraryService _gameLibrary;
        private readonly IMazeService _mazeService;
        private readonly IAuthService _authService;
        private readonly INavigationService _navigationService;
        private readonly IAvatarService _avatarService;
        private readonly IDialogService _dialogService;

        // Whether the caller is an administrator (resolved with their profile) —
        // gates resetting a 3D-game board they don't own.
        private bool _isAdmin;

        // Resolved avatar per player (user_id → PNG bytes or null when none),
        // so a player appearing on multiple rows/pages is fetched once.
        private readonly Dictionary<string, byte[]?> _avatarCache = new();

        private List<MazeOption> _mazes = new();
        private ScoreEntry? _mostRecent;
        private string? _currentUserId;

        // A game id to preselect (set by the page from a `?def=` nav argument when
        // the Leaderboards page is opened from a game card's Leaderboard button).
        private string? _preselectDefinitionId;

        // The currently loaded board, so a redundant reselect is a no-op and
        // load-more knows the subject.
        private string? _loadedKey;
        private ScoreSubject? _currentSubject;
        private ScoreMetric _metric = ScoreMetric.Time;

        // Suppresses the SelectedBoardDate-changed reload while the date list is
        // (re)populated for a new game (the caller reloads the board itself).
        private bool _suppressDateReload;

        /// <summary>The Game Type picker options (fixed).</summary>
        public ObservableCollection<GameTypeOption> GameTypes { get; } = new()
        {
            new GameTypeOption(LeaderboardGameType.MyMazes, "Mazes"),
            new GameTypeOption(LeaderboardGameType.Play3d, "3D Games"),
        };

        /// <summary>The maze Game picker options (Mazes type only).</summary>
        public ObservableCollection<GameOption> Games { get; } = new();

        /// <summary>The selectable board days for a daily 3D game (Today pinned first,
        /// then the days that have boards, most-recent first).</summary>
        public ObservableCollection<BoardDateOption> BoardDates { get; } = new();

        /// <summary>The loaded board rows.</summary>
        public ObservableCollection<LeaderboardRow> Rows { get; } = new();

        /// <summary>The selected Game Type (first cascade level).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(CanPlay))]
        [NotifyPropertyChangedFor(nameof(IsMazeType))]
        [NotifyPropertyChangedFor(nameof(Is3dType))]
        [NotifyPropertyChangedFor(nameof(IsDailyGame))]
        [NotifyCanExecuteChangedFor(nameof(PlayCommand))]
        private GameTypeOption? selectedGameType;

        /// <summary>The selected maze (Mazes type only).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(CanPlay))]
        [NotifyCanExecuteChangedFor(nameof(PlayCommand))]
        private GameOption? selectedGame;

        /// <summary>The picked 3D game (3D Games type only).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(CanPlay))]
        [NotifyPropertyChangedFor(nameof(PickedGameLabel))]
        [NotifyPropertyChangedFor(nameof(IsDailyGame))]
        [NotifyCanExecuteChangedFor(nameof(PlayCommand))]
        private PickedGame? pickedGame;

        /// <summary>The selected board day (daily 3D games only); changing it reloads
        /// that day's board.</summary>
        [ObservableProperty]
        private BoardDateOption? selectedBoardDate;

        /// <summary>Whether a further board page exists.</summary>
        [ObservableProperty]
        private bool hasMore;

        /// <summary>Whether a load-more is in flight.</summary>
        [ObservableProperty]
        private bool isLoadingMore;

        /// <summary>Whether to show the Player column + the caller highlight
        /// (3D-game boards only).</summary>
        [ObservableProperty]
        private bool showPlayerColumn;

        /// <summary>Whether the status message (empty/error) is shown.</summary>
        [ObservableProperty]
        private bool showStatusMessage;

        /// <summary>The status message (empty-state or error text).</summary>
        [ObservableProperty]
        private string statusMessage = "";

        /// <summary>Whether the Fastest Time metric is selected (toggle styling).</summary>
        [ObservableProperty]
        private bool isTimeMetricSelected = true;

        /// <summary>Whether the Highest Score metric is selected (toggle styling).</summary>
        [ObservableProperty]
        private bool isScoreMetricSelected;

        /// <summary>Whether the caller already has a run on the loaded board
        /// (drives the Play / Play-Again label).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(PlayLabel))]
        private bool hasPlayed;

        /// <summary>Whether the Reset control is offered for the loaded board — true
        /// only when the board has rows AND the caller may clear it (a 3D game's board
        /// is the game owner's or an admin's; a stored maze's board is the owner's,
        /// and this page lists only the caller's own mazes).</summary>
        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(ResetBoardCommand))]
        private bool showReset;

        /// <summary>Play-button label: "▶ Play" for a subject the caller hasn't
        /// run, "↻ Play Again" once they have a row on the loaded board.</summary>
        public string PlayLabel => HasPlayed ? "↻ Play Again" : "▶ Play";

        /// <summary>Whether the Play button can launch — a maze (Mazes type) or a
        /// picked 3D game (3D Games type) is selected.</summary>
        public bool CanPlay => SelectedGameType?.Kind == LeaderboardGameType.Play3d
            ? PickedGame is not null
            : SelectedGame is not null;

        /// <summary>Whether the Reset command can run — mirrors <see cref="ShowReset"/>.</summary>
        public bool CanReset => ShowReset;

        /// <summary>Whether the Mazes game type is selected (shows the maze picker).</summary>
        public bool IsMazeType => SelectedGameType?.Kind == LeaderboardGameType.MyMazes;

        /// <summary>Whether the 3D Games type is selected (shows the game picker).</summary>
        public bool Is3dType => SelectedGameType?.Kind == LeaderboardGameType.Play3d;

        /// <summary>Whether a daily 3D game is picked (shows the board-day picker).</summary>
        public bool IsDailyGame => Is3dType && PickedGame?.Rotation == GameVocabulary.Rotation.Daily;

        /// <summary>The picked 3D game's name, or a prompt when none is chosen yet.</summary>
        public string PickedGameLabel => PickedGame?.Name ?? "Choose a game";

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="scoresService">Injected scores service</param>
        /// <param name="gameLibrary">Injected game-library service (3D game lookup/picker)</param>
        /// <param name="mazeService">Injected maze service (maze names + Play launch)</param>
        /// <param name="authService">Injected auth service (caller identity)</param>
        /// <param name="navigationService">Injected navigation service (Play → 3D game)</param>
        /// <param name="avatarService">Injected avatar service (player avatars)</param>
        /// <param name="dialogService">Injected dialog service (game picker + reset confirmation + errors)</param>
        public LeaderboardsViewModel(
            IScoresService scoresService,
            IGameLibraryService gameLibrary,
            IMazeService mazeService,
            IAuthService authService,
            INavigationService navigationService,
            IAvatarService avatarService,
            IDialogService dialogService)
        {
            Title = "Leaderboards";
            _scoresService = scoresService;
            _gameLibrary = gameLibrary;
            _mazeService = mazeService;
            _authService = authService;
            _navigationService = navigationService;
            _avatarService = avatarService;
            _dialogService = dialogService;
        }

        /// <summary>Sets the game to preselect (from a card's Leaderboard button); applied on Initialize.</summary>
        /// <param name="definitionId">The game definition id, or <c>null</c></param>
        public void SetPreselectGame(string? definitionId) => _preselectDefinitionId = definitionId;

        // Repopulating the maze Game list is synchronous; the board reload is driven
        // by the page (picker change → ReloadBoardCommand) and InitializeAsync.
        partial void OnSelectedGameTypeChanged(GameTypeOption? value) => PopulateGames();

        // A user-driven board-day change reloads that day's board. Suppressed while
        // the date list is (re)populated for a new game (the caller reloads then),
        // and while a load is already in flight (InitializeAsync sets the default).
        partial void OnSelectedBoardDateChanged(BoardDateOption? value)
        {
            if (_suppressDateReload || IsBusy)
                return;
            _ = ReloadWithBusyAsync(force: true);
        }

        /// <summary>
        /// Discovers the player's played subjects, picks the default selection, and
        /// loads its board. Invoked by the page on first appearance.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private async Task InitializeAsync()
        {
            if (IsBusy)
                return;

            IsBusy = true;
            ShowStatusMessage = false;
            try
            {
                _currentUserId = await ResolveCurrentUserIdAsync();
                await DiscoverSubjectsAsync();
                await ApplyDefaultSelectionAsync();
                await RefreshBoardDatesAsync();
                await LoadBoardCoreAsync(force: true);
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

        /// <summary>
        /// Reloads the board for the current selection (invoked by the page when a
        /// picker changes). A reselect that resolves to the already-loaded board is
        /// a no-op.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private Task ReloadBoardAsync() => ReloadWithBusyAsync(force: false);

        /// <summary>
        /// Re-fetches the current board for the current selection (toolbar Refresh),
        /// resetting to the first page. Force-reloads past the same-key no-op.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private Task RefreshAsync() => ReloadWithBusyAsync(force: true);

        /// <summary>Selects the Fastest Time metric and reloads.</summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private Task SelectTimeMetricAsync() => SetMetricAsync(ScoreMetric.Time);

        /// <summary>Selects the Highest Score metric and reloads.</summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private Task SelectScoreMetricAsync() => SetMetricAsync(ScoreMetric.Score);

        /// <summary>
        /// Opens the scoped game picker; on a selection, shows that game's board.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private async Task PickGameAsync()
        {
            GameDefinition? picked = await _dialogService.ShowGamePickerAsync();
            if (picked is null)
                return;

            SelectGame(PickedGame.From(picked));
            await RefreshBoardDatesAsync();
            await ReloadWithBusyAsync(force: true);
        }

        /// <summary>
        /// Appends the next page to the current board.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand]
        private async Task LoadMoreAsync()
        {
            if (IsLoadingMore || !HasMore || _currentSubject is null)
                return;

            IsLoadingMore = true;
            try
            {
                var resp = await _scoresService.GetLeaderboardAsync(
                    _currentSubject.Value, _metric, null, BoardPageSize, Rows.Count, ShowPlayerColumn);
                await LoadAvatarsForRowsAsync(AppendRows(resp));
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

        /// <summary>
        /// Launches the selected subject in 3D — a personal maze with its saved
        /// settings, or the picked stored game (the host page fetches its config by
        /// id). Direct launch, no prompt.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand(CanExecute = nameof(CanPlay))]
        private async Task PlayAsync()
        {
            if (Is3dType)
            {
                if (PickedGame is null)
                    return;
                await _navigationService.GoToAsync(nameof(Play3dGamePage), new Dictionary<string, object>
                {
                    { "def", PickedGame.Id },
                });
                return;
            }

            GameOption? game = SelectedGame;
            if (game is null)
                return;

            // Load the full maze for its saved settings; Play3dGamePage appends them
            // to the /game/?id= URL (the MAUI WebView can't read the SPA's
            // localStorage, so settings ride the query string).
            MazeItem full = await _mazeService.GetMazeItem(game.MazeId) ?? new MazeItem { ID = game.MazeId };

            // Reject an empty / cleared maze before launching (Definition.Solve()
            // throws when unsolvable), the same validation the Mazes page and Maze
            // Editor Play-3D buttons use.
            try { full.Definition?.Solve(); }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert("MAZE", $"Cannot play maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
                return;
            }

            await _navigationService.GoToAsync(nameof(Play3dGamePage), new Dictionary<string, object>
            {
                { "MazeItem", full },
                { "LaunchSettings", full.GameSettings ?? new MazeGameSettings() },
            });
        }

        /// <summary>
        /// Resets the loaded board to empty after a destructive-confirm prompt, then
        /// reloads it (now empty, so the Reset control hides). The server enforces
        /// access independently of this UI gating.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand(CanExecute = nameof(CanReset))]
        private async Task ResetBoardAsync()
        {
            if (IsBusy || _currentSubject is not ScoreSubject subject || !SubjectAllowsReset() || Rows.Count == 0)
                return;

            bool confirmed = await _dialogService.ShowConfirmation(
                "Reset Leaderboard",
                "This permanently deletes every score on this leaderboard. This cannot be undone.",
                "Reset",
                "Cancel",
                isDestructive: true);
            if (!confirmed)
                return;

            IsBusy = true;
            ShowStatusMessage = false;
            try
            {
                await _scoresService.ClearLeaderboardAsync(subject);
                await LoadBoardCoreAsync(force: true);
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert("Error", $"Failed to reset leaderboard\n\n{ex.Message}", "OK");
            }
            finally
            {
                IsBusy = false;
            }
        }

        private async Task SetMetricAsync(ScoreMetric metric)
        {
            if (_metric == metric)
                return;

            _metric = metric;
            IsTimeMetricSelected = metric == ScoreMetric.Time;
            IsScoreMetricSelected = metric == ScoreMetric.Score;
            await ReloadWithBusyAsync(force: true);
        }

        private async Task ReloadWithBusyAsync(bool force)
        {
            if (IsBusy)
                return;

            IsBusy = true;
            ShowStatusMessage = false;
            try
            {
                await LoadBoardCoreAsync(force);
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

        private async Task LoadBoardCoreAsync(bool force)
        {
            ScoreSubject? subject = ResolveSubject();
            string? key = subject is null ? null : $"{_metric}|{SubjectKey(subject.Value)}";
            if (!force && key == _loadedKey)
                return;

            _loadedKey = key;
            _currentSubject = subject;
            ShowPlayerColumn = Is3dType;
            Rows.Clear();
            HasMore = false;
            HasPlayed = false;
            ShowReset = false;

            if (subject is null)
            {
                SetStatus(Is3dType && PickedGame is null ? ChooseGameMessage : EmptyMessage);
                return;
            }

            var resp = await _scoresService.GetLeaderboardAsync(
                subject.Value, _metric, null, BoardPageSize, 0, ShowPlayerColumn);
            List<LeaderboardRow> added = AppendRows(resp);
            SetStatusForRows();
            UpdateShowReset();
            await LoadAvatarsForRowsAsync(added);
        }

        // The Reset control shows only when the board has rows and the caller may
        // clear it: a 3D game's board is the game owner's or an admin's; a stored
        // maze's board is the owner's (this page lists only the caller's own mazes).
        private bool SubjectAllowsReset()
        {
            if (_currentSubject is not ScoreSubject subject)
                return false;
            if (subject.MazeId is not null)
                return true;
            return _isAdmin || (PickedGame is not null && PickedGame.OwnerId == _currentUserId);
        }

        private void UpdateShowReset() => ShowReset = Rows.Count > 0 && SubjectAllowsReset();

        private ScoreSubject? ResolveSubject()
        {
            if (Is3dType)
                return PickedGame is null
                    ? null
                    : ScoreSubject.ForChallenge(GameChallenge.For(PickedGame.Id, PickedGame.Rotation, SelectedBoardDate?.DateUtc));

            return SelectedGame is null ? null : ScoreSubject.ForMaze(SelectedGame.MazeId);
        }

        // (Re)builds the board-day list for the picked game. For a daily game: Today
        // is pinned first, then the days that have boards (GetBoardDatesAsync,
        // newest-first, today deduped); the default selection is the most-recent day
        // that actually has runs (Today only when today itself has runs, or when the
        // game has no runs at all). For anything else the list is cleared/hidden.
        // Repopulating is guarded so it doesn't trigger a board reload — the caller
        // reloads once after this.
        private async Task RefreshBoardDatesAsync()
        {
            _suppressDateReload = true;
            try
            {
                BoardDates.Clear();
                SelectedBoardDate = null;
                if (!IsDailyGame || PickedGame is null)
                    return;

                string today = GameChallenge.TodayUtc();
                var todayOption = BoardDateOption.Today(today);
                BoardDates.Add(todayOption);

                // The default is the most-recent day with runs: if that's today (or
                // there are no runs) it's the Today pin, else the matching past-day
                // option captured as it's added.
                string? mostRecentWithRuns = null;
                BoardDateOption? defaultPastDay = null;
                try
                {
                    BoardDatesResponse resp = await _scoresService.GetBoardDatesAsync(PickedGame.Id);
                    mostRecentWithRuns = resp.Dates.Count > 0 ? resp.Dates[0] : null;
                    foreach (string date in resp.Dates)
                    {
                        if (date == today)
                            continue;
                        var option = BoardDateOption.ForDate(date);
                        BoardDates.Add(option);
                        if (date == mostRecentWithRuns)
                            defaultPastDay = option;
                    }
                }
                catch
                {
                    // Best-effort: at least Today stays selectable.
                }

                SelectedBoardDate = defaultPastDay ?? todayOption;
            }
            finally
            {
                _suppressDateReload = false;
            }
        }

        private List<LeaderboardRow> AppendRows(ScoreboardResponse resp)
        {
            var added = new List<LeaderboardRow>();
            int rank = Rows.Count;
            foreach (ScoreEntry entry in resp.Scores)
            {
                rank++;
                bool isMe = _currentUserId is not null && entry.UserId == _currentUserId;
                if (isMe)
                    HasPlayed = true;
                var row = new LeaderboardRow(rank, entry, isMe && ShowPlayerColumn, ShowPlayerColumn);
                Rows.Add(row);
                added.Add(row);
            }
            HasMore = resp.HasMore;
            return added;
        }

        /// <summary>
        /// Resolves and swaps in player avatars for the given rows. Only runs on
        /// boards that show the Player column; each player is fetched at most once
        /// (cached by user id across rows and pages). Rows for players with no
        /// avatar keep their <c>null</c> source, so the control shows the placeholder.
        /// </summary>
        private async Task LoadAvatarsForRowsAsync(IReadOnlyList<LeaderboardRow> rows)
        {
            if (!ShowPlayerColumn)
                return;

            foreach (LeaderboardRow row in rows)
            {
                if (string.IsNullOrEmpty(row.AvatarUpdatedAt))
                    continue;

                if (!_avatarCache.TryGetValue(row.UserId, out byte[]? bytes))
                {
                    bytes = await _avatarService.TryLoadAvatarBytesAsync(row.UserId, row.AvatarUpdatedAt);
                    _avatarCache[row.UserId] = bytes;
                }

                if (bytes is not null)
                    row.AvatarBytes = bytes;
            }
        }

        private async Task DiscoverSubjectsAsync()
        {
            // The Mazes game type lists ALL the player's mazes (scored or not).
            List<MazeItem> mazes = await _mazeService.GetMazeItems(false);
            _mazes = mazes
                .Select(maze => new MazeOption(maze.ID, maze.Name))
                .OrderBy(m => m.Name, StringComparer.CurrentCulture)
                .ToList();

            // Only the most-recent run is needed — to pick the default subject.
            ScoreboardResponse history = await _scoresService.GetScoreHistoryAsync(1, 0);
            _mostRecent = history.Scores.FirstOrDefault();
        }

        // The board to show first: a game preselected from its card, else the subject
        // of the caller's most-recent run (their maze, or the 3D game behind a
        // `def:<id>` challenge — resolved via the access-checked play-fetch so a
        // gone / inaccessible game falls through), else their first maze, else
        // 3D Games with no game picked.
        private async Task ApplyDefaultSelectionAsync()
        {
            if (!string.IsNullOrEmpty(_preselectDefinitionId) && await TrySelectGameByIdAsync(_preselectDefinitionId))
                return;

            if (_mostRecent?.MazeId is not null)
            {
                string? resolved = ResolveMazeId(_mostRecent.MazeId);
                if (resolved is not null)
                {
                    SelectMaze(resolved);
                    return;
                }
            }

            string? gameId = _mostRecent?.Challenge is not null ? GameChallenge.DefinitionIdFromChallenge(_mostRecent.Challenge) : null;
            if (gameId is not null && await TrySelectGameByIdAsync(gameId))
                return;

            if (_mazes.Count > 0)
            {
                SelectMaze(_mazes[0].MazeId);
                return;
            }

            SelectedGameType = GameTypes.First(t => t.Kind == LeaderboardGameType.Play3d);
            PickedGame = null;
        }

        private async Task<bool> TrySelectGameByIdAsync(string definitionId)
        {
            try
            {
                GamePlayResponse def = await _gameLibrary.GetGameDefinitionAsync(definitionId);
                SelectGame(PickedGame.From(def));
                return true;
            }
            catch
            {
                return false; // gone / no longer accessible
            }
        }

        // Resolve a history maze_id to a maze in the list — exact id, then by
        // filename (FileStore ids are paths that may differ between a score row
        // and the maze list).
        private string? ResolveMazeId(string historyId)
        {
            if (_mazes.Any(m => m.MazeId == historyId))
                return historyId;
            string basename = Basename(historyId);
            return _mazes.FirstOrDefault(m => Basename(m.MazeId) == basename)?.MazeId;
        }

        private void SelectMaze(string mazeId)
        {
            SelectedGameType = GameTypes.First(t => t.Kind == LeaderboardGameType.MyMazes);
            SelectedGame = Games.FirstOrDefault(g => g.MazeId == mazeId) ?? Games.FirstOrDefault();
        }

        private void SelectGame(PickedGame game)
        {
            SelectedGameType = GameTypes.First(t => t.Kind == LeaderboardGameType.Play3d);
            PickedGame = game;
        }

        private void PopulateGames()
        {
            Games.Clear();
            if (IsMazeType)
            {
                foreach (MazeOption maze in _mazes)
                    Games.Add(GameOption.ForMaze(maze.MazeId, maze.Name));
                SelectedGame = Games.FirstOrDefault();
            }
            else
            {
                SelectedGame = null;
            }
        }

        private async Task<string?> ResolveCurrentUserIdAsync()
        {
            // Highlight is best-effort: a failure here just disables it.
            try
            {
                UserProfile profile = await _authService.GetMyProfileAsync();
                _isAdmin = profile?.IsAdmin ?? false;
                return string.IsNullOrEmpty(profile?.Id) ? null : profile!.Id;
            }
            catch
            {
                _isAdmin = false;
                return null;
            }
        }

        private void SetStatus(string message)
        {
            StatusMessage = message;
            ShowStatusMessage = true;
        }

        private void SetStatusForRows()
        {
            if (Rows.Count == 0)
            {
                SetStatus(EmptyMessage);
            }
            else
            {
                StatusMessage = "";
                ShowStatusMessage = false;
            }
        }

        private static string SubjectKey(ScoreSubject subject) =>
            subject.MazeId is not null ? $"m:{subject.MazeId}" : $"c:{subject.Challenge}";

        private static string Basename(string id)
        {
            int idx = Math.Max(id.LastIndexOf('/'), id.LastIndexOf('\\'));
            return idx >= 0 ? id[(idx + 1)..] : id;
        }
    }
}
