using System.Threading.Tasks;
using System.Collections.Generic;

namespace Maze.Maui.App.Services
{
    public interface IMazeService
    {
        public Task<List<Models.MazeItem>> GetMazeItems(bool loadDefinitions);
    }
}
