using Maze.Maui.App.Views;

namespace Maze.Maui.App
{
    /// <summary>
    /// MAUI application shell class
    /// </summary>
    public partial class AppShell : Shell
    {
        /// <summary>
        /// Constructor
        /// </summary>
        public AppShell()
        {
            InitializeComponent();
            Routing.RegisterRoute(nameof(MazePage), typeof(MazePage));
            Routing.RegisterRoute(nameof(SignUpPage), typeof(SignUpPage));
        }
    }
}
