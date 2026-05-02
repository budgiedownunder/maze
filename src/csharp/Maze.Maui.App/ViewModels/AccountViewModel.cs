using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Net;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents the view model for the account page
    /// </summary>
    public partial class AccountViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;
        private readonly IDialogService _dialogService;

        private string _loadedUsername = "";
        private string _loadedFullName = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SaveProfileCommand))]
        private string username = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SaveProfileCommand))]
        private string fullName = "";

        [ObservableProperty]
        private bool isAdmin;

        [ObservableProperty]
        private string errorMessage = "";

        [ObservableProperty]
        private string loadStatus = "";

        /// <summary>
        /// When true, the AccountPopup renders a one-line welcome banner above
        /// the form. Set by the OAuth sign-in flow when the server signals
        /// <c>new_user=true</c>; cleared by AppShell after the popup is
        /// auto-shown so subsequent burger-menu opens of the Account UI don't
        /// keep showing the banner.
        /// </summary>
        [ObservableProperty]
        private bool isWelcomeMode;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        /// <param name="dialogService">Injected dialog service</param>
        public AccountViewModel(IAuthService authService, IDialogService dialogService)
        {
            Title = "Account";
            _authService = authService;
            _dialogService = dialogService;
        }

        /// <summary>
        /// Loads the user's profile from the server
        /// </summary>
        [RelayCommand]
        private async Task LoadProfile()
        {
            if (IsBusy)
                return;

            ClearProfile();
            IsBusy = true;
            try
            {
                var profile = await _authService.GetMyProfileAsync();
                Username = _loadedUsername = profile.Username;
                FullName = _loadedFullName = profile.FullName;
                IsAdmin = profile.IsAdmin;
                LoadStatus = "";
            }
            catch
            {
                ErrorMessage = "Failed to load profile. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        /// <summary>
        /// Clears all profile fields and sets the load status message
        /// </summary>
        public void ClearProfile()
        {
            Username = _loadedUsername = "";
            FullName = _loadedFullName = "";
            IsAdmin = false;
            ErrorMessage = "";
            LoadStatus = "Loading profile...";
        }

        partial void OnUsernameChanged(string value) => ErrorMessage = "";
        partial void OnFullNameChanged(string value) => ErrorMessage = "";

        private bool CanSaveProfile() =>
            !IsBusy &&
            !string.IsNullOrWhiteSpace(Username) &&
            (Username != _loadedUsername || FullName != _loadedFullName);

        /// <summary>
        /// Saves the user's updated profile to the server
        /// </summary>
        [RelayCommand(CanExecute = nameof(CanSaveProfile))]
        private async Task SaveProfile()
        {
            IsBusy = true;
            ErrorMessage = "";
            try
            {
                var profile = await _authService.UpdateProfileAsync(Username, FullName);
                Username = _loadedUsername = profile.Username;
                FullName = _loadedFullName = profile.FullName;
                IsAdmin = profile.IsAdmin;
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Conflict)
            {
                ErrorMessage = "Username is already in use by another account";
            }
            catch
            {
                ErrorMessage = "Failed to save profile. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        /// <summary>
        /// Navigates to the change password page
        /// </summary>
        [RelayCommand]
        private async Task ChangePassword()
        {
            await Shell.Current.GoToAsync(nameof(ChangePasswordPage));
        }

        /// <summary>
        /// Confirms and deletes the user's account, then navigates to the login page
        /// </summary>
        [RelayCommand]
        private async Task DeleteAccount()
        {
            bool confirmed = await _dialogService.ShowConfirmation(
                "Delete Account",
                "Are you sure you want to permanently delete your account? This will also delete all your mazes and cannot be undone.",
                "Delete",
                "Cancel",
                isDestructive: true);

            if (!confirmed)
                return;

            IsBusy = true;
            ErrorMessage = "";
            try
            {
                await _authService.DeleteMyAccountAsync();
                WeakReferenceMessenger.Default.Send(new MazesInvalidatedMessage());
                ClearProfile();
                await Shell.Current.GoToAsync("//LoginPage");
            }
            catch
            {
                ErrorMessage = "Failed to delete account. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }
    }
}
