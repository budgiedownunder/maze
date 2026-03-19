using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;
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
        private readonly MazesViewModel _mazesViewModel;
        private readonly AccountViewModel _accountViewModel;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="mazesViewModel">Injected mazes view model</param>
        /// <param name="accountViewModel">Injected account view model</param>
        public AppShell(IAuthService authService, IDialogService dialogService, MazesViewModel mazesViewModel, AccountViewModel accountViewModel)
        {
            _authService = authService;
            _dialogService = dialogService;
            _mazesViewModel = mazesViewModel;
            _accountViewModel = accountViewModel;
            InitializeComponent();
            Routing.RegisterRoute(nameof(MazePage), typeof(MazePage));
            Routing.RegisterRoute(nameof(SignUpPage), typeof(SignUpPage));
            Routing.RegisterRoute(nameof(ChangePasswordPage), typeof(ChangePasswordPage));
        }

        /// <summary>
        /// Opens the Account popup.
        /// </summary>
        private async void OnAccountMenuItemClicked(object sender, EventArgs e)
        {
            FlyoutIsPresented = false;
            await CurrentPage.ShowPopupAsync(new AccountPopup(_accountViewModel));
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
                _mazesViewModel.InvalidateData();
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
