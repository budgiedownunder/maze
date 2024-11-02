using System.ComponentModel;
using System.Runtime.CompilerServices;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    class MainPageViewModel : INotifyPropertyChanged
    {
        private readonly IDeviceTypeService _deviceTypeService;

        public bool IsTouchOnlyDevice
        {
            get => _deviceTypeService.IsTouchOnlyDevice();
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
