using Microsoft.Maui.Handlers;

namespace Maze.Maui.App
{
    partial class GameWebViewHandler : WebViewHandler
    {
        internal static bool IgnoreSslErrors { get; set; }
    }
}
