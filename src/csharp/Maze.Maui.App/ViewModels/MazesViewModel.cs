using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.ComponentModel;
using Maze.Maui.App.Extensions;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents a mazes view model
    /// </summary>
    public partial class MazesViewModel : BaseViewModel
    {
        public ObservableCollection<MazeItem> MazeItems { get; } = new();
        // Private properties
        IMazeService _mazeService;
        IDialogService _dialogService;
        private bool _dataLoaded = false;
        public bool IsDataLoaded => _dataLoaded;
        public void InvalidateData()
        {
            _dataLoaded = false;
            MazeItems.Clear();
            LoadStatus = "No mazes found";
        }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="mazeService">Injected maze service</param>
        public MazesViewModel(IDialogService dialogService, IMazeService mazeService)
        {
            Title = "Mazes";
            _mazeService = mazeService;
            _dialogService = dialogService;
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
                List<MazeItem> items = await _mazeService.GetMazeItems(true);
                DisplayItems(items.OrderBy(i => i.Name).ToList());
                _dataLoaded = true;
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert("Error", $"Unable to load mazes\n\n{ex.Message.CapitalizeFirst()}", "OK");
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
        /// <param name="item">Maze item</param>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        async Task GoToDesignAsync(MazeItem item)
        {
            if (item is null)
                return;

            if (IsBusy)
                return;

            IsBusy = true;
            try
            {
                await Shell.Current.GoToAsync($"{nameof(MazePage)}", true,
                    new Dictionary<string, object>
                    {
                        {"MazeItem", item }
                    });
                // Hold IsBusy for a further 500ms after navigation completes to block any
                // buffered second taps (e.g. double-click) that arrive after GoToAsync returns
                await Task.Delay(500);
            }
            finally
            {
                IsBusy = false;
            }
        }
        /// <summary>
        /// Navigates to the maze game page for the given maze item
        /// </summary>
        /// <param name="item">Maze item</param>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        async Task GoToPlayAsync(MazeItem item)
        {
            if (item?.Definition is null)
                return;

            if (IsBusy)
                return;

            try
            {
                item.Definition.Solve();
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert("MAZE", $"Cannot play maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
                return;
            }

            IsBusy = true;
            try
            {
                await Shell.Current.GoToAsync($"{nameof(Views.MazeGamePage)}", true,
                    new Dictionary<string, object>
                    {
                        {"MazeItem", item }
                    });
                await Task.Delay(500);
            }
            finally
            {
                IsBusy = false;
            }
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
        /// <param name="item">Maze item</param>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        async Task RenameAsync(MazeItem item)
        {
            if (item is null)
                return;

            if (IsBusy)
                return;

            bool finished = false;
            string initialName = item.Name;

            while (!finished)
            {
                string? name = await _dialogService.DisplayPrompt("Rename Maze", "Name", "Name", 
                                            "OK", "Cancel", "Enter new maze name",
                                            keyboard: Keyboard.Text, initialValue: initialName,
                                            allowEmpty: false, trimResult: true);

                if (name is not null && name != item.Name)
                {
                    finished = await RenameMaze(item, name);
                    if (!finished)
                        initialName = name;
                }
                else
                    finished = true;
            }
        }
        /// <summary>
        /// Handles duplication of the given maze item
        /// </summary>
        /// <param name="item">Maze item</param>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        async Task DuplicateAsync(MazeItem item)
        {
            if (item is null)
                return;

            if (IsBusy)
                return;

            bool finished = false;
            string initialName = $"Copy of {item.Name}";

            while (!finished)
            {
                string? name = await _dialogService.DisplayPrompt("Duplicate Maze", "Name", "Name",
                                            "OK", "Cancel", "Enter new maze name",
                                            keyboard: Keyboard.Text, initialValue: initialName,
                                            allowEmpty: false, trimResult: true);
                if (name is not null)
                {
                    finished = await DuplicateMaze(item, name);
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
        /// <param name="item">Maze item</param>
        /// <returns>Task</returns>
        [RelayCommandAttribute]
        async Task DeleteAsync(MazeItem item)
        {
            if (item is null)
                return;

            if (IsBusy)
                return;

            if (await _dialogService.ShowConfirmation(
                "Delete Maze",
                $"Are you sure you want to delete '{item.Name}'?",
                "Yes",
                "No",
                isDestructive: true))
            {
                bool deleted = await DeleteMaze(item);
                if (deleted)
                    RemoveItem(item);
            }
        }
        /// <summary>
        /// Renames a maze item to the given name
        /// </summary>
        /// <param name="item">Maze item</param>
        /// <param name="newName">New name</param>
        /// <returns>Task containing a boolean result</returns>
        async Task<bool> RenameMaze(MazeItem item, string newName)
        {
            if (IsBusy)
                return false;

            if (NameExists(newName))
            {
                await _dialogService.ShowAlert("Error", $"The name '{newName}' is already in use.\n\nPlease choose another name.", "OK");
                return false;
            }

            bool renamed = false;

            try
            {
                IsBusy = true;
                await _mazeService.RenameMazeItem(item, newName);
                SortItems();
                renamed = true;
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert("Error", $"Failed to rename maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
            }
            finally
            {
                IsBusy = false;
            }
            return renamed;
        }
        /// <summary>
        /// Duplicates a maze item with the given name
        /// </summary>
        /// <param name="item">Maze item</param>
        /// <param name="newName">New name</param>
        /// <returns>Task containing a boolean result</returns>
        async Task<bool> DuplicateMaze(MazeItem item, string newName)
        {
            if (IsBusy)
                return false;

            if (NameExists(newName))
            {
                await _dialogService.ShowAlert("Error", $"The name '{newName}' is already in use.\n\nPlease choose another name.", "OK");
                return false;
            }

            bool duplicated = false;

            try
            {
                IsBusy = true;
                MazeItem duplicateItem = item.Duplicate();
                duplicateItem.ID = "";
                duplicateItem.Name = newName;
                await _mazeService.CreateMazeItem(duplicateItem);
                AddNewItem(duplicateItem);
                duplicated = true;
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert("Error", $"Failed to duplicate maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
            }
            finally
            {
                IsBusy = false;
            }
            return duplicated;
        }
        /// <summary>
        /// Deletes the given maze item
        /// </summary>
        /// <param name="item">Maze item</param>
        /// <returns>Task containing a boolean result</returns>
        async Task<bool> DeleteMaze(MazeItem item)
        {
            bool deleted = false;
            try
            {
                IsBusy = true;
                await _mazeService.DeleteMazeItem(item.ID);
                deleted = true;
            }
            catch (Exception ex)
            {
                await _dialogService.ShowAlert("Error", $"Failed to delete maze\n\n{ex.Message.CapitalizeFirst()}", "OK");
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
        /// Sorts the list of displayed mazes into ascending order by name
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
