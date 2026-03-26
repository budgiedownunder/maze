using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views;

/// <summary>
/// Represents the login page within the application.
/// On appearing, attempts to restore an existing session by verifying the stored token against the
/// server and navigates to the main page if the session is still valid.
/// </summary>
public partial class LoginPage : ContentPage
{
    private readonly LoginViewModel _viewModel;

    /// <summary>
    /// Constructor
    /// </summary>
    /// <param name="viewModel">Injected login view model</param>
    public LoginPage(LoginViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = _viewModel = viewModel;
    }

    /// <inheritdoc/>
    protected override async void OnAppearing()
    {
        base.OnAppearing();
        if (await _viewModel.TryRestoreSessionAsync())
            await Shell.Current.GoToAsync("//MainPage");
    }
}
