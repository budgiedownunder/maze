using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// A page that displays the user's account details and allows profile editing,
    /// password change, and account deletion.
    /// This is how the page appears on Windows Desktop:
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
    public partial class AccountPage : ContentPage
    {
        private readonly AccountViewModel _viewModel;

        /// <summary>The Email Addresses panel ViewModel — exposed as a
        /// public property so the XAML can reach it via
        /// <c>{Binding Source={x:Reference ...}}</c> without affecting the
        /// outer page's BindingContext (still the AccountViewModel).</summary>
        public EmailAddressesViewModel EmailsViewModel { get; }

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="viewModel">The account view model</param>
        /// <param name="emailsViewModel">The email-addresses view model
        ///   (transient — fresh instance per page navigation).</param>
        public AccountPage(AccountViewModel viewModel, EmailAddressesViewModel emailsViewModel)
        {
            _viewModel = viewModel;
            EmailsViewModel = emailsViewModel;
            BindingContext = viewModel;
            InitializeComponent();
            viewModel.LoadProfileCommand.Execute(null);
            emailsViewModel.LoadEmailsCommand.Execute(null);
        }

        /// <summary>
        /// Clears the welcome-banner flag once the page is left (by back
        /// navigation or sign-out). Subsequent burger-menu opens of the
        /// Account page then render without the banner. Keeping the flag
        /// set during the page's lifetime is what makes the banner visible
        /// in the first place.
        /// </summary>
        protected override void OnDisappearing()
        {
            base.OnDisappearing();
            _viewModel.IsWelcomeMode = false;
        }
    }
}
