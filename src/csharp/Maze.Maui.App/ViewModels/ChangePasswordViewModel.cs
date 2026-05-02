using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Services;
using Microsoft.Maui.Controls;
using System.Net;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Backing ViewModel for the password change/set page. Branches on
    /// <see cref="HasPassword"/>: when the authenticated user already has
    /// a password (Change variant) the form requires the current password
    /// and submits to <see cref="IAuthService.ChangePasswordAsync"/>; when
    /// they don't (Set variant — OAuth-only user adding a password as a
    /// second login method), the Current Password field is irrelevant and
    /// the submit goes to <see cref="IAuthService.SetInitialPasswordAsync"/>.
    /// </summary>
    [QueryProperty(nameof(HasPassword), "HasPassword")]
    public partial class ChangePasswordViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;
        private readonly INavigationService _navigationService;

        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(HeadingText))]
        [NotifyPropertyChangedFor(nameof(SubmitText))]
        [NotifyPropertyChangedFor(nameof(SubmitBusyText))]
        [NotifyCanExecuteChangedFor(nameof(ChangePasswordCommand))]
        private bool hasPassword = true;

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(ChangePasswordCommand))]
        private string currentPassword = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(ChangePasswordCommand))]
        private string newPassword = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(ChangePasswordCommand))]
        private string confirmNewPassword = "";

        [ObservableProperty]
        private string errorMessage = "";

        [ObservableProperty]
        private bool showCurrentPassword = false;

        [ObservableProperty]
        private bool showNewPassword = false;

        [ObservableProperty]
        private bool showConfirmNewPassword = false;

        /// <summary>Title shown on the page header.</summary>
        public string HeadingText => HasPassword ? "Change Password" : "Set Password";

        /// <summary>Submit-button label while idle.</summary>
        public string SubmitText => HasPassword ? "Change Password" : "Set Password";

        /// <summary>Submit-button label while a request is in flight.</summary>
        public string SubmitBusyText => HasPassword ? "Changing..." : "Setting...";

        [RelayCommand]
        private void ToggleShowCurrentPassword() => ShowCurrentPassword = !ShowCurrentPassword;

        [RelayCommand]
        private void ToggleShowNewPassword() => ShowNewPassword = !ShowNewPassword;

        [RelayCommand]
        private void ToggleShowConfirmNewPassword() => ShowConfirmNewPassword = !ShowConfirmNewPassword;

        public ChangePasswordViewModel(IAuthService authService, INavigationService navigationService)
        {
            Title = "Change Password";
            _authService = authService;
            _navigationService = navigationService;
        }

        private bool CanChangePassword()
        {
            if (IsBusy) return false;
            if (string.IsNullOrWhiteSpace(NewPassword)) return false;
            if (string.IsNullOrWhiteSpace(ConfirmNewPassword)) return false;
            // Current password is required only in the Change variant —
            // OAuth-only users have no current password to verify.
            if (HasPassword && string.IsNullOrWhiteSpace(CurrentPassword)) return false;
            return true;
        }

        [RelayCommand(CanExecute = nameof(CanChangePassword))]
        private async Task ChangePassword()
        {
            if (NewPassword != ConfirmNewPassword)
            {
                ErrorMessage = "New passwords do not match";
                return;
            }

            if (NewPassword.Length < 8 ||
                !NewPassword.Any(char.IsUpper) ||
                !NewPassword.Any(char.IsLower) ||
                !NewPassword.Any(char.IsDigit) ||
                !NewPassword.Any(c => !char.IsLetterOrDigit(c)))
            {
                ErrorMessage = "New password must be at least 8 characters and contain uppercase, lowercase, digit and symbol";
                return;
            }

            IsBusy = true;
            ErrorMessage = "";
            try
            {
                if (HasPassword)
                {
                    await _authService.ChangePasswordAsync(CurrentPassword, NewPassword);
                }
                else
                {
                    await _authService.SetInitialPasswordAsync(NewPassword);
                }
                // Notify before navigating back so AccountViewModel (or any
                // other recipient) sees the state change while the trigger
                // page is still mounting; avoids a one-frame flicker on the
                // trigger button label.
                WeakReferenceMessenger.Default.Send(new PasswordSetMessage());
                await _navigationService.GoBackAsync();
            }
            catch (HttpRequestException ex) when (HasPassword && ex.StatusCode == HttpStatusCode.Unauthorized)
            {
                ErrorMessage = "Current password is incorrect";
            }
            catch
            {
                ErrorMessage = HasPassword
                    ? "Failed to change password. Please try again."
                    : "Failed to set password. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        [RelayCommand]
        private async Task GoBack()
        {
            await _navigationService.GoBackAsync();
        }
    }
}
