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
        IDialogService dialogService;
        public ObservableCollection<MazeItem> MazeItems { get; } = new();

        public MazesViewModel(IMazeService mazeService, IDialogService dialogService)
        {
            Title = "Mazes";
            this.mazeService = mazeService;
            this.dialogService = dialogService;
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
                DisplayItems(await mazeService.GetMazeItems(true));
            }
            catch (Exception ex)
            {
                await dialogService.ShowAlert("Error", $"Unable to load mazes: {ex.Message}", "OK");
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

        [RelayCommandAttribute]
        async Task NewAsync()
        {
            await GoToDesignAsync(new MazeItem());
        }

        [RelayCommandAttribute]
        async Task RenameAsync(MazeItem mazeItem)
        {
            if (mazeItem is null)
                return;

            if (IsBusy)
                return;

            bool finished = false;
            string initialName = mazeItem.Name;

            while (!finished)
            {
                string? name = await dialogService.DisplayPrompt("Rename Maze", "Name", "Name", "OK", "Cancel", "Enter new maze name",
                                                   keyboard: Keyboard.Text, initialValue: initialName,
                                                   allowEmpty: false, trimResult: true);
                if (name is not null && name != mazeItem.Name)
                {
                    finished = await RenameMaze(mazeItem, name);
                    if (!finished)
                        initialName = name;
                }
                else
                    finished = true;
            }
        }

        [RelayCommandAttribute]
        async Task DeleteAsync(MazeItem mazeItem)
        {
            if (mazeItem is null)
                return;

            if (IsBusy)
                return;

            if (await dialogService.ShowConfirmation(
                "Delete Maze",
                $"Are you sure you want to delete '{mazeItem.Name}'?",
                "Yes",
                "No"
                ))
            {
                bool deleted = await DeleteMaze(mazeItem);
                if (deleted)
                    RemoveItem(mazeItem);
            }
        }

        async Task<bool> RenameMaze(MazeItem mazeItem, string newName)
        {
            if (IsBusy)
                return false;

            if (NameExists(newName))
            {
                await dialogService.ShowAlert("Error", $"The name '{newName}' is already in use.\n\nPlease choose another name.", "OK");
                return false;
            }

            bool renamed = false;

            try
            {
                IsBusy = true;
                await mazeService.RenameMazeItem(mazeItem, newName);
                renamed = true;
            }
            catch (Exception ex)
            {
                await dialogService.ShowAlert("Error", $"Failed to rename maze: {ex.Message}", "OK");
            }
            finally
            {
                IsBusy = false;
            }
            return renamed;
        }

        async Task<bool> DeleteMaze(MazeItem mazeItem)
        {
            bool deleted = false;
            if (IsBusy)
                return deleted;

            try
            {
                IsBusy = true;
                await mazeService.DeleteMazeItem(mazeItem.ID);
                deleted = true;
            }
            catch (Exception ex)
            {
                await dialogService.ShowAlert("Error", $"Failed to delete maze: {ex.Message}", "OK");
            }
            finally
            {
                IsBusy = false;
            }
            return deleted;
        }

        public void AddNewItem(MazeItem item)
        {
            MazeItems.Add(item);
            SortItems();
        }

        public void RemoveItem(MazeItem item)
        {
            MazeItems.Remove(item);
        }

        private void SortItems()
        {
            var sortedItems = MazeItems.OrderBy(i => i.Name).ToList();
            DisplayItems(sortedItems);
        }

        private void DisplayItems(List<MazeItem> items)
        {
            if (MazeItems.Count != 0)
                MazeItems.Clear();

            foreach (MazeItem item in items)
            {
                MazeItems.Add(item);
            }
        }

        private bool NameExists(string name)
        {
            string nameUpper = name.ToUpper();
            return MazeItems.Any(item => item.Name.ToUpper() == nameUpper);
        }
    }
}
