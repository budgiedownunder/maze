using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.Services;
using Maze.Maui.App.Extensions;

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
        /// Saves the given maze definition
        /// </summary>
        /// <param name="definition">Maze definition</param>
        /// <returns>Task containing a boolean result</returns>
        public async Task<bool> SaveMaze(Api.Maze definition)
        {
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
                    Definition = definition
                };

                await _mazeService.CreateMazeItem(item);
                MazeItem.Name = name;
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
                Definition = definition
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
                isDestructive: true)) {

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
                        UpdateCanSaveRefresh(false);
                        refreshed = true;
                    }
                }
                catch (Exception ex )
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
        /// Updates the `CanSave`/`CanRefresh` property states for the given dirty state
        /// </summary>
        /// <returns>Nothing</returns>
        private void UpdateCanSaveRefresh(bool dirty)
        {
            IsDirty = dirty;
            if (IsStored)
                CanRefresh = IsDirty && !IsBusy;
            CanSave = IsDirty && !IsBusy;
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
