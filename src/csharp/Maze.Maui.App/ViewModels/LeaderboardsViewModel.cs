using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Drives the Leaderboards page: discovers the player's played subjects,
    /// cascades a Game Type → Game selection into a board subject, and loads that
    /// subject's paged leaderboard (metric toggle + load-more). Reads only through
    /// the service interfaces — no MAUI runtime types — so it is unit-testable in
    /// isolation. The scoring data layer stays <c>Scores*</c>; this UI layer is
    /// <c>Leaderboards*</c>.
    /// </summary>
    public partial class LeaderboardsViewModel : BaseViewModel
    {
        private const int BoardPageSize = 20;
        private const string EmptyMessage = "No winning scores yet";

        private readonly IScoresService _scoresService;
        private readonly IGameConfigService _gameConfigService;
        private readonly IMazeService _mazeService;
        private readonly IAuthService _authService;
        private readonly INavigationService _navigationService;
        private readonly IAvatarService _avatarService;

        // Resolved avatar per player (user_id → PNG bytes or null when none),
        // so a player appearing on multiple rows/pages is fetched once.
        private readonly Dictionary<string, byte[]?> _avatarCache = new();

        // difficulty → fixed seed; the seeds don't change, so resolve each once.
        private readonly Dictionary<Difficulty, ulong> _seedCache = new();
        private List<MazeOption> _mazes = new();
        private ScoreEntry? _mostRecent;
        private string? _currentUserId;

        // The currently loaded board, so a redundant reselect (e.g. the same
        // subject firing both picker changes) is a no-op and load-more knows the
        // subject.
        private string? _loadedKey;
        private ScoreSubject? _currentSubject;
        private ScoreMetric _metric = ScoreMetric.Time;

        /// <summary>The Game Type picker options (fixed).</summary>
        public ObservableCollection<GameTypeOption> GameTypes { get; } = new()
        {
            new GameTypeOption(LeaderboardGameType.MyMazes, "Mazes"),
            new GameTypeOption(LeaderboardGameType.Play3d, "Play 3D"),
        };

        /// <summary>The Game picker options for the selected Game Type.</summary>
        public ObservableCollection<GameOption> Games { get; } = new();

        /// <summary>The loaded board rows.</summary>
        public ObservableCollection<LeaderboardRow> Rows { get; } = new();

        /// <summary>The selected Game Type (first cascade level).</summary>
        [ObservableProperty]
        private GameTypeOption? selectedGameType;

        /// <summary>The selected Game (second cascade level).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(CanPlay))]
        [NotifyCanExecuteChangedFor(nameof(PlayCommand))]
        private GameOption? selectedGame;

        /// <summary>Whether a further board page exists.</summary>
        [ObservableProperty]
        private bool hasMore;

        /// <summary>Whether a load-more is in flight.</summary>
        [ObservableProperty]
        private bool isLoadingMore;

        /// <summary>Whether to show the Player column + the caller highlight
        /// (Play-3D boards only).</summary>
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

        /// <summary>Play-button label: "▶ Play" for a subject the caller hasn't
        /// run, "↻ Play Again" once they have a row on the loaded board.</summary>
        public string PlayLabel => HasPlayed ? "↻ Play Again" : "▶ Play";

        /// <summary>Whether the Play button can launch — true when a game subject
        /// is selected (false e.g. for the Mazes type when the player has none).</summary>
        public bool CanPlay => SelectedGame is not null;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="scoresService">Injected scores service</param>
        /// <param name="gameConfigService">Injected game-config service (curated seeds)</param>
        /// <param name="mazeService">Injected maze service (maze names + Play launch)</param>
        /// <param name="authService">Injected auth service (caller identity)</param>
        /// <param name="navigationService">Injected navigation service (Play → 3D game)</param>
        /// <param name="avatarService">Injected avatar service (player avatars)</param>
        public LeaderboardsViewModel(
            IScoresService scoresService,
            IGameConfigService gameConfigService,
            IMazeService mazeService,
            IAuthService authService,
            INavigationService navigationService,
            IAvatarService avatarService)
        {
            Title = "Leaderboards";
            _scoresService = scoresService;
            _gameConfigService = gameConfigService;
            _mazeService = mazeService;
            _authService = authService;
            _navigationService = navigationService;
            _avatarService = avatarService;
        }

        // Repopulating the Game list is synchronous; the board reload is driven by
        // the page (picker change → ReloadBoardCommand) and InitializeAsync.
        partial void OnSelectedGameTypeChanged(GameTypeOption? value) => PopulateGames();

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
                ApplyDefaultSelection();
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
        /// settings, or a curated difficulty (server resolves the preset). Direct
        /// launch, no prompt.
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommand(CanExecute = nameof(CanPlay))]
        private async Task PlayAsync()
        {
            GameOption? game = SelectedGame;
            if (game is null)
                return;

            if (game.Difficulty is Difficulty difficulty)
            {
                await _navigationService.GoToAsync(nameof(Play3dGamePage), new Dictionary<string, object>
                {
                    { "difficulty", difficulty.ToQueryValue() },
                });
                return;
            }

            if (game.MazeId is not null)
            {
                // Load the full maze for its saved settings; Play3dGamePage appends
                // them to the /game/?id= URL (the MAUI WebView can't read the SPA's
                // localStorage, so settings ride the query string).
                MazeItem full = await _mazeService.GetMazeItem(game.MazeId) ?? new MazeItem { ID = game.MazeId };
                await _navigationService.GoToAsync(nameof(Play3dGamePage), new Dictionary<string, object>
                {
                    { "MazeItem", full },
                    { "LaunchSettings", full.GameSettings ?? new MazeGameSettings() },
                });
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
            ScoreSubject? subject = await ResolveSubjectAsync();
            string? key = subject is null ? null : $"{_metric}|{SubjectKey(subject.Value)}";
            if (!force && key == _loadedKey)
                return;

            _loadedKey = key;
            _currentSubject = subject;
            ShowPlayerColumn = SelectedGameType?.Kind == LeaderboardGameType.Play3d;
            Rows.Clear();
            HasMore = false;
            HasPlayed = false;

            if (subject is null)
            {
                SetStatus(EmptyMessage);
                return;
            }

            var resp = await _scoresService.GetLeaderboardAsync(
                subject.Value, _metric, null, BoardPageSize, 0, ShowPlayerColumn);
            List<LeaderboardRow> added = AppendRows(resp);
            SetStatusForRows();
            await LoadAvatarsForRowsAsync(added);
        }

        private async Task<ScoreSubject?> ResolveSubjectAsync()
        {
            GameOption? game = SelectedGame;
            if (game is null)
                return null;

            if (game.MazeId is not null)
                return ScoreSubject.ForMaze(game.MazeId);

            if (game.Difficulty is Difficulty difficulty)
            {
                if (!_seedCache.TryGetValue(difficulty, out ulong seed))
                {
                    Play3dConfig config = await _gameConfigService.GetPlay3dConfigAsync(difficulty);
                    seed = config.Seed;
                    _seedCache[difficulty] = seed;
                }
                return ScoreSubject.ForCuratedGame(difficulty.ToQueryValue(), seed);
            }

            return null;
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
        /// avatar keep their <c>null</c> source, so the control shows the
        /// placeholder.
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
            // The Mazes game type lists ALL the player's mazes (scored or not),
            // mirroring the Play-3D list showing every difficulty.
            List<MazeItem> mazes = await _mazeService.GetMazeItems(false);
            _mazes = mazes
                .Select(maze => new MazeOption(maze.ID, maze.Name))
                .OrderBy(m => m.Name, StringComparer.CurrentCulture)
                .ToList();

            // Only the most-recent run is needed — to pick the default subject.
            ScoreboardResponse history = await _scoresService.GetScoreHistoryAsync(1, 0);
            _mostRecent = history.Scores.FirstOrDefault();
        }

        private void ApplyDefaultSelection()
        {
            if (_mostRecent?.MazeId is not null)
            {
                string? resolved = ResolveMazeId(_mostRecent.MazeId);
                if (resolved is not null)
                {
                    SelectMaze(resolved);
                    return;
                }
            }
            if (_mostRecent?.Challenge is not null)
            {
                SelectDifficulty(ParseDifficulty(_mostRecent.Challenge));
                return;
            }
            if (_mazes.Count > 0)
            {
                SelectMaze(_mazes[0].MazeId);
                return;
            }
            SelectDifficulty(Difficulty.Easy);
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

        private void SelectDifficulty(Difficulty difficulty)
        {
            SelectedGameType = GameTypes.First(t => t.Kind == LeaderboardGameType.Play3d);
            SelectedGame = Games.FirstOrDefault(g => g.Difficulty == difficulty) ?? Games.FirstOrDefault();
        }

        private void PopulateGames()
        {
            Games.Clear();
            if (SelectedGameType?.Kind == LeaderboardGameType.Play3d)
            {
                foreach (Difficulty difficulty in new[] { Difficulty.Easy, Difficulty.Tricky, Difficulty.Hard })
                    Games.Add(GameOption.ForDifficulty(difficulty));
            }
            else
            {
                foreach (MazeOption maze in _mazes)
                    Games.Add(GameOption.ForMaze(maze.MazeId, maze.Name));
            }
            SelectedGame = Games.FirstOrDefault();
        }

        private async Task<string?> ResolveCurrentUserIdAsync()
        {
            // Highlight is best-effort: a failure here just disables it.
            try
            {
                UserProfile profile = await _authService.GetMyProfileAsync();
                return string.IsNullOrEmpty(profile?.Id) ? null : profile!.Id;
            }
            catch
            {
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

        private static Difficulty ParseDifficulty(string challenge)
        {
            string token = challenge.Split(':')[0];
            return Enum.TryParse(token, ignoreCase: true, out Difficulty difficulty) ? difficulty : Difficulty.Easy;
        }

        private static string Basename(string id)
        {
            int idx = Math.Max(id.LastIndexOf('/'), id.LastIndexOf('\\'));
            return idx >= 0 ? id[(idx + 1)..] : id;
        }
    }
}
