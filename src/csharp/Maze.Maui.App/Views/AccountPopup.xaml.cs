using CommunityToolkit.Maui.Views;
using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// A popup that displays the user's account details and allows profile editing,
    /// password change, and account deletion.
    /// </summary>
    public partial class AccountPopup : Popup
    {
        private readonly AccountViewModel _viewModel;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="viewModel">The account view model</param>
        public AccountPopup(AccountViewModel viewModel)
        {
            _viewModel = viewModel;
            BindingContext = viewModel;
            InitializeComponent();
            viewModel.LoadProfileCommand.Execute(null);
        }

        /// <summary>
        /// Handles the Close button click.
        /// </summary>
        private async void OnCloseClicked(object sender, EventArgs e)
        {
            await CloseAsync();
        }
    }
}
