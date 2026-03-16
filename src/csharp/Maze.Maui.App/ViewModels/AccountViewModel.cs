using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents the view model for the account page
    /// </summary>
    public partial class AccountViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;
        private readonly IDialogService _dialogService;

        [ObservableProperty]
        private string username = "";

        [ObservableProperty]
        private string fullName = "";

        [ObservableProperty]
        private string email = "";

        [ObservableProperty]
        private bool isAdmin;

        [ObservableProperty]
        private string errorMessage = "";

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

            IsBusy = true;
            ErrorMessage = "";
            try
            {
                var profile = await _authService.GetMyProfileAsync();
                Username = profile.Username;
                FullName = profile.FullName;
                Email = profile.Email;
                IsAdmin = profile.IsAdmin;
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
        /// Signs the user out and navigates to the login page
        /// </summary>
        [RelayCommand]
        private async Task SignOut()
        {
            IsBusy = true;
            try
            {
                await _authService.SignOutAsync();
                await Shell.Current.GoToAsync("//LoginPage");
            }
            finally
            {
                IsBusy = false;
            }
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
                "Cancel");

            if (!confirmed)
                return;

            IsBusy = true;
            ErrorMessage = "";
            try
            {
                await _authService.DeleteMyAccountAsync();
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
