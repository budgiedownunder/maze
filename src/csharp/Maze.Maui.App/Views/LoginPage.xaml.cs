using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views;

/// <summary>
/// Represents the login page within the application.
/// On appearing, checks whether the user is already authenticated and navigates to the main page if so,
/// allowing app restart with a persisted token to skip the login page.
/// </summary>
public partial class LoginPage : ContentPage
{
    private readonly LoginViewModel _viewModel;
    private readonly IAuthService _authService;

    /// <summary>
    /// Constructor
    /// </summary>
    /// <param name="viewModel">Injected login view model</param>
    /// <param name="authService">Injected auth service</param>
    public LoginPage(LoginViewModel viewModel, IAuthService authService)
    {
        InitializeComponent();
        BindingContext = _viewModel = viewModel;
        _authService = authService;
    }

    /// <inheritdoc/>
    protected override async void OnAppearing()
    {
        base.OnAppearing();
        if (await _authService.IsAuthenticatedAsync())
            await Shell.Current.GoToAsync("//MainPage");
    }
}
