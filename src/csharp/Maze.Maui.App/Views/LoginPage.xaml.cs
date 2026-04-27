using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views;

/// <summary>
/// Represents the login page within the application.
/// On appearing, attempts to restore an existing session by verifying the stored token against the
/// server and navigates to the main page if the session is still valid.
/// This is how the page appears on Windows Desktop:
/// 
///   <table>
///     <thead>
///       <tr>
///         <th><strong>Windows</strong></th>
///       </tr>
///     </thead>
///     <tbody>
///       <tr>
///         <td><img src="../../images/screenshots/windows-login.png" height="500" width="500"/></td>
///       </tr>
///     </tbody> 
///  </table>
///  
/// and this is how it appears on Android/iOS devices:
/// 
///   <table>
///     <thead>
///       <tr>
///         <th><strong>Android</strong></th>
///         <th><strong>iOS</strong></th>
///       </tr>
///     </thead>
///     <tbody>
///       <tr>
///         <td><img src="../../images/screenshots/android-login.png" width="250"/></td>
///         <td><img src="../../images/screenshots/ios-login.png" width="250"/></td>
///       </tr>
///     </tbody> 
///  </table>
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
        // Track OS-level theme changes so the GitHub OAuth icon swaps to the
        // light- or dark-fill variant at runtime. Subscribed here (and
        // unsubscribed in OnDisappearing) so the handler lifetime matches the
        // page's, regardless of the ViewModel's transient registration.
        if (Application.Current is { } app)
            app.RequestedThemeChanged += OnAppRequestedThemeChanged;
        if (await _viewModel.TryRestoreSessionAsync())
            await Shell.Current.GoToAsync("//MainPage");
    }

    /// <inheritdoc/>
    protected override void OnDisappearing()
    {
        base.OnDisappearing();
        if (Application.Current is { } app)
            app.RequestedThemeChanged -= OnAppRequestedThemeChanged;
    }

    private void OnAppRequestedThemeChanged(object? sender, AppThemeChangedEventArgs e)
        => _viewModel.RefreshOAuthProviderItems();
}
