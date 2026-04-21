using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Net;
using System.Text.RegularExpressions;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents the view model for the account page
    /// </summary>
    public partial class AccountViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;
        private readonly IDialogService _dialogService;
        private readonly MazesViewModel _mazesViewModel;

        private string _loadedUsername = "";
        private string _loadedFullName = "";
        private string _loadedEmail = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SaveProfileCommand))]
        private string username = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SaveProfileCommand))]
        private string fullName = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SaveProfileCommand))]
        private string email = "";

        [ObservableProperty]
        private bool isAdmin;

        [ObservableProperty]
        private string errorMessage = "";

        [ObservableProperty]
        private string loadStatus = "";

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="mazesViewModel">Injected view model</param>
        public AccountViewModel(IAuthService authService, IDialogService dialogService, MazesViewModel mazesViewModel)
        {
            Title = "Account";
            _authService = authService;
            _dialogService = dialogService;
            _mazesViewModel = mazesViewModel;
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
                Email = _loadedEmail = profile.Email;
                IsAdmin = profile.IsAdmin;
                LoadStatus = "";
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
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
            Email = _loadedEmail = "";
            IsAdmin = false;
            ErrorMessage = "";
            LoadStatus = "Loading profile...";
        }

        private bool CanSaveProfile() =>
            !IsBusy &&
            !string.IsNullOrWhiteSpace(Username) &&
            !string.IsNullOrWhiteSpace(Email) &&
            (Username != _loadedUsername || FullName != _loadedFullName || Email != _loadedEmail);

        /// <summary>
        /// Saves the user's updated profile to the server
        /// </summary>
        [RelayCommand(CanExecute = nameof(CanSaveProfile))]
        private async Task SaveProfile()
        {
            if (!Regex.IsMatch(Email, @"^[^@\s]+@[^@\s]+\.[^@\s]+$"))
            {
                ErrorMessage = "Please enter a valid email address";
                return;
            }
            IsBusy = true;
            ErrorMessage = "";
            try
            {
                var profile = await _authService.UpdateProfileAsync(Username, FullName, Email);
                Username = _loadedUsername = profile.Username;
                FullName = _loadedFullName = profile.FullName;
                Email = _loadedEmail = profile.Email;
                IsAdmin = profile.IsAdmin;
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Conflict)
            {
                ErrorMessage = "Username or email is already in use by another account";
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
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
                _mazesViewModel.InvalidateData();
                ClearProfile();
                await Shell.Current.GoToAsync("//LoginPage");
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
            }
            finally
            {
                IsBusy = false;
            }
        }
    }
}
