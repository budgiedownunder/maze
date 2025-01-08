using Maze.Maui.App.ViewModels;

namespace Maze.Maui.App.Views;

public partial class MazesPage : ContentPage
{
    private readonly MazesViewModel viewModel;
    public MazesPage(MazesViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = this.viewModel = viewModel;
    }
}