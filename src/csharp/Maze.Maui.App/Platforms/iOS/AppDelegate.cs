using Foundation;
using Microsoft.Maui.Authentication;
using UIKit;

namespace Maze.Maui.App
{
    /// <summary>
    /// Provides application-specific behavior to supplement the default Application class.
    /// </summary>
    [Register("AppDelegate")]
    public class AppDelegate : MauiUIApplicationDelegate
    {
        /// <summary>
        /// Application instance creation override
        /// </summary>
        /// <returns>Instance</returns>
        protected override MauiApp CreateMauiApp() => MauiProgram.CreateMauiApp();

        /// <summary>
        /// Forward custom-scheme URLs (e.g. maze-app://oauth-callback) into MAUI's
        /// WebAuthenticator so the in-flight AuthenticateAsync task can resolve.
        /// </summary>
        public override bool OpenUrl(UIApplication app, NSUrl url, NSDictionary options)
        {
            if (WebAuthenticator.Default.OpenUrl(new Uri(url.AbsoluteString!)))
                return true;
            return base.OpenUrl(app, url, options);
        }
    }
}
