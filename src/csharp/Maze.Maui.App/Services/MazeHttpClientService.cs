using Maze.Maui.App.Models;
using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Maze.Maui.App.Services
{
    public class DefinitionConverter : JsonConverter<Maze.Api.Maze>
    {
        public override Maze.Api.Maze? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            if (reader.TokenType == JsonTokenType.String)
            {
                string jsonString = reader.GetString() ?? string.Empty;
                Api.Maze maze = new Api.Maze(0, 0);
                maze.FromJson(jsonString);
                return maze;
            }

            throw new JsonException("Expected a JSON string for Definition");
        }

        public override void Write(Utf8JsonWriter writer, Api.Maze value, JsonSerializerOptions options)
        {
            writer.WriteStringValue(value.ToString());
        }
    }
    public class MazeHttpClientService : IMazeService
    {
        HttpClient _httpClient;
        List<Models.MazeItem> _mazeItems = new();

        // TO DO - pass root in constructor
#if WINDOWS
        string _rootUrl = "http://localhost:8080/api/v1";
#elif ANDROID
        string _rootUrl = "http://10.0.2.2:8080/api/v1";
#elif IOS
        string _rootUrl = "http://localhost:8080/api/v1";
#else
        string _rootUrl = "http://localhost:8080/api/v1";
#endif

        public MazeHttpClientService()
        {
            _httpClient = new HttpClient();
            _httpClient.Timeout = TimeSpan.FromSeconds(30);

        }
        public async Task<List<Models.MazeItem>> GetMazeItems(bool includeDefinitions)
        {
            var url = $"{_rootUrl}/mazes?includeDefinitions={(includeDefinitions ? "true" : "false")}";
            var response = await _httpClient.GetAsync(url);

            if (response.IsSuccessStatusCode)
            {
                var options = new JsonSerializerOptions
                {
                    PropertyNameCaseInsensitive = true
                };

                options.Converters.Add(new DefinitionConverter());

                _mazeItems = await response.Content.ReadFromJsonAsync<List<Models.MazeItem>>(options) ?? new ();
            }
            return _mazeItems;
        }
    }
}