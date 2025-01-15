using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents a mazes view model
    /// </summary>
    public partial class MazesViewModel : BaseViewModel
    {
        // Private properties
        IMazeService mazeService;
        IDialogService dialogService;
        public ObservableCollection<MazeItem> MazeItems { get; } = new();
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="mazeService">Injected maze service</param>
        public MazesViewModel(IDialogService dialogService, IMazeService mazeService)
        {
            Title = "Mazes";
            this.mazeService = mazeService;
            this.dialogService = dialogService;
            _ = GetMazesAsync();
        }
        /// <summary>
        /// Represents the load status
        /// </summary>
        /// <returns>String value</returns>
        [ObservableProperty]
        protected string loadStatus = "No mazes found";
        /// <summary>
        /// Indicates whether the view is currently refreshing
        /// </summary>
        /// <returns>Boolean value</returns>
        [ObservableProperty]
        protected bool isRefreshing;
        /// <summary>
        /// Loads the maze list
        /// </summary>
        /// <returns>Task</returns>
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
        /// <summary>
        /// Activates the maze (design) page for the given maze item
        /// </summary>
        /// <param name="mazeItem">Maze item</param>
        /// <returns>Task</returns>
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
        /// <summary>
        /// Activates the maze (design) page for a new maze item
        /// </summary>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        async Task NewAsync()
        {
            await GoToDesignAsync(new MazeItem());
        }
        /// <summary>
        /// Handles rename of the given maze item
        /// </summary>
        /// <param name="mazeItem">Maze item</param>
        /// <returns>Task</returns>
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
        /// <summary>
        /// Handles deletion of the given maze item
        /// </summary>
        /// <param name="mazeItem">Maze item</param>
        /// <returns>Task</returns>
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
        /// <summary>
        /// Renames a maze item to the given name
        /// </summary>
        /// <param name="mazeItem">Maze item</param>
        /// <param name="newName">New name</param>
        /// <returns>Task containing a boolean result</returns>
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
        /// <summary>
        /// Deletes the given maze item
        /// </summary>
        /// <param name="mazeItem">Maze item</param>
        /// <returns>Task containing a boolean result</returns>
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
        /// <summary>
        /// Adds a new maze item to the list of mazes
        /// </summary>
        /// <param name="item">Maze item</param>
        /// <returns>Nothing</returns>
        public void AddNewItem(MazeItem item)
        {
            MazeItems.Add(item);
            SortItems();
        }
        /// <summary>
        /// Removes a maze item from the list of mazes
        /// </summary>
        /// <param name="item">Maze item</param>
        /// <returns>Nothing</returns>
        public void RemoveItem(MazeItem item)
        {
            MazeItems.Remove(item);
        }
        /// <summary>
        /// Sorts the list of mazes into ascending name order
        /// </summary>
        /// <returns>Nothing</returns>
        private void SortItems()
        {
            var sortedItems = MazeItems.OrderBy(i => i.Name).ToList();
            DisplayItems(sortedItems);
        }
        /// <summary>
        /// Updates the display with the given list of maze items
        /// </summary>
        /// <param name="items">Maze items</param>
        /// <returns>Nothing</returns>
        private void DisplayItems(List<MazeItem> items)
        {
            if (MazeItems.Count != 0)
                MazeItems.Clear();

            foreach (MazeItem item in items)
            {
                MazeItems.Add(item);
            }
        }
        /// <summary>
        /// Checks whether a given name exists (case-insensitive)
        /// </summary>
        /// <param name="name">Name to check</param>
        /// <returns>Boolean</returns>
        private bool NameExists(string name)
        {
            string nameUpper = name.ToUpper();
            return MazeItems.Any(item => item.Name.ToUpper() == nameUpper);
        }
    }
}
