namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Factory for creating HTTP clients configured for the Maze Web API.
    /// Centralises base address, timeout, and TLS certificate validation settings
    /// sourced from <see cref="ConfigurationService"/>.
    /// </summary>
    internal static class ApiHttpClientFactory
    {
        private const double REQUEST_TIMEOUT_SECONDS = 30.0;

        /// <summary>
        /// Creates an <see cref="HttpClientHandler"/> with TLS certificate validation configured
        /// according to <paramref name="configurationService"/>.
        /// </summary>
        /// <param name="configurationService">Configuration service</param>
        /// <returns>Configured handler</returns>
        internal static HttpClientHandler CreateHandler(ConfigurationService configurationService)
        {
            var handler = new HttpClientHandler();
            if (configurationService.DisableStrictTLSCertificateValidation)
                handler.ServerCertificateCustomValidationCallback = (_, _, _, _) => true;
            return handler;
        }

        /// <summary>
        /// Creates an <see cref="HttpClient"/> configured with the API base address and timeout.
        /// If <paramref name="outerHandler"/> is supplied it is used as the message handler;
        /// otherwise a new handler is created via <see cref="CreateHandler"/>.
        /// </summary>
        /// <param name="configurationService">Configuration service</param>
        /// <param name="outerHandler">
        /// Optional outer message handler (e.g. a delegating handler wrapping a configured inner handler).
        /// If null, a new <see cref="HttpClientHandler"/> is created automatically.
        /// </param>
        /// <returns>Configured HTTP client</returns>
        internal static HttpClient Create(ConfigurationService configurationService, HttpMessageHandler? outerHandler = null)
        {
            HttpMessageHandler handler = outerHandler ?? CreateHandler(configurationService);
            return new HttpClient(handler)
            {
                BaseAddress = new Uri(configurationService.ApiRootUri),
                Timeout = TimeSpan.FromSeconds(REQUEST_TIMEOUT_SECONDS),
            };
        }
    }
}
