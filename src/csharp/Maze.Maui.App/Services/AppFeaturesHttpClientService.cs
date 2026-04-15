using System.Net.Http.Json;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Represents an HTTP client service for retrieving server-controlled feature flags.
    /// </summary>
    public class AppFeaturesHttpClientService : IAppFeaturesService
    {
        private const double REQUEST_TIMEOUT_SECONDS = 30.0;
        private readonly HttpClient _httpClient;

        /// <inheritdoc/>
        public AppFeatures Features { get; private set; } = new AppFeatures();

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="configurationService">Injected configuration service</param>
        public AppFeaturesHttpClientService(ConfigurationService configurationService)
        {
            var handler = new HttpClientHandler();
            if (configurationService.DisableStrictTLSCertificateValidation)
                handler.ServerCertificateCustomValidationCallback = (_, _, _, _) => true;
            _httpClient = new HttpClient(handler)
            {
                BaseAddress = new Uri(configurationService.ApiRootUri),
                Timeout = TimeSpan.FromSeconds(REQUEST_TIMEOUT_SECONDS),
            };
        }

        /// <inheritdoc/>
        public async Task RefreshAsync()
        {
            try
            {
                var response = await _httpClient.GetAsync("features");
                if (!response.IsSuccessStatusCode) return;
                var features = await response.Content.ReadFromJsonAsync<AppFeatures>();
                if (features != null) Features = features;
            }
            catch
            {
                // fail-open: keep current value (defaults)
            }
        }
    }
}
