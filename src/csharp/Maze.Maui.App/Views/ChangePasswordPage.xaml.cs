using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views;

/// <summary>
/// Represents the change password page within the application.
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
///         <td><img src="../../images/screenshots/windows-change-password.png" height="500" width="500"/></td>
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
///         <td><img src="../../images/screenshots/android-change-password.png" width="250"/></td>
///         <td><img src="../../images/screenshots/ios-change-password.png" width="250"/></td>
///       </tr>
///     </tbody> 
///  </table>
/// </summary>
public partial class ChangePasswordPage : ContentPage
{
    /// <summary>
    /// Constructor
    /// </summary>
    /// <param name="viewModel">Injected change password view model</param>
    public ChangePasswordPage(ChangePasswordViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = viewModel;
    }
}
