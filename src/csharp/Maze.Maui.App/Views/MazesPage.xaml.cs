using Maze.Maui.App.ViewModels;
using Maze.Maui.Controls.Pointer;

namespace Maze.Maui.App.Views;

/// <summary>
/// This class represents the maze list page within the application.
/// 
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
///         <td><img src="../../images/screenshots/windows-mazes.png" height="500" width="500"/></td>
///       </tr>
///     </tbody> 
///  </table>
///  
/// and this is how it appears on Android/iOS devices as a list:
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
///         <td><img src="../../images/screenshots/android-mazes.png" width="250"/></td>
///         <td><img src="../../images/screenshots/ios-mazes.png" width="250"/></td>
///       </tr>
///     </tbody> 
///  </table>
///
/// and this is how it appears on Android/iOS devices with the swipe-left buttons visible:
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
///         <td><img src="../../images/screenshots/android-mazes-swipe-left.png" width="250"/></td>
///         <td><img src="../../images/screenshots/ios-mazes-swipe-left.png" width="250"/></td>
///       </tr>
///     </tbody> 
///  </table>
/// </summary>
public partial class MazesPage : ContentPage
{
    private readonly MazesViewModel viewModel;

    /// <summary>
    /// Constructor 
    /// </summary>
    /// <param name="viewModel">Injected mazes view model</param>
    /// 
    public MazesPage(MazesViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = this.viewModel = viewModel;
        SizeChanged += OnSizeChanged;
    }

    private void OnSizeChanged(object? sender, EventArgs e)
    {
        viewModel.UseShortDimensions = Width < 300;
    }

    protected override void OnAppearing()
    {
        base.OnAppearing();
        if (!viewModel.IsDataLoaded)
            viewModel.GetMazesCommand.Execute(null);
    }

    protected override void OnNavigatedTo(NavigatedToEventArgs args)
    {
        base.OnNavigatedTo(args);
        if (viewModel.IsDataLoaded)
        {
            Pointer.SetCursor(this, Icon.Wait);
            viewModel.IsBusy = true;
            Dispatcher.Dispatch(async () =>
            {
                await Task.Delay(300);
                viewModel.IsBusy = false;
                Pointer.SetCursor(this, Icon.Arrow);
            });
        }
    }
}