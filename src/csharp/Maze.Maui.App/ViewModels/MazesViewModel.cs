using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Maze.Maui.App.ViewModels
{
    public partial class MazesViewModel : BaseViewModel
    {
        IMazeService mazeService;
        public ObservableCollection<MazeItem> MazeItems { get; } = new();
        
        public MazesViewModel(IMazeService mazeService)
        {
            Title = "Mazes";
            this.mazeService = mazeService;
            _ = GetMazesAsync();
        }

        [ObservableProperty]
        string loadStatus = "No mazes found";

        [ObservableProperty]
        bool isRefreshing;

        [RelayCommandAttribute]
        async Task GetMazesAsync()
        {
            if (IsBusy)
                return;

            LoadStatus = "Loading mazes...";

            try
            {
                IsBusy = true;
                var mazeItems = await mazeService.GetMazeItems(true);

                if (MazeItems.Count != 0)
                    MazeItems.Clear();

                foreach (MazeItem item in mazeItems) 
                {
                    MazeItems.Add(item);
                }
            }
            catch (Exception ex)
            {
                await Shell.Current.DisplayAlert("Error", 
                    $"Unable to load mazes: {ex.Message}", "OK");
            }
            finally
            {
                IsBusy = false;
                IsRefreshing = false;
            }

            LoadStatus = MazeItems.Count == 0 ? "No mazes found" : "";

        }
        [RelayCommandAttribute]
        async Task GoToDesignAsync(MazeItem mazeItem)
        {
            if (mazeItem is null)
                return;

            if (IsBusy)
                return;

            IsBusy = true;

            await Shell.Current.GoToAsync($"{nameof(MazePage)}", true,
                new Dictionary<string, object>
                {
                    {"MazeItem", mazeItem }
                });

            IsBusy = false;
        }
    }
}
