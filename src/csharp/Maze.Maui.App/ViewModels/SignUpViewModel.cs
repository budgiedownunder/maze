using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents the view model for the sign up page
    /// </summary>
    public partial class SignUpViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SignUpCommand))]
        private string username = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SignUpCommand))]
        private string fullName = "";

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

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        public SignUpViewModel(IAuthService authService)
        {
            Title = "Sign Up";
            _authService = authService;
        }

        private bool CanSignUp() =>
            !string.IsNullOrWhiteSpace(Username) &&
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
                await _authService.SignUpAsync(Username, FullName, Email, Password);
                await Shell.Current.GoToAsync("..");
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
