
using CommunityToolkit.Mvvm.ComponentModel;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents a base view model
    /// </summary>
    public partial class BaseViewModel : ObservableObject
    {
        /// <summary>
        /// Constructor
        /// </summary>
        public BaseViewModel()
        {
        }
        /// <summary>
        /// Indicates whether the view model is busy
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsNotBusy))]
        protected bool isBusy;
        /// <summary>
        /// Represent the title associated with the view model
        /// </summary>
        /// <returns>Title</returns>
        [ObservableProperty]
        protected string title = "";
        /// <summary>
        /// Indicates whether the view model is not busy
        /// </summary>
        /// <returns>Boolean value</returns>
        public bool IsNotBusy => !IsBusy;
    }
}
