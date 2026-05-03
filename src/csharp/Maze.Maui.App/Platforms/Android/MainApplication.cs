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
            if (nativeLibraryDir is not null)
            {
                var wasmerLibFilePath = System.IO.Path.Combine(nativeLibraryDir, "libwasmer.so");
                try
                {
                    IntPtr libHandle = NativeLibrary.Load(wasmerLibFilePath);
                    NativeLibrary.Free(libHandle);
                }
                catch (Exception)
                {
                }
            }
        }

        protected override MauiApp CreateMauiApp() => MauiProgram.CreateMauiApp();
    }
}
