using CommunityToolkit.Mvvm.ComponentModel;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// One row in the Email Addresses panel — an address plus its primary
    /// and verification status. Per-row enabled/disabled state for actions
    /// is computed by the parent <see cref="EmailAddressesViewModel"/>.
    /// </summary>
    public partial class EmailRowViewModel : ObservableObject
    {
        [ObservableProperty]
        private string email = "";

        [ObservableProperty]
        private bool isPrimary;

        [ObservableProperty]
        private bool verified;
    }
}
