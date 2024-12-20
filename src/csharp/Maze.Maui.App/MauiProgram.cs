using CommunityToolkit.Maui;
using MauiGestures;
//using Maze.Maui.App.Services;
using Microsoft.Extensions.Logging;
using Maze.Wasm.Interop;

namespace Maze.Maui.App
{
    public static class MauiProgram
    {
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

  //     builder.Services.AddSingleton<IDeviceTypeService, DeviceTypeService>();

#if DEBUG
            builder.Logging.AddDebug();
#endif

            InitializeMazeWasmInterop();

            return builder.Build();
        }

        // To do - move to a service
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
