using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;
using System.Net;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents the view model for the change password page
    /// </summary>
    public partial class ChangePasswordViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;

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

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        public ChangePasswordViewModel(IAuthService authService)
        {
            Title = "Change Password";
            _authService = authService;
        }

        private bool CanChangePassword() =>
            !string.IsNullOrWhiteSpace(CurrentPassword) &&
            !string.IsNullOrWhiteSpace(NewPassword) &&
            !string.IsNullOrWhiteSpace(ConfirmNewPassword) &&
            !IsBusy;

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
                await _authService.ChangePasswordAsync(CurrentPassword, NewPassword);
                await Shell.Current.GoToAsync("..");
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Unauthorized)
            {
                ErrorMessage = "Current password is incorrect";
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
        private async Task GoBack()
        {
            await Shell.Current.GoToAsync("..");
        }
    }
}
