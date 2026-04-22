using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Net;
using System.Text.RegularExpressions;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents the view model for the login page
    /// </summary>
    public partial class LoginViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;
        private readonly IAppFeaturesService _appFeaturesService;

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SignInCommand))]
        private string email = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SignInCommand))]
        private string password = "";

        [ObservableProperty]
        private string errorMessage = "";

        [ObservableProperty]
        private bool showPassword = false;

        [ObservableProperty]
        private bool allowSignUp = true;

        [RelayCommand]
        private void ToggleShowPassword() => ShowPassword = !ShowPassword;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        /// <param name="appFeaturesService">Injected app features service</param>
        public LoginViewModel(IAuthService authService, IAppFeaturesService appFeaturesService)
        {
            Title = "Sign In";
            _authService = authService;
            _appFeaturesService = appFeaturesService;
        }

        /// <summary>
        /// Refreshes server feature flags and attempts to restore an existing session by verifying
        /// the stored token against the server.
        /// Returns true if the session is valid and the app should navigate to the main page.
        /// Returns false if there is no stored token or if the server cannot be reached, setting
        /// <see cref="ErrorMessage"/> with details of any connection failure.
        /// </summary>
        public async Task<bool> TryRestoreSessionAsync()
        {
            await _appFeaturesService.RefreshAsync();
            AllowSignUp = _appFeaturesService.Features.AllowSignUp;

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
            !string.IsNullOrWhiteSpace(Email) &&
            !string.IsNullOrWhiteSpace(Password) &&
            !IsBusy;

        [RelayCommand(CanExecute = nameof(CanSignIn))]
        private async Task SignIn()
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
                await _authService.SignInAsync(Email, Password);
                await Shell.Current.GoToAsync("//MainPage");
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Unauthorized)
            {
                ErrorMessage = "Invalid email or password";
            }
            catch
            {
                ErrorMessage = "Sign in failed. Please try again.";
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
