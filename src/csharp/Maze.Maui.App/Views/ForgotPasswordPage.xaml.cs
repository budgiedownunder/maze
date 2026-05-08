using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views;

/// <summary>
/// Forgot Password page — email-only form whose submit triggers the
/// password-reset request endpoint and flips the page into a "check your
/// inbox" success state.
/// </summary>
public partial class ForgotPasswordPage : ContentPage
{
    /// <summary>
    /// Constructor.
    /// </summary>
    /// <param name="viewModel">Injected forgot-password view model.</param>
    public ForgotPasswordPage(ForgotPasswordViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = viewModel;
    }
}
