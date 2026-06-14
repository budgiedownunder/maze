using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views;

/// <summary>
/// The Leaderboards page: a cascading Game Type → Game selector over a paged
/// board (metric toggle + load-more). The page-level <see cref="LeaderboardsViewModel"/>
/// holds the logic; this code-behind only triggers the first load and forwards
/// the Game picker's selection change to the reload command (the view model keeps
/// its property hooks free of async work so it stays unit-testable).
/// </summary>
public partial class LeaderboardsPage : ContentPage
{
    private readonly LeaderboardsViewModel viewModel;
    private bool _loaded;

    /// <summary>
    /// Constructor
    /// </summary>
    /// <param name="viewModel">Injected leaderboards view model</param>
    public LeaderboardsPage(LeaderboardsViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = this.viewModel = viewModel;
    }

    /// <inheritdoc/>
    protected override void OnAppearing()
    {
        base.OnAppearing();
        if (_loaded)
            return;
        _loaded = true;
        viewModel.InitializeCommand.Execute(null);
    }

    private void OnGameSelectionChanged(object? sender, EventArgs e)
    {
        viewModel.ReloadBoardCommand.Execute(null);
    }
}
