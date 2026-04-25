namespace Maze.Maui.App
{
    partial class GameWebViewHandler
    {
        static GameWebViewHandler()
        {
            Mapper.AppendToMapping("GameWebViewSetup", (handler, view) =>
            {
                if (IgnoreSslErrors && handler is GameWebViewHandler gameHandler)
                    gameHandler.PlatformView.SetWebViewClient(
                        new Platforms.Android.IgnoreSslWebViewClient(gameHandler));
            });
        }
    }
}
