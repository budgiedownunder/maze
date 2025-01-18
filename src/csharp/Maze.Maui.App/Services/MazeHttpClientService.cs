using Maze.Maui.App.Models;
using System.Net;
using System.Net.Http.Json;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Represents a maze item definition converter for reading/writing maze item definitions to/from a JSON reader/writer
    /// </summary>
    public class DefinitionConverter : JsonConverter<Maze.Api.Maze>
    {
        /// <summary>
        /// Reads a maze definition from a JSON reader
        /// </summary>
        /// <param name="reader">JSON reader</param>
        /// <param name="typeToConvert">Type to convert</param>
        /// <param name="options">Serializer options</param>
        /// <returns>Maze item or null. Will throw an exception if the definition could not be read.</returns>
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
        /// <summary>
        /// Writes a maze definition to a JSON writer
        /// </summary>
        /// <param name="writer">JSON writer</param>
        /// <param name="value">Maze definition to write</param>
        /// <param name="options">Serializer options</param>
        /// <returns>Nothing. Will throw an exception if the definition could not be written.</returns>
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
    /// <summary>
    /// Represents a Http client service for managing the load, save and deletion of mazes
    /// </summary>
    public class MazeHttpClientService : IMazeService
    {
        // Private definitions
        private const string AUTH_COOKIE_NAME = "AuthToken";
        private const string AUTH_COOKIE_VALUE = "0595C1D2-6341-44BF-BB34-C2E350A8AD72";
        private const double REQUEST_TIMEOUT_SECONDS = 30.0;

        // Private properties
        ConfigurationService _configurationService;
        HttpClient _httpClient;
        List<Models.MazeItem> _mazeItems = new();
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="configurationService">Injected configuration service</param>
        public MazeHttpClientService(ConfigurationService configurationService)
        {
            _configurationService = configurationService;
            _httpClient = CreateHttpClient();
        }
        /// <summary>
        /// Creates and initializes an HTTP client
        /// </summary>
        /// <returns>HTTP client</returns>
        private HttpClient CreateHttpClient()
        {
            string apiRootUri = _configurationService.ApiRootUri;
            string cookieValue = $"{AUTH_COOKIE_NAME}={AUTH_COOKIE_VALUE}; Path=/; HttpOnly; Secure; SameSite=Lax";
            CookieContainer cookieContainer = new CookieContainer();

            cookieContainer.SetCookies(new Uri(apiRootUri), cookieValue);

            HttpClientHandler handler = new HttpClientHandler
            {
                CookieContainer = cookieContainer
            };

            if (_configurationService.DisableStrictTLSCertificateValidation)
                handler.ServerCertificateCustomValidationCallback = (message, cert, chain, sslPolicyErrors) => true;

            HttpClient httpClient = new HttpClient(handler)
            {
                BaseAddress = new Uri(apiRootUri),
                Timeout = TimeSpan.FromSeconds(REQUEST_TIMEOUT_SECONDS)
            };

            return httpClient;
        }
        /// <summary>
        /// Loads the current list of maze items
        /// </summary>
        /// <param name="includeDefinitions">Include the maze definitions?</param>
        /// <returns>A task that contains the list of maze items. Will throw an exception if the items could not be loaded.</returns>
        public async Task<List<Models.MazeItem>> GetMazeItems(bool includeDefinitions)
        {
            var uri = $"mazes?includeDefinitions={(includeDefinitions ? "true" : "false")}";
            var response = await _httpClient.GetAsync(uri);

            if (response.IsSuccessStatusCode)
            {
                var options = new JsonSerializerOptions();
                options.Converters.Add(new DefinitionConverter());

                _mazeItems = await response.Content.ReadFromJsonAsync<List<Models.MazeItem>>(options) ?? new();
            }
            
            return _mazeItems;
        }
        /// <summary>
        /// Creates a new maze item and assigns the allocated `ID` to it
        /// </summary>
        /// <param name="item">Maze item to create</param>
        /// <returns>A task. If successful, the allocated `ID` is set within the maze item object supplied. If unsuccessful, an exception will be thrown.</returns>
        public async Task CreateMazeItem(Models.MazeItem item)
        {
            if (item is null)
            {
                throw new Exception("Maze item is null");
            }

            string url = $"mazes/";
            string json = GetMazeItemJson(item);
            HttpContent content = new StringContent(json, Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync(url, content);
            response.EnsureSuccessStatusCode();
            item.ID = await ReadId(response);
        }
        /// <summary>
        /// Loads a maze item based on its `ID`
        /// </summary>
        /// <param name="id">Maze item `ID`</param>
        /// <returns>A task containing the loaded maze item. Will throw an exception if the item could not be loaded.</returns>
        public async Task<Models.MazeItem?> GetMazeItem(string id)
        {
            if (id is null || id == "")
            {
                throw new Exception("Maze item or id is null or empty");
            }
            string url = GetIdUriPath(id);
            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();
            return await ReadItem(response);
        }
        /// <summary>
        /// Updates a maze item
        /// </summary>
        /// <param name="item">Maze item to update</param>
        /// <returns>A task. Will throw an exception if the item could not be updated.</returns>
        public async Task UpdateMazeItem(Models.MazeItem item)
        {
            if (item is null || item.ID == "")
            {
                throw new Exception("Maze item or id is null or empty");
            }
            string url = GetIdUriPath(item.ID);
            string json = GetMazeItemJson(item);
            HttpContent content = new StringContent(json, Encoding.UTF8, "application/json");
            var response = await _httpClient.PutAsync(url, content);
            response.EnsureSuccessStatusCode();
        }
        /// <summary>
        /// Renames a maze item
        /// </summary>
        /// <param name="item">Maze item to rename</param>
        /// <param name="newName">New name</param>
        /// <returns>A task. If successful, the new name is set within the maze item object supplied. If unsuccessful, an exception will be thrown.</returns>
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
        /// <summary>
        /// Deletes a maze item based on its `ID`
        /// </summary>
        /// <param name="id">Maze item `ID`</param>
        /// <returns>A task. Will throw an exception if the item could not be deleted.</returns>
        public async Task DeleteMazeItem(string id)
        {
            string url = GetIdUriPath(id);
            var response = await _httpClient.DeleteAsync(url);
            response.EnsureSuccessStatusCode();
        }
        /// <summary>
        /// Constructs the uri path for a given id`
        /// </summary>
        /// <param name="id">Maze item `ID`</param>
        /// <returns>Uri path</returns>
        private string GetIdUriPath(string id)
        {
            string idEncoded = Uri.EscapeDataString(id);
            return $"mazes/{idEncoded}";
        }
        /// <summary>
        /// Converts a maze item to JSON`
        /// </summary>
        /// <param name="item">Maze item</param>
        /// <returns>JSON string</returns>
        private string GetMazeItemJson(MazeItem item)
        {
            var options = new JsonSerializerOptions();
            options.Converters.Add(new DefinitionConverter());
            return JsonSerializer.Serialize(item, options);
        }
        /// <summary>
        /// Reads an `id` field from a HTTP response containing JSON
        /// </summary>
        /// <param name="response">Response</param>
        /// <returns>Task containin the id. If unsuccessful, an exception will be thrown.</returns>
        private static async Task<string> ReadId(HttpResponseMessage response)
        {
            string jsonResponse = await response.Content.ReadAsStringAsync();
            Dictionary<string, object>? fields = JsonSerializer.Deserialize<Dictionary<string, object>>(jsonResponse);
            return ReadStringField(fields, "id", false);
        }
        /// <summary>
        /// Reads a maze item from a HTTP response containing JSON
        /// </summary>
        /// <param name="response">Response</param>
        /// <returns>Task containin the maze item. If unsuccessful, an exception will be thrown.</returns>
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
        /// <summary>
        /// Reads a string field from a dictionary containing JSON elements
        /// </summary>
        /// <param name="fields">Dictionary</param>
        /// <param name="name">Name of field to read</param>
        /// <param name="allowEmpty">Allow an empty/blank value?</param>
        /// <returns>String value. If unsuccessful, an exception will be thrown.</returns>
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
    }
}