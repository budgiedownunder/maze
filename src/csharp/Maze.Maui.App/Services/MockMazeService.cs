using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Represents a mock service for managing the load, save and deletion of mazes. Mazes are stored in memory, 
    /// with an empty list at start-up.
    /// </summary>
    public class MockMazeService : IMazeService
    {
        // Private properties
        List<Models.MazeItem> _mazeItems = new();

        /// <summary>
        /// Constructor
        /// </summary>
        public MockMazeService()
        {
        }
        /// <summary>
        /// Loads the current list of maze items
        /// </summary>
        /// <param name="includeDefinitions">Include the maze definitions?</param>
        /// <returns>A task that contains the list of maze items. Will throw an exception if the items could not be loaded.</returns>
        public async Task<List<Models.MazeItem>> GetMazeItems(bool includeDefinitions)
        {
            // Dummy await, to keep the compiler happy for this async method
            await Task.CompletedTask; 

            if (includeDefinitions)
            {
                return _mazeItems;
            }

            List<Models.MazeItem> emptyItems = new();
            foreach (MazeItem item in _mazeItems)
            {
                emptyItems.Add(new Models.MazeItem()
                {
                    ID = item.ID,
                    Name = item.Name,
                });
            }
            return emptyItems;
        }
        /// <summary>
        /// Creates a new maze item and assigns the name as the `ID` to it
        /// </summary>
        /// <param name="item">Maze item to create</param>
        /// <returns>A task. If successful, the allocated `ID` is set within the maze item object supplied. If unsuccessful, an exception will be thrown.</returns>
        public async Task CreateMazeItem(Models.MazeItem item)
        {
            // Dummy await, to keep the compiler happy for this async method
            await Task.CompletedTask;

            if (item is null)
            {
                throw new Exception("Maze item is null");
            }

            item.ID = item.Name;

            _mazeItems.Add(new MazeItem()
            {
                ID= item.ID,
                Name = item.Name,   
                Definition = item.Definition,
            });

            SortItems();
        }
        /// <summary>
        /// Loads a maze item based on its `ID`
        /// </summary>
        /// <param name="id">Maze item `ID`</param>
        /// <returns>A task containing the loaded maze item. Will throw an exception if the item could not be loaded.</returns>
        public async Task<Models.MazeItem?> GetMazeItem(string id)
        {
            // Dummy await, to keep the compiler happy for this async method
            await Task.CompletedTask;

            if (id is null || id == "")
            {
                throw new Exception("Maze item or id is null or empty");
            }

            return FindItem(id);

        }
        /// <summary>
        /// Updates a maze item
        /// </summary>
        /// <param name="item">Maze item to update</param>
        /// <returns>A task. Will throw an exception if the item could not be updated.</returns>
        public async Task UpdateMazeItem(Models.MazeItem item)
        {
            // Dummy await, to keep the compiler happy for this async method
            await Task.CompletedTask;

            if (item is null || item.ID == "")
            {
                throw new Exception("Maze item or id is null or empty");
            }

            MazeItem? storedItem = FindItem(item.ID);

            if (storedItem is not null)
            {
                storedItem.Name = item.Name;
                storedItem.Definition = item.Definition;
            }

            SortItems();
        }
        /// <summary>
        /// Renames a maze item
        /// </summary>
        /// <param name="item">Maze item to rename</param>
        /// <param name="newName">New name</param>
        /// <returns>A task. If successful, the new name is set within the maze item object supplied. If unsuccessful, an exception will be thrown.</returns>
        public async Task RenameMazeItem(Models.MazeItem item, string newName)
        {
            // Dummy await, to keep the compiler happy for this async method
            await Task.CompletedTask;

            if (item is null || item.ID == "")
            {
                throw new Exception("Maze item or id is null or empty");
            }

            MazeItem? storedItem = FindItem(item.ID);
            if (storedItem is not null)
            {
                storedItem.Name = newName;
            }

            item.Name = newName;

            SortItems();
        }
        /// <summary>
        /// Deletes a maze item based on its `ID`
        /// </summary>
        /// <param name="id">Maze item `ID`</param>
        /// <returns>A task. Will throw an exception if the item could not be deleted.</returns>
        public async Task DeleteMazeItem(string id)
        {
            // Dummy await, to keep the compiler happy for this async method
            await Task.CompletedTask;

            RemoveItem(id);
        }
        /// <summary>
        /// Locates an maze item by its ID in the stored items
        /// </summary>
        /// <param name="id">Maze item ID</param>
        /// <returns>Maze item. If unsuccessful, an exception will be thrown.</returns>
        private MazeItem? FindItem(string id)
        {
            MazeItem? item = _mazeItems.FirstOrDefault(item => item.ID == id);
            if (item is null)
            {
                throw new Exception($"Maze item with id '{id} was not found");
            }
            return item;
        }
        /// <summary>
        /// Locates an maze item by its ID within the stored items
        /// </summary>
        /// <param name="id">Maze item ID</param>
        /// <returns>Nothing</returns>
        private void RemoveItem(string id)
        {
            _mazeItems.RemoveAll(item => item.ID == id);
        }
        /// <summary>
        /// Sorts the stored maze items
        /// </summary>
        /// <returns>Nothing</returns>
        private void SortItems()
        {
            _mazeItems = _mazeItems.OrderBy(i => i.Name).ToList();
        }
    }
}