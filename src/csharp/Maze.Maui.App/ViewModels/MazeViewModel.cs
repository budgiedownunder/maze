using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.Services;
using Maze.Maui.App.Extensions;
using Microsoft.Maui.Controls;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents a maze view model
    /// </summary>
    [QueryProperty("MazeItem", "MazeItem")]
    public partial class MazeViewModel : BaseViewModel
    {
        // Private definitions
        private const int COMMAND_DELAY_MS = 50;

        // Private properties
        private readonly IDeviceTypeService _deviceTypeService;
        private readonly IMazeService _mazeService;
        private readonly IDialogService _dialogService;

        /// <summary>
        /// Represents an insert rows requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? InsertRowsRequested;
        /// <summary>
        /// Represents a delete rows requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? DeleteRowsRequested;
        /// <summary>
        /// Represents an insert columns requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? InsertColumnsRequested;
        /// <summary>
        /// Represents a delete columns requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? DeleteColumnsRequested;
        /// <summary>
        /// Represents a select range requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SelectRangeRequested;
        /// <summary>
        /// Represents a done requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? DoneRequested;
        /// <summary>
        /// Represents a set wall cell requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SetWallRequested;
        /// <summary>
        /// Represents a set start cell requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SetStartRequested;
        /// <summary>
        /// Represents a set finish cell requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SetFinishRequested;
        /// <summary>
        /// Represents a set key cell(s) requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SetKeyRequested;
        /// <summary>
        /// Represents a set door cell(s) requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SetDoorRequested;
        /// <summary>
        /// Represents a set enemy cell(s) requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SetEnemyRequested;
        /// <summary>
        /// Represents a set health cell(s) requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SetHealthRequested;
        /// <summary>
        /// Represents a set treasure cell(s) requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SetTreasureRequested;
        /// <summary>
        /// Represents a clear cells requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? ClearRequested;
        /// <summary>
        /// Represents a solve maze requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SolveRequested;
        /// <summary>
        /// Represents a clear solution requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? ClearSolutionRequested;
        /// <summary>
        /// Represents a save maze requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? SaveRequested;
        /// <summary>
        /// Represents a refresh maze requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? RefreshRequested;
        /// <summary>
        /// Represents a generate maze requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? GenerateRequested;
        /// <summary>
        /// Represents a walk solution requested event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event EventHandler? WalkSolutionRequested;
        /// <summary>
        /// Indicates whether the view maze is stored
        /// </summary>
        /// <returns>Boolean value</returns>
        public bool IsStored { get; set; }
        /// <summary>
        /// Indicates whether the view maze is dirty (unsaved)
        /// </summary>
        /// <returns>Boolean value</returns>
        public bool IsDirty { get; set; }
        /// <summary>
        /// Indicates whether the maze's 3D game settings have unsaved edits. Tracked
        /// separately from <see cref="IsDirty"/> (the grid/definition dirty flag) so a
        /// settings-only change still enables Save and triggers the save-on-play prompt —
        /// mirroring the React client's separate <c>gameSettingsDirty</c> flag.
        /// </summary>
        /// <returns>Boolean value</returns>
        public bool GameSettingsDirty { get; set; }
        /// <summary>
        /// The maze item currently being displayed
        /// </summary>
        /// <returns>Maze item</returns>
        [ObservableProperty]
        protected MazeItem mazeItem = new MazeItem();
        /// <summary>
        /// Indicates whether rows can currently be inserted
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canInsertRows = false;
        /// <summary>
        /// Indicates whether rows can currently be deleted
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canDeleteRows = false;
        /// <summary>
        /// Indicates whether columns can currently be inserted
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canInsertColumns = false;
        /// <summary>
        /// Indicates whether columns can currently be inserted
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canDeleteColumns = false;
        /// <summary>
        /// Indicates whether the selection can currently switch to extended cell selection mode
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSelectRange = false;
        /// <summary>
        /// Indicates whether the "done" button can be displayed
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canShowDone = false;
        /// <summary>
        /// Indicates whether wall cells can be set within the current selection
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSetWall = false;
        /// <summary>
        /// Indicates whether a start cell can be set within the current selection
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSetStart = false;
        /// <summary>
        /// Indicates whether a finish cell can be set within the current selection
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSetFinish = false;
        /// <summary>
        /// Indicates whether key cells can be set within the current selection
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSetKey = false;
        /// <summary>
        /// Indicates whether door cells can be set within the current selection
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSetDoor = false;
        /// <summary>
        /// Indicates whether enemy cells can be set within the current selection
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSetEnemy = false;
        /// <summary>
        /// Indicates whether health cells can be set within the current selection
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSetHealth = false;
        /// <summary>
        /// Indicates whether treasure cells can be set within the current selection
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSetTreasure = false;
        /// <summary>
        /// Indicates whether the currently selected cells can be cleared
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canClear = false;
        /// <summary>
        /// Indicates whether the maze can be solved
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSolve = false;
        /// <summary>
        /// Indicates whether the maze solution can be cleared
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canClearSolution = false;
        /// <summary>
        /// Indicates whether a maze can be generated
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canGenerate = false;
        /// <summary>
        /// Indicates whether the walk solution can be started
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canWalkSolution = false;
        /// <summary>
        /// Indicates whether a walk solution animation is currently in progress
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool isWalking = false;
        /// <summary>
        /// Indicates whether the maze can be saved
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canSave = true;
        /// <summary>
        /// Indicates whether the maze can be refreshed
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool canRefresh = false;
        /// <summary>
        /// Indicates whether the current device is touch-only
        /// </summary>
        /// <returns>Boolean value</returns>
        public bool IsTouchOnlyDevice
        {
            get => _deviceTypeService.IsTouchOnlyDevice();
        }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="deviceTypeService">Injected device type service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="mazeService">Injected maze service</param>
        public MazeViewModel(IDeviceTypeService deviceTypeService, IDialogService dialogService, IMazeService mazeService)
        {
            this._deviceTypeService = deviceTypeService;
            this._mazeService = mazeService;
            this._dialogService = dialogService;
        }
        /// <summary>
        /// Insert rows command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task InsertRowsAsync()
        {
            await RunRequest(InsertRowsRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Delete rows command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task DeleteRowsAsync()
        {
            await RunRequest(DeleteRowsRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Insert columns command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task InsertColumnsAsync()
        {
            await RunRequest(InsertColumnsRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Delete columns command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task DeleteColumnsAsync()
        {
            await RunRequest(DeleteColumnsRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Enter extended selection mode command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SelectRangeAsync()
        {
            await RunRequest(SelectRangeRequested);
        }
        /// <summary>
        /// Done command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task DoneAsync()
        {
            await RunRequest(DoneRequested);
        }
        /// <summary>
        /// Set wall cell(s) within selection command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SetWallAsync()
        {
            await RunRequest(SetWallRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Set start cell command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SetStartAsync()
        {
            await RunRequest(SetStartRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Set finish cell command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SetFinishAsync()
        {
            await RunRequest(SetFinishRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Set key cell(s) within selection command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SetKeyAsync()
        {
            await RunRequest(SetKeyRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Set door cell(s) within selection command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SetDoorAsync()
        {
            await RunRequest(SetDoorRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Set enemy cell(s) within selection command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SetEnemyAsync()
        {
            await RunRequest(SetEnemyRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Set health cell(s) within selection command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SetHealthAsync()
        {
            await RunRequest(SetHealthRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Set treasure cell(s) within selection command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SetTreasureAsync()
        {
            await RunRequest(SetTreasureRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Clear selected cell content command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task ClearAsync()
        {
            await RunRequest(ClearRequested);
            UpdateCanSaveRefresh(true);
        }
        /// <summary>
        /// Solve maze command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SolveAsync()
        {
            await RunRequest(SolveRequested);
        }
        /// <summary>
        /// Clear maze solution command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task ClearSolutionAsync()
        {
            await RunRequest(ClearSolutionRequested);
        }
        /// <summary>
        /// Save maze command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task SaveAsync()
        {
            await RunRequest(SaveRequested);
        }
        /// <summary>
        /// Refresh maze command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task RefreshAsync()
        {
            await RunRequest(RefreshRequested);
        }
        /// <summary>
        /// Generate maze command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task GenerateAsync()
        {
            await RunRequest(GenerateRequested);
        }
        /// <summary>
        /// Walk solution command
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        private async Task WalkSolutionAsync()
        {
            await RunRequest(WalkSolutionRequested);
        }
        /// <summary>
        /// Saves the given maze definition. Refuses the save up-front when the grid
        /// exceeds a cap: more key + door cells than the key-aware solver can handle
        /// (<see cref="Api.Maze.MaxTotalFeatures"/>), or more enemy / health /
        /// treasure cells than their per-type limits
        /// (<see cref="Api.Maze.MaxEnemyCount"/>, <see cref="Api.Maze.MaxHealthCount"/>,
        /// <see cref="Api.Maze.MaxTreasureCount"/>).
        /// </summary>
        /// <param name="definition">Maze definition</param>
        /// <returns>Task containing a boolean result</returns>
        public async Task<bool> SaveMaze(Api.Maze definition)
        {
            // Keys + doors share one budget: together they drive the key-aware
            // solver's feature mask (Api.Maze.MaxTotalFeatures).
            (uint keys, uint doors) = Utils.MazeCellCounter.CountKeysAndDoors(definition);
            if (keys + doors > Api.Maze.MaxTotalFeatures)
            {
                await _dialogService.ShowAlert(
                    "Cannot save",
                    $"This maze has {keys} keys + {doors} doors = {keys + doors}, " +
                    $"over the limit of {Api.Maze.MaxTotalFeatures}. " +
                    "Remove some key or door cells before saving.",
                    "OK");
                return false;
            }

            // Enemy, health and treasure cells are each capped to the same
            // per-type limit the generator places and the server enforces on save,
            // so a hand-authored maze cannot (for example) stack hundreds of
            // treasure chests and overwhelm the 3D renderer. Refuse over-cap
            // up-front with a clear message rather than letting the save fail with
            // a raw server error.
            foreach ((Api.Maze.CellType type, uint max, string label) in new[]
            {
                (Api.Maze.CellType.Enemy, Api.Maze.MaxEnemyCount, "enemies"),
                (Api.Maze.CellType.Health, Api.Maze.MaxHealthCount, "health pickups"),
                (Api.Maze.CellType.Treasure, Api.Maze.MaxTreasureCount, "treasure items"),
            })
            {
                uint count = Utils.MazeCellCounter.CountCellsOfType(definition, type);
                if (count > max)
                {
                    await _dialogService.ShowAlert(
                        "Cannot save",
                        $"This maze has {count} {label}, over the limit of {max}. " +
                        $"Remove some {label} before saving.",
                        "OK");
                    return false;
                }
            }

            bool saved = false;

            try
            {
                if (IsStored)
                {
                    await UpdateMazeItem(definition);
                    saved = true;
                }
                else
                    saved = await CreateMazeItem(definition);

                if (saved)
                    UpdateCanSaveRefresh(false);
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert("Error", $"Failed to save maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
            }
            return saved;
        }
        /// <summary>
        /// Prompts the user for a maze name and then creates a new maze item with that name and 
        /// the supplied definition
        /// </summary>
        /// <param name="definition">Maze definition</param>
        /// <returns>Task containing a boolean result</returns>
        private async Task<bool> CreateMazeItem(Api.Maze definition)
        {
            bool created = false;
            string? name = await _dialogService.DisplayPrompt("Create Maze", "Name", "Name", "OK", "Cancel", "Enter maze name",
                                                keyboard: Keyboard.Text, allowEmpty: false, trimResult: true);
            if (name is not null)
            {
                MazeItem item = new MazeItem
                {
                    Name = name,
                    Definition = definition,
                    GameSettings = MazeItem.GameSettings
                };

                await _mazeService.CreateMazeItem(item);
                MazeItem.ID = item.ID;
                MazeItem.Name = name;
                IsStored = true;
                WeakReferenceMessenger.Default.Send(new NewMazeItemMessage(item));
                created = true;
            }
            return created;
        }
        /// <summary>
        /// Updates the current maze with the given definition
        /// </summary>
        /// <param name="definition">Maze definition</param>
        /// <returns>Task</returns>
        private async Task UpdateMazeItem(Api.Maze definition)
        {
            MazeItem item = new MazeItem
            {
                ID = MazeItem.ID,
                Name = MazeItem.Name,
                Definition = definition,
                GameSettings = MazeItem.GameSettings
            };
            await _mazeService.UpdateMazeItem(item);
            MazeItem.Definition = definition;
        }
        /// <summary>
        /// Prompts the user for a confirmation and, if confirmed, refreshes the maze definition
        /// the supplied definition
        /// </summary>
        /// <returns>Task containing a boolean result</returns>
        public async Task<bool> RefreshMaze()
        {
            bool refreshed = false;
            if (await _dialogService.ShowConfirmation("Refresh Maze",
                "Are you sure you want to refresh the maze?\n\nNote: any changes you have made will be lost",
                "Yes", "No",
                isDestructive: true))
            {

                if (CanClearSolution)
                    await ClearSolutionAsync();

                try
                {
                    IsBusy = true;
                    MazeItem? item = await _mazeService.GetMazeItem(MazeItem.ID);
                    if (item is not null)
                    {
                        MazeItem.Name = item?.Name ?? "";
                        MazeItem.Definition = item?.Definition ?? new Api.Maze(1, 1);
                        MazeItem.GameSettings = item?.GameSettings;
                        UpdateCanSaveRefresh(false);
                        refreshed = true;
                    }
                }
                catch (Exception ex)
                {
                    await _dialogService.ShowAlert("Error", $"Failed to refresh maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
                }
                finally
                {
                    IsBusy = false;
                }
            }
            return refreshed;
        }
        /// <summary>
        /// Notifies the view model that the maze has been changed (e.g. after generation)
        /// </summary>
        public void NotifyMazeChanged() => UpdateCanSaveRefresh(true);
        /// <summary>
        /// Applies edited 3D game settings to the current maze and marks it dirty,
        /// so the change is persisted on the next Save (the settings ride the maze).
        /// </summary>
        /// <param name="settings">The chosen game settings</param>
        public void ApplyGameSettings(MazeGameSettings settings)
        {
            MazeItem.GameSettings = settings;
            GameSettingsDirty = true;
            RefreshSaveState();
        }
        /// <summary>
        /// Updates the grid/definition dirty state and recomputes the `CanSave`/`CanRefresh`
        /// states. A save/refresh (<paramref name="dirty"/> == false) persists or reloads the
        /// whole maze — including its game settings — so it also clears the separate
        /// game-settings dirty flag.
        /// </summary>
        /// <returns>Nothing</returns>
        private void UpdateCanSaveRefresh(bool dirty)
        {
            IsDirty = dirty;
            if (!dirty)
                GameSettingsDirty = false;
            RefreshSaveState();
        }
        /// <summary>
        /// Recomputes the `CanSave`/`CanRefresh` states from the current dirty flags. Save is
        /// enabled when either the grid/definition or the game settings have unsaved edits
        /// (and the view model is not busy).
        /// </summary>
        /// <returns>Nothing</returns>
        private void RefreshSaveState()
        {
            bool hasUnsavedWork = (IsDirty || GameSettingsDirty) && !IsBusy;
            if (IsStored)
                CanRefresh = hasUnsavedWork;
            CanSave = hasUnsavedWork;
        }
        /// <summary>
        /// Reacts to <see cref="BaseViewModel.IsBusy"/> changes. Because save-state is gated on
        /// <c>!IsBusy</c>, a transient busy blip would otherwise strand <c>CanSave</c> at the
        /// value computed while busy. (The game-settings popup closes via a Shell navigation,
        /// which fires <c>MazePage.OnNavigatedTo</c> and flips <c>IsBusy</c> on for ~300ms; a
        /// settings edit applied inside that window would not enable Save.) Recomputing the
        /// save-state whenever <c>IsBusy</c> changes keeps it consistent once busy clears.
        /// </summary>
        /// <param name="e">The changed-property arguments</param>
        protected override void OnPropertyChanged(System.ComponentModel.PropertyChangedEventArgs e)
        {
            base.OnPropertyChanged(e);
            if (e.PropertyName == nameof(IsBusy))
                RefreshSaveState();
        }
        /// <summary>
        /// Runs the given event handler request
        /// </summary>
        /// <returns>Task</returns>
        private async Task RunRequest(EventHandler? eventHandler)
        {
            try
            {
                IsBusy = true;
                await Task.Delay(COMMAND_DELAY_MS);
                eventHandler?.Invoke(this, EventArgs.Empty);
            }
            finally
            {
                IsBusy = false;
            }
        }
    }
}
