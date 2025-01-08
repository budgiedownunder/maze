using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Maze.Api;

namespace Maze.Maui.App.Models
{
    public class MazeItem
    {
        public string ID { get; set; } = "";
        public string Name { get; set; } = "";
        public Api.Maze? Definition { get; set; }

        public string DimensionsSummary
        {
            get {
                if (Definition is not null)
                {
                    string rowsLabel = Definition.RowCount == 1 ? "row" : "rows";
                    string colsLabel = Definition.ColCount == 1 ? "column" : "columns";
                    return $"{Definition.RowCount} {rowsLabel} x {Definition.ColCount} {colsLabel}";
                }
                return "Definition not available";
            }
        }

        public MazeItem() { 
        }    
    }
}
