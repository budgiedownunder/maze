using System.ComponentModel;
using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// The `MazeItem` class represents a stored maze
    /// </summary>
    public class MazeItem : INotifyPropertyChanged
    {
        // Private properties
        private string name = "";
        private Api.Maze? definition;

        /// <summary>
        /// The ID of the maze item within the store
        /// </summary>
        /// <returns>Maze ID</returns>
        [JsonPropertyName("id")]
        public string ID { get; set; } = "";
        /// <summary>
        /// The name of the maze within the store
        /// </summary>
        /// <returns>Maze name</returns>
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
        /// <summary>
        /// The maze definition
        /// </summary>
        /// <returns>Maze definition</returns>
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
        /// <summary>
        /// Provides a textual summary of the maze's dimensions
        /// </summary>
        /// <returns>Dimensions summary</returns>
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
        /// <summary>
        /// Constructor
        /// </summary>
        public MazeItem() { 
        }
        /// <summary>
        /// Registered property changed event handler
        /// </summary>
        /// <returns>Event handler</returns>
        public event PropertyChangedEventHandler? PropertyChanged;
        /// <summary>
        /// Triggers a property changed event on any subscribed handlers
        /// </summary>
        /// <param name="propertyName">Property name</param>
        protected virtual void OnPropertyChanged(string propertyName)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
        /// <summary>
        /// Duplicates the item
        /// </summary>
        /// <returns>Duplicated item</returns>
        public MazeItem Duplicate()
        {
            MazeItem duplicateItem = new MazeItem()
            {
                ID = this.ID,
                Name = this.Name,
                Definition = new Api.Maze(0, 0),
            };

            if (this.Definition is not null)
            {
                string definitionJson = this.Definition.ToJson();
                duplicateItem.Definition.FromJson(definitionJson);
            }

            return duplicateItem;
        }
    }
}
