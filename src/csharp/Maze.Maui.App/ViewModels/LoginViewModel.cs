using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;

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

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        public LoginViewModel(IAuthService authService)
        {
            Title = "Sign In";
            _authService = authService;
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
