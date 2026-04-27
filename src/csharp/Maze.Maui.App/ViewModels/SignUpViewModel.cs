using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;
using System.Collections.ObjectModel;
using System.Net;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents the view model for the sign up page
    /// </summary>
    public partial class SignUpViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;
        private readonly IAppFeaturesService _appFeaturesService;

        /// <summary>OAuth providers exposed by the server. Same data as on the login
        /// form — both screens render the same OAuth buttons because the server does
        /// not distinguish "sign in" from "sign up" intent.</summary>
        public ObservableCollection<OAuthProviderPublic> OAuthProviders { get; } = new();

        /// <summary>True when at least one OAuth provider is enabled.</summary>
        public bool HasOAuthProviders => OAuthProviders.Count > 0;

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SignUpCommand))]
        private string email = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SignUpCommand))]
        private string password = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SignUpCommand))]
        private string confirmPassword = "";

        [ObservableProperty]
        private string errorMessage = "";

        [ObservableProperty]
        private bool showPassword = false;

        [ObservableProperty]
        private bool showConfirmPassword = false;

        partial void OnEmailChanged(string value) => ErrorMessage = "";
        partial void OnPasswordChanged(string value) => ErrorMessage = "";
        partial void OnConfirmPasswordChanged(string value) => ErrorMessage = "";

        [RelayCommand]
        private void ToggleShowPassword() => ShowPassword = !ShowPassword;

        [RelayCommand]
        private void ToggleShowConfirmPassword() => ShowConfirmPassword = !ShowConfirmPassword;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        /// <param name="appFeaturesService">Injected app features service</param>
        public SignUpViewModel(IAuthService authService, IAppFeaturesService appFeaturesService)
        {
            Title = "Sign Up";
            _authService = authService;
            _appFeaturesService = appFeaturesService;
        }

        /// <summary>
        /// Refreshes the OAuth providers list from the server. Called on page appear so
        /// the OAuth buttons reflect any server-side configuration change since last visit.
        /// </summary>
        public async Task RefreshOAuthProvidersAsync()
        {
            await _appFeaturesService.RefreshAsync();
            OAuthProviders.Clear();
            foreach (var p in _appFeaturesService.Features.OAuthProviders)
                OAuthProviders.Add(p);
            OnPropertyChanged(nameof(HasOAuthProviders));
        }

        private bool CanSignUp() =>
            !string.IsNullOrWhiteSpace(Email) &&
            !string.IsNullOrWhiteSpace(Password) &&
            !string.IsNullOrWhiteSpace(ConfirmPassword) &&
            !IsBusy;

        [RelayCommand(CanExecute = nameof(CanSignUp))]
        private async Task SignUp()
        {
            if (Password != ConfirmPassword)
            {
                ErrorMessage = "Passwords do not match";
                return;
            }

            if (Password.Length < 8 ||
                !Password.Any(char.IsUpper) ||
                !Password.Any(char.IsLower) ||
                !Password.Any(char.IsDigit) ||
                !Password.Any(c => !char.IsLetterOrDigit(c)))
            {
                ErrorMessage = "Password must be at least 8 characters and contain uppercase, lowercase, digit and symbol";
                return;
            }

            IsBusy = true;
            ErrorMessage = "";
            try
            {
                await _authService.SignUpAsync(Email, Password);
                await Shell.Current.GoToAsync("..");
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Conflict)
            {
                ErrorMessage = "Email already in use";
            }
            catch
            {
                ErrorMessage = "Sign up failed. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        [RelayCommand]
        private async Task GoBack()
        {
            await Shell.Current.GoToAsync("..");
        }

        [RelayCommand]
        private async Task SignInWithOAuth(string? providerName)
        {
            if (string.IsNullOrWhiteSpace(providerName)) return;
            IsBusy = true;
            ErrorMessage = "";
            try
            {
                await _authService.SignInWithOAuthAsync(providerName);
                await Shell.Current.GoToAsync("//MainPage");
            }
            catch (OAuthFlowFailedException ex)
            {
                ErrorMessage = OAuthErrorMessages.FromReason(ex.Reason);
            }
            catch (TimeoutException)
            {
                ErrorMessage = "Sign-in was cancelled or did not complete in time. Please try again.";
            }
            catch (TaskCanceledException)
            {
                // Broker reported a clean cancellation.
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"OAuth sign-in failed: {ex}");
                ErrorMessage = $"Sign in with {providerName} failed. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }
    }
}
