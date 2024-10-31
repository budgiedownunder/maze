using System.ComponentModel;
using System.Runtime.CompilerServices;
using Microsoft.Maui.Devices;

namespace Maze.Maui.App.ViewModels
{
    class MainPageViewModel : INotifyPropertyChanged
    {
        private bool showSelectRangeBtn = true;
        private bool showCancelBtn = false;
        
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
