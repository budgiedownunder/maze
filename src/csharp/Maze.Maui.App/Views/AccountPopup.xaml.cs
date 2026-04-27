using CommunityToolkit.Maui.Views;
using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// A popup that displays the user's account details and allows profile editing,
    /// password change, and account deletion.
    /// This is how the popup appears on Windows Desktop:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Windows</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/windows-account.png" height="500" width="500"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    ///  
    /// and this is how it appears on Android/iOS devices:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Android</strong></th>
    ///         <th><strong>iOS</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/android-account.png" width="250"/></td>
    ///         <td><img src="../../images/screenshots/ios-account.png" width="250"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    /// </summary>
    public partial class AccountPopup : Popup
    {
        private readonly AccountViewModel _viewModel;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="viewModel">The account view model</param>
        public AccountPopup(AccountViewModel viewModel)
        {
            _viewModel = viewModel;
            BindingContext = viewModel;
            InitializeComponent();
            double screenWidth = DeviceDisplay.Current.MainDisplayInfo.Width
                / DeviceDisplay.Current.MainDisplayInfo.Density;
            WidthRequest = Math.Min(screenWidth * 0.85, 400);
            viewModel.LoadProfileCommand.Execute(null);

            // Clear the welcome-banner flag once the popup is closed (by either
            // path — Close button or tap-outside — both fire Closed). Subsequent
            // burger-menu opens of the Account UI then render without the
            // banner. Keeping the flag set during the popup's lifetime is what
            // makes the banner visible in the first place.
            Closed += OnPopupClosed;
        }

        private void OnPopupClosed(object? sender, EventArgs e)
        {
            _viewModel.IsWelcomeMode = false;
            Closed -= OnPopupClosed;
        }

        /// <summary>
        /// Handles the Close button click.
        /// </summary>
        private async void OnCloseClicked(object sender, EventArgs e)
        {
            await CloseAsync();
        }
    }
}
