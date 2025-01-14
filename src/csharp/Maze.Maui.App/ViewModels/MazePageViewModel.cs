using System.Windows.Input;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.Services;

namespace Maze.Maui.App.ViewModels
{   
    [QueryProperty("MazeItem", "MazeItem")]
    public partial class MazePageViewModel : BaseViewModel
    {
        private const int COMMAND_DELAY_MS = 50;

        public event EventHandler? InsertRowsRequested;
        public event EventHandler? DeleteRowsRequested;
        public event EventHandler? InsertColumnsRequested;
        public event EventHandler? DeleteColumnsRequested;
        public event EventHandler? SelectRangeRequested;
        public event EventHandler? DoneRequested;
        public event EventHandler? SetWallRequested;
        public event EventHandler? SetStartRequested;
        public event EventHandler? SetFinishRequested;
        public event EventHandler? ClearRequested;
        public event EventHandler? SolveRequested;
        public event EventHandler? ClearSolutionRequested;
        public event EventHandler? SaveRequested;
        public event EventHandler? RefreshRequested;

        private readonly IDeviceTypeService _deviceTypeService;
        private readonly IMazeService _mazeService;
        private readonly IDialogService _dialogService;

        private readonly MazesViewModel _mazesViewModel;

        public bool IsStored { get; set; }
        public bool IsDirty { get; set; }

        [ObservableProperty]
        MazeItem mazeItem = new MazeItem();

        [ObservableProperty]
        bool canInsertRows = false;

        [ObservableProperty]
        bool canDeleteRows = false;

        [ObservableProperty]
        bool canInsertColumns = false;

        [ObservableProperty]
        bool canDeleteColumns = false;

        [ObservableProperty]
        bool canSelectRange = false;

        [ObservableProperty]
        bool canShowDone = false;

        [ObservableProperty]
        bool canSetWall = false;

        [ObservableProperty]
        bool canSetStart = false;

        [ObservableProperty]
        bool canSetFinish = false;

        [ObservableProperty]
        bool canClear = false;

        [ObservableProperty]
        bool canSolve = false;

        [ObservableProperty]
        bool canClearSolution = false;

        [ObservableProperty]
        bool canSave = true;

        [ObservableProperty]
        bool canRefresh = false;

        public bool IsTouchOnlyDevice
        {
            get => _deviceTypeService.IsTouchOnlyDevice();
        }

        public MazePageViewModel(IDeviceTypeService deviceTypeService, IMazeService mazeService, MazesViewModel mazesViewModel, IDialogService dialogService)
        {
            this._deviceTypeService = deviceTypeService;
            this._mazeService = mazeService;
            this._mazesViewModel = mazesViewModel;
            this._dialogService = dialogService;
        }

        [RelayCommandAttribute]
        private async Task InsertRowsAsync()
        {
            await RunCommand(InsertRowsRequested);
            UpdateCanSaveRefresh(true);
        }

        [RelayCommandAttribute]
        private async Task DeleteRowsAsync()
        {
            await RunCommand(DeleteRowsRequested);
            UpdateCanSaveRefresh(true);
        }

        [RelayCommandAttribute]
        private async Task InsertColumnsAsync()
        {
            await RunCommand(InsertColumnsRequested);
            UpdateCanSaveRefresh(true);
        }

        [RelayCommandAttribute]
        private async Task DeleteColumnsAsync()
        {
            await RunCommand(DeleteColumnsRequested);
            UpdateCanSaveRefresh(true);
        }

        [RelayCommandAttribute]
        private async Task SelectRangeAsync()
        {
            await RunCommand(SelectRangeRequested);
        }

        [RelayCommandAttribute]
        private async Task DoneAsync()
        {
            await RunCommand(DoneRequested);
        }

        [RelayCommandAttribute]
        private async Task SetWallAsync()
        {
            await RunCommand(SetWallRequested);
            UpdateCanSaveRefresh(true);
        }

        [RelayCommandAttribute]
        private async Task SetStartAsync()
        {
            await RunCommand(SetStartRequested);
            UpdateCanSaveRefresh(true);
        }

        [RelayCommandAttribute]
        private async Task SetFinishAsync()
        {
            await RunCommand(SetFinishRequested);
            UpdateCanSaveRefresh(true);
        }

        [RelayCommandAttribute]
        private async Task ClearAsync()
        {
            await RunCommand(ClearRequested);
            UpdateCanSaveRefresh(true);
        }

        [RelayCommandAttribute]
        private async Task SolveAsync()
        {
            await RunCommand(SolveRequested);
        }

        [RelayCommandAttribute]
        private async Task ClearSolutionAsync()
        {
            await RunCommand(ClearSolutionRequested);
        }

        [RelayCommandAttribute]
        private async Task SaveAsync()
        {
            await RunCommand(SaveRequested);
        }

        [RelayCommandAttribute]
        private async Task RefreshAsync()
        {
            await RunCommand(RefreshRequested);
        }

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
                await _dialogService.ShowAlert("Error", $"Failed to save maze: {ex.Message}", "OK");
            }
            return saved;
        }

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
                _mazesViewModel.AddNewItem(item);
                created = true;
            }
            return created;
        }

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

        public async Task<bool> RefreshMaze()
        {
            bool refreshed = false;
            if (await _dialogService.ShowConfirmation("Refresh Maze", 
                "Are you sure you want to refresh the maze?\n\nNote: any changes you have made will be lost", 
                "Yes", "No")) {

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
                    await _dialogService.ShowAlert("Error", $"Failed to refresh maze: {ex.Message}", "OK");
                }
                finally
                {
                    IsBusy = false;
                }
            }
            return refreshed;
        }

        private void UpdateCanSaveRefresh(bool dirty)
        {
            IsDirty = dirty;
            if (IsStored)
                CanRefresh = IsDirty && !IsBusy;
            CanSave = IsDirty && !IsBusy;
        }

        private async Task RunCommand(EventHandler? eventHandler)
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
