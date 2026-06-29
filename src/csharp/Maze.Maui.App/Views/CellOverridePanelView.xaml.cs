namespace Maze.Maui.App.Views
{
    /// <summary>
    /// Inline per-cell override editor for a single selected feature cell. Bound to a
    /// <see cref="ViewModels.CellOverridePanelViewModel"/> (the page sets the binding
    /// context to one wired to the live grid).
    /// </summary>
    public partial class CellOverridePanelView : ContentView
    {
        /// <summary>Constructor.</summary>
        public CellOverridePanelView()
        {
            InitializeComponent();
        }
    }
}
