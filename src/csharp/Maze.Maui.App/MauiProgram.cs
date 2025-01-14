using CommunityToolkit.Maui;
using MauiGestures;
//using Maze.Maui.App.Services;
using Microsoft.Extensions.Logging;
using Maze.Wasm.Interop;
using Maze.Maui.Services;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Maze.Maui.App.Views;

namespace Maze.Maui.App
{
    /// <summary>
    /// MAUI program class
    /// </summary>
    public static class MauiProgram
    {
        /// <summary>
        /// Initializes the application static instance
        /// </summary>
        /// <returns>Instance</returns>
        public static MauiApp CreateMauiApp()
        {
            var builder = MauiApp.CreateBuilder();
            builder
                .UseMauiApp<App>()
                .UseMauiCommunityToolkit()
                .UseAdvancedGestures()
                .ConfigureFonts(fonts =>
                {
                    fonts.AddFont("OpenSans-Regular.ttf", "OpenSansRegular");
                    fonts.AddFont("OpenSans-Semibold.ttf", "OpenSansSemibold");
                });

            InitializeMazeWasmInterop();

            builder.Services.AddSingleton<IMazeService>(provider => GetMazeService());
            builder.Services.AddSingleton<IDeviceTypeService>(provider => new DeviceTypeService());
            builder.Services.AddSingleton<IDialogService>(provider => new DialogService());

            builder.Services.AddSingleton<MazesViewModel>();
            builder.Services.AddTransient<MazePageViewModel>();

            builder.Services.AddSingleton<MazesPage>();
            builder.Services.AddTransient<MazePage>();


#if DEBUG
            builder.Logging.AddDebug();
#endif
            return builder.Build();
        }

        private static IMazeService GetMazeService()
        {
            // TO DO - drive the choice of client service and endpoint(s) from
            // configuration file settings
#if WINDOWS
            string rootUri = "http://localhost:8080/api/v1";
#elif ANDROID
        string rootUri = "http://10.0.2.2:8080/api/v1";
#elif IOS
        string rootUri = "http://localhost:8080/api/v1";
#else
        string rootUri = "http://localhost:8080/api/v1";
#endif
            return new MazeHttpClientService(rootUri);
        }

        // TO DO - move to a service
        private static async void InitializeMazeWasmInterop()
        {
            try
            {
                using var stream = await FileSystem.OpenAppPackageFileAsync("maze_wasm.wasm");
                using var memoryStream = new MemoryStream();
                await stream.CopyToAsync(memoryStream);
                byte[] wasmBytes2 = memoryStream.ToArray();
                MazeWasmInterop.ConnectionType connectionType = OperatingSystem.IsIOS() || OperatingSystem.IsAndroid() 
                    ? MazeWasmInterop.ConnectionType.Wasmer : MazeWasmInterop.ConnectionType.Wasmtime;
                MazeWasmInterop interop2 = MazeWasmInterop.GetInstance(connectionType, true, wasmBytes2);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to initialize MazeWasmInterop: {ex.Message}");
            }
        }
    }
}
