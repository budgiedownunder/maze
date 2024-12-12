using Android.App;
using Android.Runtime;
using System.Runtime.InteropServices;

namespace Maze.Maui.App
{
    [Application]
    public class MainApplication : MauiApplication
    {
        public MainApplication(IntPtr handle, JniHandleOwnership ownership)
            : base(handle, ownership)
        {
            string? nativeLibraryDir = Android.App.Application.Context.ApplicationInfo?.NativeLibraryDir;
            if(nativeLibraryDir != null)
            {
                Console.WriteLine($"*********** => Native library directory: {nativeLibraryDir}");
                var wasmerLibFilePath = System.IO.Path.Combine(nativeLibraryDir, "libwasmer.so");
                Console.WriteLine($"************* => {wasmerLibFilePath} exist = {File.Exists(wasmerLibFilePath)}");

                try
                {
                    IntPtr libHandle = NativeLibrary.Load(wasmerLibFilePath);

                    // If successful, log success
                    Console.WriteLine($"************* => {wasmerLibFilePath} successfully loaded");

                    // Free the library to release resources
                    NativeLibrary.Free(libHandle);

                }
                catch (Exception ex)
                {
                    Console.WriteLine($"\"************* Error loading {wasmerLibFilePath}: {ex.Message}");
                }
            }
        }

        protected override MauiApp CreateMauiApp() => MauiProgram.CreateMauiApp();
    }
}
