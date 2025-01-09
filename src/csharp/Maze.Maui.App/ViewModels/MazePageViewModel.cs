using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Windows.Input;
using CommunityToolkit.Mvvm.ComponentModel;
using Maze.Maui.App.Models;
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

        private readonly IDeviceTypeService _deviceTypeService;
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

        public bool IsTouchOnlyDevice
        {
            get => _deviceTypeService.IsTouchOnlyDevice();
        }

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

        public MazePageViewModel(IDeviceTypeService deviceTypeService)
        {
            _deviceTypeService = deviceTypeService;
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

        }
        [ObservableProperty]
        MazeItem mazeItem = new MazeItem();

        private async Task OnInsertRows()
        {
            await RunCommand(InsertRowsRequested);
        }

        private async Task OnDeleteRows()
        {
            await RunCommand(DeleteRowsRequested);
        }

        private async Task OnInsertColumns()
        {
            await RunCommand(InsertColumnsRequested);
        }

        private async Task OnDeleteColumns()
        {
            await RunCommand(DeleteColumnsRequested);
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
        }

        private async Task OnSetStart()
        {
            await RunCommand(SetStartRequested);
        }

        private async Task OnSetFinish()
        {
            await RunCommand(SetFinishRequested);
        }

        private async Task OnClear()
        {
            await RunCommand(ClearRequested);
        }

        private async Task OnSolve()
        {
            await RunCommand(SolveRequested);
        }

        private async Task OnClearSolution()
        {
            await RunCommand(ClearSolutionRequested);
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
