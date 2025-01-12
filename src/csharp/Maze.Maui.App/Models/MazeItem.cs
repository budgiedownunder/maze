using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using Maze.Api;

namespace Maze.Maui.App.Models
{
    public class MazeItem : INotifyPropertyChanged
    {
        private string name = "";
        private Api.Maze? definition;


        [JsonPropertyName("id")]
        public string ID { get; set; } = "";

        [JsonPropertyName("name")]
        public string Name { 
            get => name;
            set
            {
                if (name != value)
                {
                    name = value;
                    OnPropertyChanged(nameof(Name));
                }
            } 
        }

        [JsonPropertyName("definition")]
        public Api.Maze? Definition
        {
            get => definition;
            set
            {
                if (definition != value)
                {
                    definition = value;
                    OnPropertyChanged(nameof(Definition));
                    OnPropertyChanged(nameof(DimensionsSummary));
                }
            }
        }

        [JsonIgnore]
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
       
        public event PropertyChangedEventHandler? PropertyChanged;
        protected virtual void OnPropertyChanged(string propertyName)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
