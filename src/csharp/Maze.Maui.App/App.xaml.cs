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
        }

        /// <summary>
        /// Creates the application window
        /// </summary>
        /// <param name="activationState">Activation state</param>
        /// <returns>Window</returns>
        protected override Window CreateWindow(IActivationState? activationState) => new Window(new AppShell());
    }
}
