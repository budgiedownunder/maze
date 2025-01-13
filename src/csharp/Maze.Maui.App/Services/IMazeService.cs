using System.Threading.Tasks;
using System.Collections.Generic;
using Maze.Maui.App.Models;
using Microsoft.Maui.Controls;
using System.Xml.Linq;

namespace Maze.Maui.App.Services
{
    public interface IMazeService
    {
        public Task<List<Models.MazeItem>> GetMazeItems(bool loadDefinitions);

        public Task CreateMazeItem(Models.MazeItem item);

        public Task<Models.MazeItem?> GetMazeItem(string id);

        public Task UpdateMazeItem(Models.MazeItem item);

        public Task RenameMazeItem(Models.MazeItem item, string newName);

        public Task DeleteMazeItem(string id);
    }
}
