using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;

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
        // Bound the discovery scan for a very active player; the server caps each
        // history page at 100.
        private const int MaxDiscoveryPages = 25;
        private const int DiscoveryPageSize = 100;
        private const int BoardPageSize = 20;
        private const string EmptyMessage = "No winning scores yet";

        private readonly IScoresService _scoresService;
        private readonly IGameConfigService _gameConfigService;
        private readonly IMazeService _mazeService;
        private readonly IAuthService _authService;

        // difficulty → fixed seed; the seeds don't change, so resolve each once.
        private readonly Dictionary<Difficulty, ulong> _seedCache = new();
        private List<PlayedMaze> _playedMazes = new();
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
            new GameTypeOption(LeaderboardGameType.MyMazes, "My Mazes"),
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

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="scoresService">Injected scores service</param>
        /// <param name="gameConfigService">Injected game-config service (curated seeds)</param>
        /// <param name="mazeService">Injected maze service (maze names)</param>
        /// <param name="authService">Injected auth service (caller identity)</param>
        public LeaderboardsViewModel(
            IScoresService scoresService,
            IGameConfigService gameConfigService,
            IMazeService mazeService,
            IAuthService authService)
        {
            Title = "Leaderboards";
            _scoresService = scoresService;
            _gameConfigService = gameConfigService;
            _mazeService = mazeService;
            _authService = authService;
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
                AppendRows(resp);
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

            if (subject is null)
            {
                SetStatus(EmptyMessage);
                return;
            }

            var resp = await _scoresService.GetLeaderboardAsync(
                subject.Value, _metric, null, BoardPageSize, 0, ShowPlayerColumn);
            AppendRows(resp);
            SetStatusForRows();
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

        private void AppendRows(ScoreboardResponse resp)
        {
            int rank = Rows.Count;
            foreach (ScoreEntry entry in resp.Scores)
            {
                rank++;
                bool highlight = ShowPlayerColumn && _currentUserId is not null && entry.UserId == _currentUserId;
                Rows.Add(new LeaderboardRow(rank, entry, highlight, ShowPlayerColumn));
            }
            HasMore = resp.HasMore;
        }

        private async Task DiscoverSubjectsAsync()
        {
            var orderedIds = new List<string>();
            var seen = new HashSet<string>();
            _mostRecent = null;

            int offset = 0;
            for (int page = 0; page < MaxDiscoveryPages; page++)
            {
                ScoreboardResponse resp = await _scoresService.GetScoreHistoryAsync(DiscoveryPageSize, offset);
                if (page == 0)
                    _mostRecent = resp.Scores.FirstOrDefault();
                foreach (ScoreEntry row in resp.Scores)
                {
                    if (row.MazeId is not null && seen.Add(row.MazeId))
                        orderedIds.Add(row.MazeId);
                }
                if (!resp.HasMore)
                    break;
                offset += DiscoveryPageSize;
            }

            List<MazeItem> mazes = await _mazeService.GetMazeItems(false);
            var nameById = new Dictionary<string, string>();
            var nameByBasename = new Dictionary<string, string>();
            foreach (MazeItem maze in mazes)
            {
                nameById[maze.ID] = maze.Name;
                nameByBasename[Basename(maze.ID)] = maze.Name;
            }

            _playedMazes = orderedIds
                .Select(id => new PlayedMaze(id, MazeLabel(id, nameById, nameByBasename)))
                .OrderBy(m => m.Name, StringComparer.CurrentCulture)
                .ToList();
        }

        private void ApplyDefaultSelection()
        {
            if (_mostRecent?.MazeId is not null)
            {
                SelectMaze(_mostRecent.MazeId);
            }
            else if (_mostRecent?.Challenge is not null)
            {
                SelectDifficulty(ParseDifficulty(_mostRecent.Challenge));
            }
            else if (_playedMazes.Count > 0)
            {
                SelectMaze(_playedMazes[0].MazeId);
            }
            else
            {
                SelectDifficulty(Difficulty.Easy);
            }
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
                foreach (PlayedMaze maze in _playedMazes)
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

        private static string MazeLabel(string id, Dictionary<string, string> nameById, Dictionary<string, string> nameByBasename)
        {
            if (nameById.TryGetValue(id, out string? name))
                return name;
            string basename = Basename(id);
            if (nameByBasename.TryGetValue(basename, out string? byBasename))
                return byBasename;
            return basename.EndsWith(".json", StringComparison.OrdinalIgnoreCase) ? basename[..^5] : basename;
        }
    }
}
