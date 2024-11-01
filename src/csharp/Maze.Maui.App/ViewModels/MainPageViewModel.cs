using System.ComponentModel;
using System.Runtime.CompilerServices;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    class MainPageViewModel : INotifyPropertyChanged
    {
        private bool showTopRowLayout = false;
        private bool showSelectRangeBtn = true;
        private bool showCancelBtn = false;
        private readonly IDeviceTypeService _deviceTypeService;

        public bool IsTouchOnlyDevice
        {
            get => _deviceTypeService.IsTouchOnlyDevice();
        }

        public bool ShowTopRowLayout
        {
            get => showTopRowLayout;
            set
            {
                if (showTopRowLayout != value)
                {
                    showTopRowLayout = value;
                    OnPropertyChanged();
                }
            }
        }
        

        public bool ShowSelectRangeBtn
        {
            get => showSelectRangeBtn;
            set
            {
                if (showSelectRangeBtn != value)
                {
                    showSelectRangeBtn = value;
                    OnPropertyChanged();
                }
            }
        }

        public bool ShowCancelBtn
        {
            get => showCancelBtn;
            set
            {
                if (showCancelBtn != value)
                {
                    showCancelBtn = value;
                    OnPropertyChanged();
                }
            }
        }

        public MainPageViewModel(IDeviceTypeService deviceTypeService)
        {
            _deviceTypeService = deviceTypeService;
        }

        public event PropertyChangedEventHandler? PropertyChanged;
        protected void OnPropertyChanged([CallerMemberName] string propertyName = "")
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

    }
}
