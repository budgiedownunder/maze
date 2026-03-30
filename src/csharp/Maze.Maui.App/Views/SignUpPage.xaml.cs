using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views;

/// <summary>
/// Represents the sign up page within the application
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
///         <td><img src="../../images/screenshots/windows-sign-up.png" height="500" width="500"/></td>
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
///         <td><img src="../../images/screenshots/android-sign-up.png" width="250"/></td>
///         <td><img src="../../images/screenshots/ios-sign-up.png" width="250"/></td>
///       </tr>
///     </tbody> 
///  </table>
/// </summary>
public partial class SignUpPage : ContentPage
{
    /// <summary>
    /// Constructor
    /// </summary>
    /// <param name="viewModel">Injected sign up view model</param>
    public SignUpPage(SignUpViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = viewModel;
    }
}
