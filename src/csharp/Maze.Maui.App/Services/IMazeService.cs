using System.Threading.Tasks;
using System.Collections.Generic;
using Maze.Maui.App.Models;
using Microsoft.Maui.Controls;
using System.Xml.Linq;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Represents a maze service interface
    /// </summary>
    public interface IMazeService
    {
        /// <summary>
        /// Loads the current list of maze items
        /// </summary>
        /// <param name="includeDefinitions">Include the maze definitions?</param>
        /// <returns>A task that contains the list of maze items. Will throw an exception if the items could not be loaded.</returns>
        public Task<List<Models.MazeItem>> GetMazeItems(bool includeDefinitions);
        /// <summary>
        /// Creates a new maze item and assigns the allocated `ID` to it
        /// </summary>
        /// <param name="item">Maze item to create</param>
        /// <returns>A task. If successful, the allocated `ID` is set within the maze item object supplied. If unsuccessful, an exception will be thrown.</returns>
        public Task CreateMazeItem(Models.MazeItem item);
        /// <summary>
        /// Loads a maze item based on its `ID`
        /// </summary>
        /// <param name="id">Maze item `ID`</param>
        /// <returns>A task containing the loaded maze item. Will throw an exception if the item could not be loaded.</returns>
        public Task<Models.MazeItem?> GetMazeItem(string id);
        /// <summary>
        /// Updates a maze item
        /// </summary>
        /// <param name="item">Maze item to update</param>
        /// <returns>A task. Will throw an exception if the item could not be updated.</returns>
        public Task UpdateMazeItem(Models.MazeItem item);
        /// <summary>
        /// Renames a maze item
        /// </summary>
        /// <param name="item">Maze item to rename</param>
        /// <param name="newName">New name</param>
        /// <returns>A task. If successful, the new name is set within the maze item object supplied. If unsuccessful, an exception will be thrown.</returns>
        public Task RenameMazeItem(Models.MazeItem item, string newName);
        /// <summary>
        /// Deletes a maze item based on its `ID`
        /// </summary>
        /// <param name="id">Maze item `ID`</param>
        /// <returns>A task. Will throw an exception if the item could not be deleted.</returns>
        public Task DeleteMazeItem(string id);
    }
}
