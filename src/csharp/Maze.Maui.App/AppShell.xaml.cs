using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;
using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Maze.Maui.App.Views;
using Maze.Maui.Controls.Pointer;
using Microsoft.Extensions.DependencyInjection;

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
        private readonly IServiceProvider _serviceProvider;
        // One-shot guard: Shell fires OnNavigated more than once during a
        // single GoToAsync (typically once for navigation start, once for
        // finalisation), so without this we'd queue two welcome popups —
        // closing the first then finds a second one waiting.
        private bool _welcomePopupPending;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="accountViewModel">Injected account view model</param>
        /// <param name="serviceProvider">DI service provider — used to resolve a fresh
        ///   <see cref="EmailAddressesViewModel"/> per popup-open (transient lifetime).</param>
        public AppShell(IAuthService authService, IDialogService dialogService, AccountViewModel accountViewModel, IServiceProvider serviceProvider)
        {
            _authService = authService;
            _dialogService = dialogService;
            _accountViewModel = accountViewModel;
            _serviceProvider = serviceProvider;
            InitializeComponent();
            Routing.RegisterRoute(nameof(MazePage), typeof(MazePage));
            Routing.RegisterRoute(nameof(MazeGamePage), typeof(MazeGamePage));
            Routing.RegisterRoute(nameof(Play3dGamePage), typeof(Play3dGamePage));
            Routing.RegisterRoute(nameof(SignUpPage), typeof(SignUpPage));
            Routing.RegisterRoute(nameof(ChangePasswordPage), typeof(ChangePasswordPage));
        }

        /// <summary>
        /// Opens the Account popup.
        /// </summary>
        private async void OnAccountMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;
            // Resolve a fresh EmailAddressesViewModel per popup-open so the
            // email list starts from server-authoritative state each time
            // (transient lifetime, see MauiProgram).
            var emailsViewModel = _serviceProvider.GetRequiredService<EmailAddressesViewModel>();
            await CurrentPage.ShowPopupAsync(new AccountPopup(_accountViewModel, emailsViewModel));
        }

        /// <inheritdoc/>
        protected override void OnNavigated(ShellNavigatedEventArgs args)
        {
            base.OnNavigated(args);
            // Auto-open the Account popup with a welcome banner when arriving
            // at the main page on the first sign-in of a brand-new OAuth user.
            // The flag was set by LoginViewModel / SignUpViewModel before the
            // GoToAsync call. AccountPopup itself clears the welcome flag on
            // dismiss so the banner is visible while open and absent on
            // subsequent burger-menu opens.
            if (_accountViewModel.IsWelcomeMode && !_welcomePopupPending && CurrentPage is MazesPage page)
            {
                _welcomePopupPending = true;
                var emailsViewModel = _serviceProvider.GetRequiredService<EmailAddressesViewModel>();
                var popup = new AccountPopup(_accountViewModel, emailsViewModel);
                // Reset the guard once the popup is gone so a future
                // sign-out / sign-in-as-another-new-user re-triggers cleanly.
                popup.Closed += (_, _) => _welcomePopupPending = false;
                // Dispatch onto the UI thread so the popup show happens after
                // the page-arrival event has fully settled — avoids construction
                // races on some platforms.
                Dispatcher.Dispatch(() => _ = page.ShowPopupAsync(popup));
            }
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

        /// <summary>
        /// Navigates to the 3D Bevy game page.
        /// </summary>
        private async void On3dDemoMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;
            await GoToAsync(nameof(Play3dGamePage));
        }
    }
}
