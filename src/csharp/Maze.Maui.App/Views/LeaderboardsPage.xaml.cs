using System.ComponentModel;
using Maze.Maui.App.ViewModels;
using Maze.Maui.Controls.Pointer;

namespace Maze.Maui.App.Views;

/// <summary>
/// The Leaderboards page: a Game Type → Game selector (maze picker, or a scoped
/// 3D-game picker) over a paged board (metric toggle + load-more). The page-level
/// <see cref="LeaderboardsViewModel"/> holds the logic; this code-behind triggers
/// the first load, forwards the maze picker's selection change to the reload
/// command, applies a <c>def</c> preselect from a card's Leaderboard button, and
/// reflects the view model's busy state as a wait cursor while a board (re)loads.
/// </summary>
[QueryProperty(nameof(PreselectDefinitionId), "def")]
public partial class LeaderboardsPage : ContentPage
{
    // Cap for the left-aligned, page-width-responsive Load more button.
    private const double LoadMoreMaxWidth = 480;
    // Root grid horizontal padding (10 left + 10 right).
    private const double ContentHorizontalPadding = 20;

    private readonly LeaderboardsViewModel viewModel;
    private bool _loaded;
    private string? _preselectDefinitionId;

    /// <summary>The 3D game to preselect, from the <c>def</c> nav argument (a card's Leaderboard button).</summary>
    public string? PreselectDefinitionId
    {
        get => _preselectDefinitionId;
        set
        {
            _preselectDefinitionId = value;
            viewModel.SetPreselectGame(value);
        }
    }

    /// <summary>
    /// Constructor
    /// </summary>
    /// <param name="viewModel">Injected leaderboards view model</param>
    public LeaderboardsPage(LeaderboardsViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = this.viewModel = viewModel;
        viewModel.PropertyChanged += OnViewModelPropertyChanged;
        SizeChanged += OnPageSizeChanged;
        UpdateResetToolbarItem();
    }

    private void OnPageSizeChanged(object? sender, EventArgs e)
    {
        // Keep Load more left-aligned (HorizontalOptions=Start) while filling the
        // content width up to a cap — so it shrinks with the page yet never stretches.
        double available = Width - ContentHorizontalPadding;
        if (available > 0)
            LoadMoreButton.WidthRequest = Math.Min(available, LoadMoreMaxWidth);
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

    private void OnViewModelPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        // Show a wait cursor while a board (re)loads, mirroring the rest of the app.
        if (e.PropertyName == nameof(LeaderboardsViewModel.IsBusy))
            Pointer.SetCursor(this, viewModel.IsBusy ? Icon.Wait : Icon.Arrow);
        else if (e.PropertyName == nameof(LeaderboardsViewModel.ShowReset))
            UpdateResetToolbarItem();
    }

    // Reflects ShowReset by adding (left of Refresh) or removing the Reset toolbar
    // item — ToolbarItem can't bind IsVisible, so visibility is membership.
    private void UpdateResetToolbarItem()
    {
        bool present = ToolbarItems.Contains(ResetToolbarItem);
        if (viewModel.ShowReset && !present)
            ToolbarItems.Insert(0, ResetToolbarItem);
        else if (!viewModel.ShowReset && present)
            ToolbarItems.Remove(ResetToolbarItem);
    }
}
