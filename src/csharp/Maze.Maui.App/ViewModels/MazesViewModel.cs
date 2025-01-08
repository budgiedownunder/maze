using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
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
    }
}
