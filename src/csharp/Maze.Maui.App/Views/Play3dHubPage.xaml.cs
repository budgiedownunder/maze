using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// The 3D Games hub — tiles for the four browse scopes. Featured is live; the
    /// other three are placeholders until their sub-pages ship.
    /// </summary>
    public partial class Play3dHubPage : ContentPage
    {
        /// <summary>Constructor</summary>
        /// <param name="viewModel">Injected hub view model</param>
        public Play3dHubPage(Play3dHubViewModel viewModel)
        {
            InitializeComponent();
            BindingContext = viewModel;
        }
    }
}
