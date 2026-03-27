using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Net;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents the view model for the login page
    /// </summary>
    public partial class LoginViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SignInCommand))]
        private string username = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SignInCommand))]
        private string password = "";

        [ObservableProperty]
        private string errorMessage = "";

        [ObservableProperty]
        private bool showPassword = false;

        [RelayCommand]
        private void ToggleShowPassword() => ShowPassword = !ShowPassword;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        public LoginViewModel(IAuthService authService)
        {
            Title = "Sign In";
            _authService = authService;
        }

        /// <summary>
        /// Attempts to restore an existing session by verifying the stored token against the server.
        /// Returns true if the session is valid and the app should navigate to the main page.
        /// Returns false if there is no stored token or if the server cannot be reached, setting
        /// <see cref="ErrorMessage"/> with details of any connection failure.
        /// </summary>
        public async Task<bool> TryRestoreSessionAsync()
        {
            if (!await _authService.IsAuthenticatedAsync())
                return false;

            IsBusy = true;
            ErrorMessage = "";
            try
            {
                await _authService.GetMyProfileAsync();
                return true;
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Unauthorized)
            {
                await _authService.SignOutAsync();
                ErrorMessage = "Your session has expired, please sign in again.";
                return false;
            }
            catch (Exception)
            {
                ErrorMessage = "Unable to restore your session, please sign in again.";
                return false;
            }
            finally
            {
                IsBusy = false;
            }
        }

        private bool CanSignIn() =>
            !string.IsNullOrWhiteSpace(Username) &&
            !string.IsNullOrWhiteSpace(Password) &&
            !IsBusy;

        [RelayCommand(CanExecute = nameof(CanSignIn))]
        private async Task SignIn()
        {
            IsBusy = true;
            ErrorMessage = "";
            try
            {
                await _authService.SignInAsync(Username, Password);
                await Shell.Current.GoToAsync("//MainPage");
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

        [RelayCommand]
        private async Task GoToSignUp()
        {
            await Shell.Current.GoToAsync(nameof(SignUpPage));
        }
    }
}
