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

        /// <summary>The Email Addresses panel ViewModel — exposed as a
        /// public property so the XAML can reach it via
        /// <c>{Binding Source={x:Reference ...}}</c> without affecting the
        /// outer popup's BindingContext (still the AccountViewModel).</summary>
        public EmailAddressesViewModel EmailsViewModel { get; }

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="viewModel">The account view model</param>
        /// <param name="emailsViewModel">The email-addresses view model
        ///   (transient — fresh instance per popup-open).</param>
        public AccountPopup(AccountViewModel viewModel, EmailAddressesViewModel emailsViewModel)
        {
            _viewModel = viewModel;
            EmailsViewModel = emailsViewModel;
            BindingContext = viewModel;
            InitializeComponent();
            // Constrain width to a fraction of the screen, and bound the
            // INNER ScrollView's height directly. Setting HeightRequest on
            // the popup itself only engages outer scrolling on Windows;
            // iOS and Android leave the popup-level constraint
            // unpropagated and the outer ScrollView never bounded.
            // Setting HeightRequest on the ScrollView element itself works
            // on every platform, so the Close / Delete / Change Password
            // buttons stay reachable when the email list is long.
            double density = DeviceDisplay.Current.MainDisplayInfo.Density;
            double screenWidth = DeviceDisplay.Current.MainDisplayInfo.Width / density;
            double screenHeight = DeviceDisplay.Current.MainDisplayInfo.Height / density;
            double popupWidth = Math.Min(screenWidth * 0.85, 400);
            double popupHeight = Math.Min(screenHeight * 0.85, 700);
            WidthRequest = popupWidth;
            // Outer ScrollView capped so its content is always scrollable
            // within a known frame. Popup.HeightRequest doesn't propagate
            // through the iOS UIViewController presentation; setting it on
            // the ScrollView (and on the surrounding Border, below) bounds
            // the actual visible frame so scroll engages on every platform.
            OuterScroll.HeightRequest = Math.Min(screenHeight * 0.8, 660);
            PopupBorder.HeightRequest = popupHeight;
            viewModel.LoadProfileCommand.Execute(null);
            emailsViewModel.LoadEmailsCommand.Execute(null);

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
