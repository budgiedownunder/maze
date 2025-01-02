namespace Maze.Maui.App
{
    /// <summary>
    /// MAUI application class
    /// </summary>
    public partial class App : Application
    {
        /// <summary>
        /// Constructor
        /// </summary>
        public App()
        {
            InitializeComponent();

            MainPage = new AppShell();
        }
    }
}
