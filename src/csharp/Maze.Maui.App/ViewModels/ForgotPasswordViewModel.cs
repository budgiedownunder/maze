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

        /// <summary>
        /// Constructor.
        /// </summary>
        /// <param name="authService">Injected auth service.</param>
        /// <param name="navigationService">Injected navigation service.</param>
        public ForgotPasswordViewModel(IAuthService authService, INavigationService navigationService)
        {
            Title = "Forgot Password";
            _authService = authService;
            _navigationService = navigationService;
        }

        partial void OnEmailChanged(string value) => ErrorMessage = "";

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
