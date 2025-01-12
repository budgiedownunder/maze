using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Windows.Input;
using CommunityToolkit.Mvvm.ComponentModel;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.Services;

namespace Maze.Maui.App.ViewModels
{   
    [QueryProperty("MazeItem", "MazeItem")]
    public partial class MazePageViewModel : BaseViewModel
    {
        private const int COMMAND_DELAY_MS = 50;

        public ICommand InsertRowsCommand { get; }
        public ICommand DeleteRowsCommand { get; }
        public ICommand InsertColumnsCommand { get; }
        public ICommand DeleteColumnsCommand { get; }
        public ICommand SelectRangeCommand { get; }
        public ICommand DoneCommand { get; }
        public ICommand SetWallCommand { get; }
        public ICommand SetStartCommand { get; }
        public ICommand SetFinishCommand { get; }
        public ICommand ClearCommand { get; }
        public ICommand SolveCommand { get; }
        public ICommand ClearSolutionCommand { get; }
        public ICommand SaveCommand { get; }
        public ICommand RefreshCommand { get; }

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

        private readonly IDeviceTypeService _deviceTypeService;
        private readonly IMazeService _mazeService;

        private bool _canInsertRows = false;
        private bool _canDeleteRows = false;
        private bool _canInsertColumns = false;
        private bool _canDeleteColumns = false;
        private bool _canSelectRange = false;
        private bool _canShowDone = false;
        private bool _canSetWall = false;
        private bool _canSetStart = false;
        private bool _canSetFinish = false;
        private bool _canClear = false;
        private bool _canSolve = false;
        private bool _canClearSolution = false;
        private bool _canSave = true;
        private bool _canRefresh = false;

        public bool IsTouchOnlyDevice
        {
            get => _deviceTypeService.IsTouchOnlyDevice();
        }

        public bool IsStored { get; set; }
        public bool IsDirty { get; set; }

        public bool CanInsertRows
        {
            get => _canInsertRows;
            set
            {
                if (_canInsertRows != value)
                {
                    _canInsertRows = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanDeleteRows
        {
            get => _canDeleteRows;
            set
            {
                if (_canDeleteRows != value)
                {
                    _canDeleteRows = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanInsertColumns
        {
            get => _canInsertColumns;
            set
            {
                if (_canInsertColumns != value)
                {
                    _canInsertColumns = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanDeleteColumns
        {
            get => _canDeleteColumns;
            set
            {
                if (_canDeleteColumns != value)
                {
                    _canDeleteColumns = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanSelectRange
        {
            get => _canSelectRange;
            set
            {
                if (_canSelectRange != value)
                {
                    _canSelectRange = value;
                    OnPropertyChanged();
                }
            }
        }


        public bool CanShowDone
        {
            get => _canShowDone;
            set
            {
                if (_canShowDone!= value)
                {
                    _canShowDone = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanSetWall
        {
            get => _canSetWall;
            set
            {
                if (_canSetWall != value)
                {
                    _canSetWall = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanSetStart
        {
            get => _canSetStart;
            set
            {
                if (_canSetStart != value)
                {
                    _canSetStart = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanSetFinish
        {
            get => _canSetFinish;
            set
            {
                if (_canSetFinish != value)
                {
                    _canSetFinish = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanClear
        {
            get => _canClear;
            set
            {
                if (_canClear != value)
                {
                    _canClear = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanSolve
        {
            get => _canSolve;
            set
            {
                if (_canSolve != value)
                {
                    _canSolve = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanClearSolution
        {
            get => _canClearSolution;
            set
            {
                if (_canClearSolution != value)
                {
                    _canClearSolution = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanSave
        {
            get => _canSave;
            set
            {
                if (_canSave != value)
                {
                    _canSave= value;
                    OnPropertyChanged();
                }
            }
        }

        public bool CanRefresh
        {
            get => _canRefresh;
            set
            {
                if (_canRefresh != value)
                {
                    _canRefresh = value;
                    OnPropertyChanged();
                }
            }
        }

        public MazePageViewModel(IDeviceTypeService deviceTypeService, IMazeService mazeService)
        {
            this._deviceTypeService = deviceTypeService;
            this._mazeService = mazeService;

            InsertRowsCommand = new Command(async () => await OnInsertRows());
            DeleteRowsCommand = new Command(async () => await OnDeleteRows());
            InsertColumnsCommand = new Command(async () => await OnInsertColumns());
            DeleteColumnsCommand = new Command(async () => await OnDeleteColumns());
            SelectRangeCommand = new Command(async () => await OnSelectRange());
            DoneCommand = new Command(async () => await OnDone());
            SetWallCommand = new Command(async () => await OnSetWall());
            SetStartCommand = new Command(async () => await OnSetStart());
            SetFinishCommand = new Command(async () => await OnSetFinish());
            ClearCommand = new Command(async () => await OnClear());
            SolveCommand = new Command(async () => await OnSolve());
            ClearSolutionCommand = new Command(async () => await OnClearSolution());
            SaveCommand = new Command(async () => await OnSave());
            RefreshCommand = new Command(async () => await OnRefresh());
        }
        [ObservableProperty]
        MazeItem mazeItem = new MazeItem();

        private async Task OnInsertRows()
        {
            await RunCommand(InsertRowsRequested);
            UpdateCanSaveRefresh(true);
        }

        private async Task OnDeleteRows()
        {
            await RunCommand(DeleteRowsRequested);
            UpdateCanSaveRefresh(true);
        }

        private async Task OnInsertColumns()
        {
            await RunCommand(InsertColumnsRequested);
            UpdateCanSaveRefresh(true);
        }

        private async Task OnDeleteColumns()
        {
            await RunCommand(DeleteColumnsRequested);
            UpdateCanSaveRefresh(true);
        }

        private async Task OnSelectRange()
        {
            await RunCommand(SelectRangeRequested);
        }

        private async Task OnDone()
        {
            await RunCommand(DoneRequested);
        }

        private async Task OnSetWall()
        {
            await RunCommand(SetWallRequested);
            UpdateCanSaveRefresh(true);
        }

        private async Task OnSetStart()
        {
            await RunCommand(SetStartRequested);
            UpdateCanSaveRefresh(true);
        }

        private async Task OnSetFinish()
        {
            await RunCommand(SetFinishRequested);
            UpdateCanSaveRefresh(true);
        }

        private async Task OnClear()
        {
            await RunCommand(ClearRequested);
            UpdateCanSaveRefresh(true);
        }

        private async Task OnSolve()
        {
            await RunCommand(SolveRequested);
        }

        private async Task OnClearSolution()
        {
            await RunCommand(ClearSolutionRequested);
        }

        private async Task OnSave()
        {
            await RunCommand(SaveRequested);
        }

        private async Task CreateMazeItem(Api.Maze definition)
        {
            throw new Exception("CreateMazeItem not implemented");
        }

        private async Task UpdateMazeItem(Api.Maze definition)
        {
            MazeItem mazeItem = new MazeItem
            {
                ID = MazeItem.ID,
                Name = MazeItem.Name,
                Definition = definition
            };
            await _mazeService.UpdateMazeItem(mazeItem);
            MazeItem.Definition = definition;
        }

        public async Task<bool> SaveMaze(Api.Maze definition)
        {
            bool saved = false;

            try
            {
                if (IsStored)
                    await UpdateMazeItem(definition);
                else
                    await CreateMazeItem(definition);
                UpdateCanSaveRefresh(false);
                saved = true;
            }
            catch (Exception ex)
            {
                await ShowAlert("Error", $"Unable to save maze: {ex.Message}", "OK");
            }
            finally
            {
            }
            return saved;
        }

        private async Task OnRefresh()
        {
            await ShowAlert("Action", "Would attempt to refresh maze", "OK");
        }

        private void UpdateCanSaveRefresh(bool dirty)
        {
            IsDirty = dirty;
            if (IsStored)
                CanRefresh = IsDirty;
            CanSave = IsDirty;
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

        // TO DO - move these to a dialog service
        private async Task ShowAlert(string title, string message, string cancel)
        {
            await Shell.Current.DisplayAlert(title, message, cancel);
        }

    }
}
