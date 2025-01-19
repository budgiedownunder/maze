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
        // Define whether or not to use the mock maze service
#if IOS
        static bool useMockMazeService = false;
#else
        static bool useMockMazeService = false;
#endif

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

            builder.Services.AddSingleton<ConfigurationService>();
            if(useMockMazeService)
                builder.Services.AddSingleton<IMazeService, MockMazeService>();
            else
                builder.Services.AddSingleton<IMazeService, MazeHttpClientService>();

            builder.Services.AddSingleton<IDeviceTypeService>(provider => new DeviceTypeService());
            builder.Services.AddSingleton<IDialogService>(provider => new PopupWindowService());

            builder.Services.AddSingleton<MazesViewModel>();
            builder.Services.AddTransient<MazeViewModel>();

            builder.Services.AddSingleton<MazesPage>();
            builder.Services.AddTransient<MazePage>();

#if DEBUG
            builder.Logging.AddDebug();
#endif
            return builder.Build();
        }
        /// <summary>
        /// Initializes the Maze WebAssembly interop 
        /// </summary>
        private static async void InitializeMazeWasmInterop()
        {
            // TO DO - move to a service
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
