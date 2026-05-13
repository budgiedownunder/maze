using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Home page — post-sign-in landing showing tile entries to the random
    /// 3D game and the maze list (Design and Play).
    /// </summary>
    public partial class HomePage : ContentPage
    {
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="viewModel">Injected home view model</param>
        public HomePage(HomeViewModel viewModel)
        {
            InitializeComponent();
            BindingContext = viewModel;
        }
    }
}
