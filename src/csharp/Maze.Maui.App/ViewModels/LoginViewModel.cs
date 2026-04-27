using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Collections.ObjectModel;
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
        private readonly AccountViewModel _accountViewModel;

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

        /// <summary>OAuth providers exposed by the server. Drives the per-provider
        /// button list on <see cref="LoginPage"/>; empty when OAuth is disabled.</summary>
        public ObservableCollection<OAuthProviderPublic> OAuthProviders { get; } = new();

        /// <summary>True when at least one OAuth provider is enabled — used to show/hide
        /// the divider and the OAuth section on the login form.</summary>
        public bool HasOAuthProviders => OAuthProviders.Count > 0;

        [RelayCommand]
        private void ToggleShowPassword() => ShowPassword = !ShowPassword;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        /// <param name="appFeaturesService">Injected app features service</param>
        /// <param name="accountViewModel">Injected (singleton) account view model — used to flip
        /// <see cref="AccountViewModel.IsWelcomeMode"/> when an OAuth sign-up creates a brand-new
        /// user, so the Account popup auto-opens with a welcome banner on the next page.</param>
        public LoginViewModel(IAuthService authService, IAppFeaturesService appFeaturesService, AccountViewModel accountViewModel)
        {
            Title = "Sign In";
            _authService = authService;
            _appFeaturesService = appFeaturesService;
            _accountViewModel = accountViewModel;
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
            SyncOAuthProviders(_appFeaturesService.Features.OAuthProviders);

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

        [RelayCommand]
        private async Task SignInWithOAuth(string? providerName)
        {
            if (string.IsNullOrWhiteSpace(providerName)) return;
            IsBusy = true;
            ErrorMessage = "";
            try
            {
                var result = await _authService.SignInWithOAuthAsync(providerName);
                // Flip the singleton AccountViewModel's IsWelcomeMode flag *before*
                // navigating: AppShell.OnNavigated will read it on arrival at the
                // main page and auto-open the Account popup with a welcome banner.
                _accountViewModel.IsWelcomeMode = result.IsNewUser;
                await Shell.Current.GoToAsync("//MainPage");
            }
            catch (OAuthFlowFailedException ex)
            {
                // Server-side recoverable error (e.g. signup_disabled,
                // email_not_verified, provider_error:access_denied) — show
                // a friendly per-code message.
                ErrorMessage = OAuthErrorMessages.FromReason(ex.Reason);
            }
            catch (TimeoutException)
            {
                // The OAuth flow exceeded its 5-minute upper bound — typically
                // because the user walked away from the consent screen.
                ErrorMessage = "Sign-in was cancelled or did not complete in time. Please try again.";
            }
            catch (TaskCanceledException)
            {
                // Broker reported a clean cancellation: iOS user dismissed
                // the auth sheet, or Windows user closed the WebView2 popup.
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Unauthorized)
            {
                ErrorMessage = "Sign in failed. Please try again.";
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

        private void SyncOAuthProviders(IEnumerable<OAuthProviderPublic> providers)
        {
            OAuthProviders.Clear();
            foreach (var p in providers)
                OAuthProviders.Add(p);
            OnPropertyChanged(nameof(HasOAuthProviders));
        }

        /// <summary>
        /// Force-rerenders the OAuth provider buttons. Used when the OS theme
        /// changes at runtime: the GitHub icon path that
        /// <c>OAuthProviderIconConverter</c> returns is theme-dependent and
        /// the converter is captured at bind time, so we clear and re-add
        /// the collection — which triggers <c>BindableLayout</c> to recreate
        /// each item's <c>Button</c>, re-running the converter under the new
        /// theme.
        /// </summary>
        public void RefreshOAuthProviderItems()
        {
            if (OAuthProviders.Count == 0) return;
            var snapshot = OAuthProviders.ToList();
            OAuthProviders.Clear();
            foreach (var p in snapshot)
                OAuthProviders.Add(p);
        }
    }
}
