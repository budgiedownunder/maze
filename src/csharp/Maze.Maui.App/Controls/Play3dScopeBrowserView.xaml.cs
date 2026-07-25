namespace Maze.Maui.App
{
    /// <summary>
    /// The reusable Play 3D scope-browser surface — a Games / Collections tab strip
    /// over two independently-paged card lists, with search + sort for the searchable
    /// (Community) scope. Bound to a <see cref="ViewModels.Play3dScopeBrowserViewModel"/>
    /// supplied by the hosting page; the three scope pages differ only in that model.
    /// </summary>
    public partial class Play3dScopeBrowserView : ContentView
    {
        public Play3dScopeBrowserView()
        {
            InitializeComponent();
        }
    }
}
