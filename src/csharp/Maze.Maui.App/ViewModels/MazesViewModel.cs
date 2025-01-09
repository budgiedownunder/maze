using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.Input;

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
            _ = GetMazeItemsAsync();
        }

        [RelayCommandAttribute]
        async Task GetMazeItemsAsync()
        {
            if (IsBusy)
                return;

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
            }
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
