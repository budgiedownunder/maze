using System.ComponentModel;
using System.Runtime.CompilerServices;
using Microsoft.Maui.Devices;

namespace Maze.Maui.App.ViewModels
{
    class MainPageViewModel : INotifyPropertyChanged
    {
        private bool showEnableRangeSelect = false;
        public bool ShowEnableRangeSelect
        {
            get => showEnableRangeSelect;
            set
            {
                if (showEnableRangeSelect != value)
                {
                    showEnableRangeSelect = value;
                    OnPropertyChanged();
                }
            }
        }

        public MainPageViewModel()
        {
        }

        public event PropertyChangedEventHandler? PropertyChanged;
        protected void OnPropertyChanged([CallerMemberName] string propertyName = "")
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
