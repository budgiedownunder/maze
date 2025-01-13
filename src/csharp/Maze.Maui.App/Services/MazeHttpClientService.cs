using Maze.Maui.App.Models;
using System.Net.Http.Json;
using System.Text;
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

            throw new JsonException("Expected a JSON string for 'definition' field");
        }

        public override void Write(Utf8JsonWriter writer, Api.Maze value, JsonSerializerOptions options)
        {
            string jsonMaze = value.ToJson();
            Dictionary<string, object>? fields = JsonSerializer.Deserialize<Dictionary<string, object>>(jsonMaze);

            if (fields is null || !fields.TryGetValue("definition", out var definitionValue))
            {
                writer.WriteStringValue("{\"grid\":[]}\"");
                return;
            }

            if (definitionValue is JsonElement jsonElement)
                jsonElement.WriteTo(writer);
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
                var options = new JsonSerializerOptions();
                options.Converters.Add(new DefinitionConverter());

                _mazeItems = await response.Content.ReadFromJsonAsync<List<Models.MazeItem>>(options) ?? new();
            }

            return _mazeItems;
        }

        public async Task<Models.MazeItem?> GetMazeItem(string id)
        {
            if (id is null || id == "")
            {
                throw new Exception("Maze item or id is null or empty");
            }
            string url = GetIdUrlPath(id);
            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();
            return await ReadItem(response);
        }

        public async Task CreateMazeItem(Models.MazeItem item)
        {
            if (item is null)
            {
                throw new Exception("Maze item is null");
            }

            string url = $"{_rootUrl}/mazes/";
            string json = GetMazeItemJson(item);
            HttpContent content = new StringContent(json, Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync(url, content);
            response.EnsureSuccessStatusCode();

            item.ID = await ReadId(response);
        }

        public async Task UpdateMazeItem(Models.MazeItem item)
        {
            if (item is null || item.ID == "")
            {
                throw new Exception("Maze item or id is null or empty");
            }
            string url = GetIdUrlPath(item.ID);
            string json = GetMazeItemJson(item);
            HttpContent content = new StringContent(json, Encoding.UTF8, "application/json");
            var response = await _httpClient.PutAsync(url, content);
            response.EnsureSuccessStatusCode();
        }

        public async Task RenameMazeItem(Models.MazeItem item, string newName)
        {
            Models.MazeItem tempItem = new MazeItem
            {
                ID = item.ID,
                Name = newName,
                Definition = item.Definition,
            };
            await UpdateMazeItem(tempItem);
            item.Name = newName;
        }

        public async Task DeleteMazeItem(string id)
        {
            string url = GetIdUrlPath(id);
            var response = await _httpClient.DeleteAsync(url);
            response.EnsureSuccessStatusCode();
        }

        private string GetIdUrlPath(string id)
        {
            string idEncoded = Uri.EscapeDataString(id);
            return $"{_rootUrl}/mazes/{idEncoded}";
        }

        private string GetMazeItemJson(MazeItem item)
        {
            var options = new JsonSerializerOptions();
            options.Converters.Add(new DefinitionConverter());
            return JsonSerializer.Serialize(item, options);
        }

        private static string ReadStringField(Dictionary<string, object>? fields, string name, bool allowEmpty)
        {
            string value = "";

            if (fields is null || !fields.TryGetValue(name, out var idValue))
                throw new Exception($"'{name}' not found in response");

            if (idValue is JsonElement jsonElement && jsonElement.ValueKind == JsonValueKind.String)
                value = jsonElement.GetString() ?? "";

            if (!allowEmpty && value == "")
                throw new Exception($"'{name}' is blank or empty in response");

            return value;
        }

        private static async Task<string> ReadId(HttpResponseMessage response)
        {
            string jsonResponse = await response.Content.ReadAsStringAsync();
            Dictionary<string, object>? fields = JsonSerializer.Deserialize<Dictionary<string, object>>(jsonResponse);
            return ReadStringField(fields, "id", false);
        }

        private static async Task<MazeItem> ReadItem(HttpResponseMessage response)
        {
            string jsonResponse = await response.Content.ReadAsStringAsync();
            Dictionary<string, object>? fields = JsonSerializer.Deserialize<Dictionary<string, object>>(jsonResponse);

            MazeItem item = new MazeItem
            {
                ID = ReadStringField(fields, "id", false),
                Name = ReadStringField(fields, "name", false),
                Definition = new Api.Maze(0, 0)
            };
            item.Definition.FromJson(jsonResponse);
            return item;
        }
    }
}