namespace Maze.Maui.App
{
    /// <summary>
    /// MAUI application class
    /// </summary>
    public partial class App : Application
    {
        private readonly AppShell _appShell;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="appShell">Injected app shell</param>
        public App(AppShell appShell)
        {
            _appShell = appShell;
            InitializeComponent();
        }

        /// <summary>
        /// Creates the application window
        /// </summary>
        /// <param name="activationState">Activation state</param>
        /// <returns>Window</returns>
        protected override Window CreateWindow(IActivationState? activationState) => new Window(_appShell);
    }
}
