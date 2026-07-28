namespace Maze.Maui.App
{
    /// <summary>
    /// The reusable Play 3D list body — a search box over a paged card grid with
    /// pull-to-refresh and a Load-more button (the play-side analogue of the web
    /// client's list body). Bound to a <see cref="ViewModels.Play3dListViewModel"/>
    /// supplied by the host: the Featured page uses one, and each scope browser tab
    /// uses one, so every browse surface shares this component and its card template.
    /// </summary>
    public partial class Play3dListView : ContentView
    {
        // Cap for the left-aligned, width-responsive Load more button (matches the
        // Leaderboards page): fills a narrow screen, but never stretches on desktop.
        private const double LoadMoreMaxWidth = 480;

        public Play3dListView()
        {
            InitializeComponent();
            SizeChanged += OnSizeChanged;
        }

        private void OnSizeChanged(object? sender, EventArgs e)
        {
            // The list body carries no inner padding, so its own width is the
            // available content width for the button.
            if (Width > 0)
                LoadMoreButton.WidthRequest = Math.Min(Width, LoadMoreMaxWidth);
        }
    }
}
