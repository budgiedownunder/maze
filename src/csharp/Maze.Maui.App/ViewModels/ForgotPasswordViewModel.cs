using System.Text.RegularExpressions;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Backing ViewModel for the Forgot Password page. Drives an email-only
    /// form whose submit fires the password-reset request endpoint and
    /// flips into a "check your inbox" success state. The success copy is
    /// the same regardless of whether the address is registered — the
    /// server returns 200 unconditionally to avoid leaking which emails
    /// have accounts.
    /// </summary>
    public partial class ForgotPasswordViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;
        private readonly INavigationService _navigationService;

        // Same regex used by LoginViewModel and the React isValidEmail helper.
        private static readonly Regex EmailFormat = new(@"^[^@\s]+@[^@\s]+\.[^@\s]+$", RegexOptions.Compiled);

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SubmitCommand))]
        private string email = "";

        [ObservableProperty]
        private string errorMessage = "";

        /// <summary>Flips to true once the request endpoint has been called
        /// successfully, switching the page from the entry form to the
        /// "check your inbox" state. Stays false on transport-level
        /// failures so the user can retry.</summary>
        [ObservableProperty]
        private bool submitted;

        /// <summary>True when the server is configured to send transactional
        /// email and the reset flow is therefore usable. When false the page
        /// renders a single "Password reset is unavailable on this server."
        /// message instead of the entry form. Captured at construction from
        /// the features service singleton, populated earlier by
        /// <c>LoginViewModel.TryRestoreSessionAsync</c>.</summary>
        public bool EmailEnabled { get; }

        /// <summary>True when the entry form should render: email is enabled
        /// AND the request hasn't been submitted yet.</summary>
        public bool ShowForm => EmailEnabled && !Submitted;

        /// <summary>True when the "check your inbox" success state should
        /// render: email is enabled AND the request has succeeded.</summary>
        public bool ShowSuccess => EmailEnabled && Submitted;

        /// <summary>True when the "unavailable on this server" state should
        /// render: email is disabled.</summary>
        public bool ShowUnavailable => !EmailEnabled;

        /// <summary>
        /// Constructor.
        /// </summary>
        /// <param name="authService">Injected auth service.</param>
        /// <param name="appFeaturesService">Injected features service — read once
        /// at construction to capture <see cref="EmailEnabled"/>.</param>
        /// <param name="navigationService">Injected navigation service.</param>
        public ForgotPasswordViewModel(IAuthService authService, IAppFeaturesService appFeaturesService, INavigationService navigationService)
        {
            Title = "Forgot Password";
            _authService = authService;
            _navigationService = navigationService;
            EmailEnabled = appFeaturesService.Features.EmailEnabled;
        }

        partial void OnEmailChanged(string value) => ErrorMessage = "";

        partial void OnSubmittedChanged(bool value)
        {
            // ShowForm + ShowSuccess derive from Submitted; raise PropertyChanged
            // so the view's IsVisible bindings re-evaluate when Submitted flips.
            OnPropertyChanged(nameof(ShowForm));
            OnPropertyChanged(nameof(ShowSuccess));
        }

        private bool CanSubmit() =>
            !string.IsNullOrWhiteSpace(Email) &&
            !IsBusy;

        [RelayCommand(CanExecute = nameof(CanSubmit))]
        private async Task Submit()
        {
            if (!EmailFormat.IsMatch(Email))
            {
                ErrorMessage = "Please enter a valid email address";
                return;
            }
            IsBusy = true;
            ErrorMessage = "";
            try
            {
                await _authService.RequestPasswordResetAsync(Email);
                Submitted = true;
            }
            catch
            {
                // Anti-enumeration: the server returns 200 for any address,
                // so a thrown exception here means a transport-level failure
                // (network, TLS, etc.) — surface a retry message rather than
                // anything that would hint at registration status.
                ErrorMessage = "Could not send the reset link. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        [RelayCommand]
        private async Task BackToSignIn()
        {
            await _navigationService.GoBackAsync();
        }
    }
}
