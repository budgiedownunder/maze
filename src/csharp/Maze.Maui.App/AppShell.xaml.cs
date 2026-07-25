using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Maze.Maui.App.Views;
using Maze.Maui.Controls.Pointer;

namespace Maze.Maui.App
{
    /// <summary>
    /// MAUI application shell class
    /// </summary>
    public partial class AppShell : Shell
    {
        private readonly IAuthService _authService;
        private readonly IDialogService _dialogService;
        private readonly AccountViewModel _accountViewModel;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="accountViewModel">Injected account view model</param>
        public AppShell(IAuthService authService, IDialogService dialogService, AccountViewModel accountViewModel)
        {
            _authService = authService;
            _dialogService = dialogService;
            _accountViewModel = accountViewModel;
            InitializeComponent();
            // The flyout header binds to the account view model's Username.
            FlyoutHeaderRoot.BindingContext = _accountViewModel;
            // Apply the header's accent background once the App-level colour
            // resources are available (they are not yet merged at Shell-parse
            // time — see OnShellLoaded).
            Loaded += OnShellLoaded;
            Routing.RegisterRoute(nameof(MazePage), typeof(MazePage));
            Routing.RegisterRoute(nameof(MazeGamePage), typeof(MazeGamePage));
            Routing.RegisterRoute(nameof(Play3dGamePage), typeof(Play3dGamePage));
            Routing.RegisterRoute(nameof(SignUpPage), typeof(SignUpPage));
            Routing.RegisterRoute(nameof(ChangePasswordPage), typeof(ChangePasswordPage));
            Routing.RegisterRoute(nameof(ForgotPasswordPage), typeof(ForgotPasswordPage));
            Routing.RegisterRoute(nameof(AccountPage), typeof(AccountPage));
            Routing.RegisterRoute(nameof(MazesPage), typeof(MazesPage));
            Routing.RegisterRoute(nameof(LeaderboardsPage), typeof(LeaderboardsPage));
            Routing.RegisterRoute(nameof(Play3dHubPage), typeof(Play3dHubPage));
            Routing.RegisterRoute(nameof(Play3dFeaturedPage), typeof(Play3dFeaturedPage));
            Routing.RegisterRoute(nameof(Play3dMyGamesPage), typeof(Play3dMyGamesPage));
            Routing.RegisterRoute(nameof(Play3dSharedPage), typeof(Play3dSharedPage));
            Routing.RegisterRoute(nameof(Play3dCommunityPage), typeof(Play3dCommunityPage));
        }

        /// <summary>
        /// Applies the flyout header's pale-blue background once the Shell is
        /// loaded, by which point the App-level <c>Colors.xaml</c> resources
        /// (<c>PrimaryButton</c>) have been merged. Referencing them via
        /// <c>StaticResource</c> in the Shell XAML fails at parse time because
        /// the Shell is constructed before <c>App.InitializeComponent</c>
        /// merges them. The same pale-blue fill is used in both themes (its
        /// navy text is set in XAML); mirrors the primary-button styling.
        /// </summary>
        private void OnShellLoaded(object? sender, EventArgs e)
        {
            if (Application.Current?.Resources is { } resources
                && resources.TryGetValue("PrimaryButton", out var fill) && fill is Color fillColor)
            {
                FlyoutHeaderRoot.SetAppThemeColor(VisualElement.BackgroundColorProperty, fillColor, fillColor);
            }
        }

        /// <summary>
        /// When the flyout is opened, ensure the header shows the signed-in
        /// user's name — loaded once per session (the sign-out flow clears it
        /// via <see cref="AccountViewModel.ClearProfile"/>, so a fresh sign-in
        /// re-loads it on the next flyout open).
        /// </summary>
        protected override void OnPropertyChanged(string? propertyName = null)
        {
            base.OnPropertyChanged(propertyName);
            if (propertyName == nameof(FlyoutIsPresented) && FlyoutIsPresented)
            {
                _ = EnsureFlyoutUsernameAsync();
            }
        }

        private async Task EnsureFlyoutUsernameAsync()
        {
            if (!string.IsNullOrEmpty(_accountViewModel.Username))
                return;
            if (!await _authService.IsAuthenticatedAsync())
                return;
            if (_accountViewModel.LoadProfileCommand.CanExecute(null))
                await _accountViewModel.LoadProfileCommand.ExecuteAsync(null);
        }

        /// <summary>
        /// Opens the Account page when the flyout header (the username) is tapped.
        /// </summary>
        private async void OnFlyoutHeaderTapped(object sender, TappedEventArgs e)
        {
            FlyoutIsPresented = false;
            await GoToAsync(nameof(AccountPage));
        }

        /// <summary>
        /// Navigates to the Home page (the root authenticated page).
        /// </summary>
        private async void OnHomeMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;
            await GoToAsync("//MainPage");
        }

        /// <summary>
        /// Opens the 3D Games browser hub — the same entry point as the 3D Games
        /// tile on the Home page. The hub's own tiles reach the individual scopes
        /// (Featured, …); the flyout has no sub-menus, so it lists only the hub.
        /// </summary>
        private async void OnGames3dMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;
            await GoToAsync(nameof(Play3dHubPage));
        }

        /// <summary>
        /// Navigates to the maze list.
        /// </summary>
        private async void OnMazesMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;
            await GoToAsync(nameof(MazesPage));
        }

        /// <summary>
        /// Navigates to the Leaderboards page.
        /// </summary>
        private async void OnLeaderboardsMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;
            await GoToAsync(nameof(LeaderboardsPage));
        }

        /// <summary>
        /// Navigates to the Account page.
        /// </summary>
        private async void OnAccountMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;
            await GoToAsync(nameof(AccountPage));
        }

        /// <summary>
        /// Signs the user out, showing a wait cursor for the duration.
        /// Prompts to save unsaved maze changes before proceeding.
        /// </summary>
        private async void OnSignOutMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;

            if (CurrentPage is MazePage mazePage && mazePage.IsDirty)
            {
                bool? choice = await _dialogService.ShowConfirmation(
                    "Unsaved Changes",
                    "Do you want to save your changes before signing out?",
                    "Save",
                    "Discard",
                    "Cancel");

                if (choice == true)
                {
                    bool saved = await mazePage.TrySaveAsync();
                    if (!saved)
                        return;
                }
                else if (choice == null)
                {
                    return;
                }
            }

            var page = CurrentPage;
            Pointer.SetCursor(page, Icon.Wait);
            try
            {
                await _authService.SignOutAsync();
                WeakReferenceMessenger.Default.Send(new MazesInvalidatedMessage());
                _accountViewModel.ClearProfile();
                await GoToAsync("//LoginPage");
            }
            finally
            {
                Pointer.SetCursor(page, Icon.Arrow);
            }
        }

        /// <summary>
        /// Opens the About popup.
        /// </summary>
        private async void OnAboutMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;
            await CurrentPage.ShowPopupAsync(new AboutPopup());
        }
    }
}
