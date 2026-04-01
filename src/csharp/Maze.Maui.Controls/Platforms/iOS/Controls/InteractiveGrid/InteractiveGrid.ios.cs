using UIKit;

namespace Maze.Maui.Controls.InteractiveGrid
{
    public partial class Grid
    {
        partial void InitializePlatformSpecificCode()
        {
            DisableBounceWhenReady(_dataScrollView);
            DisableBounceWhenReady(_colHeaderScrollView);
            DisableBounceWhenReady(_rowHeaderScrollView);
        }

        private static void DisableBounceWhenReady(ScrollView scrollView)
        {
            scrollView.HandlerChanged += (s, e) =>
            {
                if (scrollView.Handler?.PlatformView is UIScrollView uiScrollView)
                    uiScrollView.Bounces = false;
            };
        }
    }
}
