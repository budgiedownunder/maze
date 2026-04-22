using CommunityToolkit.Maui;
//using Maze.Maui.App.Services;
using Microsoft.Extensions.Logging;
using Maze.Interop;
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
#if ANDROID
            // IconTintColorBehavior does not apply inside SwipeItemView on Android.
            // Apply a white colour filter directly via the native image handler instead.
            Microsoft.Maui.Handlers.ImageHandler.Mapper.AppendToMapping("SwipeItemWhiteTint", (handler, view) =>
            {
                if (view is Image { ClassId: "white-tint" })
                    handler.PlatformView.SetColorFilter(
                        Android.Graphics.Color.White,
                        Android.Graphics.PorterDuff.Mode.SrcIn!);
            });
#elif IOS || MACCATALYST
            // Same limitation applies on iOS — IconTintColorBehavior does not tint inside SwipeItemView.
            Microsoft.Maui.Handlers.ImageHandler.Mapper.AppendToMapping("SwipeItemWhiteTint", (handler, view) =>
            {
                if (view is Image { ClassId: "white-tint" })
                {
                    handler.PlatformView.TintColor = UIKit.UIColor.White;
                    handler.PlatformView.Image = handler.PlatformView.Image?
                        .ImageWithRenderingMode(UIKit.UIImageRenderingMode.AlwaysTemplate);
                }
            });
#endif

            var builder = MauiApp.CreateBuilder();
            builder
                .UseMauiApp<App>()
                .UseMauiCommunityToolkit()
                .ConfigureFonts(fonts =>
                {
                    fonts.AddFont("OpenSans-Regular.ttf", "OpenSansRegular");
                    fonts.AddFont("OpenSans-Semibold.ttf", "OpenSansSemibold");
                });

            InitializeMazeInterop();

            builder.Services.AddSingleton<ConfigurationService>();
            builder.Services.AddSingleton<IAuthService, AuthHttpClientService>();
            builder.Services.AddSingleton<IAppFeaturesService, AppFeaturesHttpClientService>();
            if(useMockMazeService)
                builder.Services.AddSingleton<IMazeService, MockMazeService>();
            else
                builder.Services.AddSingleton<IMazeService, MazeHttpClientService>();

            builder.Services.AddSingleton<IDeviceTypeService>(provider => new DeviceTypeService());
            builder.Services.AddSingleton<IDialogService>(provider => new PopupWindowService());

            builder.Services.AddTransient<LoginViewModel>();
            builder.Services.AddTransient<SignUpViewModel>();
            builder.Services.AddTransient<ChangePasswordViewModel>();
            builder.Services.AddSingleton<MazesViewModel>();
            builder.Services.AddTransient<MazeViewModel>();
            builder.Services.AddTransient<MazeGameViewModel>();
            builder.Services.AddSingleton<AccountViewModel>();

            builder.Services.AddTransient<LoginPage>();
            builder.Services.AddTransient<SignUpPage>();
            builder.Services.AddTransient<ChangePasswordPage>();
            builder.Services.AddSingleton<AppShell>();
            builder.Services.AddSingleton<MazesPage>();
            builder.Services.AddTransient<MazePage>();
            builder.Services.AddTransient<MazeGamePage>();
            builder.Services.AddTransient<Play3dGamePage>();

#if DEBUG
            builder.Logging.AddDebug();
#endif
            return builder.Build();
        }
        /// <summary>
        /// Initializes the Maze WebAssembly interop 
        /// </summary>
        private static async void InitializeMazeInterop()
        {
            // TO DO - move to a service
            try
            {
                MazeInterop.ConnectionType connectionType =
                    OperatingSystem.IsIOS()     ? MazeInterop.ConnectionType.Native :
                    OperatingSystem.IsAndroid() ? MazeInterop.ConnectionType.Wasmer :
                                                MazeInterop.ConnectionType.Wasmtime;

                byte[]? wasmBytes = null;
                if (!OperatingSystem.IsIOS())
                {
                    using var stream = await FileSystem.OpenAppPackageFileAsync("maze_wasm.wasm");
                    using var ms = new MemoryStream();
                    await stream.CopyToAsync(ms);
                    wasmBytes = ms.ToArray();
                }
                MazeInterop interop = MazeInterop.GetInstance(connectionType, true, wasmBytes);
                
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to initialize MazeInterop: {ex.Message}");
            }
        }
    }
}
