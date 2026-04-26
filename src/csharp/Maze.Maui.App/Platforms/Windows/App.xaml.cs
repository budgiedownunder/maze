using Microsoft.UI.Xaml;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace Maze.Maui.App.WinUI
{
    /// <summary>
    /// Provides application-specific behavior to supplement the default Application class.
    /// </summary>
    public partial class App : MauiWinUIApplication
    {
        /// <summary>
        /// Initializes the singleton application object.  This is the first line of authored code
        /// executed, and as such is the logical equivalent of main() or WinMain().
        /// </summary>
        public App()
        {
            // WinUIEx's WebAuthenticator needs this call early in app startup so it can
            // detect when the current activation came from a maze-app:// URL (i.e. the
            // OS reactivated us with an OAuth callback). Without it, AuthenticateAsync
            // throws "OAuth redirection check on app activation was not detected."
            WinUIEx.WebAuthenticator.CheckOAuthRedirectionActivation();

            this.InitializeComponent();
        }
        /// <summary>
        /// Application instance creation override
        /// </summary>
        /// <returns>Instance</returns>
        protected override MauiApp CreateMauiApp() => MauiProgram.CreateMauiApp();
    }

}
