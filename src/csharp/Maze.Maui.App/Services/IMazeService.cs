using System.Threading.Tasks;
using System.Collections.Generic;
using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    public interface IMazeService
    {
        public Task<List<Models.MazeItem>> GetMazeItems(bool loadDefinitions);
        public Task DeleteMazeItem(Models.MazeItem item);
    }
}
