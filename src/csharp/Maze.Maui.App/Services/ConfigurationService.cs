using System.Text.Json;

namespace Maze.Maui.App.Services
{
    // Strongly typed settings model
    public class ConfigModel
    {
        /// <summary>
        /// Represents the root URI to be used. This will be the setting that is defined in the configuration
        /// (if one exists), else the development-based default for the platform
        /// </summary>
        /// <returns>Root URI</returns>
        public string ApiRootUri { get; set; } = GetDefaultDevelopmentApiRootUri();
        /// <summary>
        /// Returns the default (development-based) API root URI to be used for the platform
        /// </summary>
        /// <returns>API root URI</returns>
        private static string GetDefaultDevelopmentApiRootUri()
        {
#if WINDOWS
            string rootUri = "http://localhost:8080/api/v1/";
#elif ANDROID
            string rootUri = "http://10.0.2.2:8080/api/v1/";
#elif IOS
            string rootUri = "http://localhost:8080/api/v1/";
#else
            string rootUri = "http://localhost:8080/api/v1/";
#endif
            return rootUri;
        }
    }
    /// <summary>
    /// Represents a service for managing configuratuon settings
    /// </summary>
    public class ConfigurationService
    {
        // Private properties
        private ConfigModel _settings = new ConfigModel();
        /// <summary>
        /// Represents the root URI to be used. This will be the setting that is defined in the configuration
        /// (if one exists), else the development-based default for the platform
        /// </summary>
        /// <returns>Root URI</returns>
        public string ApiRootUri { get => _settings.ApiRootUri; }
        /// <summary>
        /// Constructor
        /// </summary>
        public ConfigurationService()
        {
            Task.Run(async () => await LoadSettingsAsync()).Wait();
        }
        /// <summary>
        /// Loads the configuration settings from the application settings file. Any missing settings adopt their default value and any unrecognised
        /// settings are ignored.
        /// </summary>
        /// <returns>Task</returns>
        private async Task LoadSettingsAsync()
        {
            try
            {
                using Stream stream = await FileSystem.OpenAppPackageFileAsync("appsettings.json");
                using StreamReader reader = new StreamReader(stream);
                string json = await reader.ReadToEndAsync();

                var settings = JsonSerializer.Deserialize<ConfigModel>(json);
                if (settings != null)
                    _settings = settings;
            }
            catch { } 
        }
    }
}
