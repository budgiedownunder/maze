
namespace Maze.Maui.App.Views
{
    using System;
    using Maze.Api;
    using Microsoft.Maui.Controls;
    using Maze.Maui.App.ViewModels;
    using Maze.Maui.Controls;
    using Maze.Maui.Services;

    using Maze.Wasm.Interop;
    
    using System.Diagnostics;
    using System.Runtime.InteropServices;
    using System.Reflection;
    using Wasmtime;
    using System.Reflection.Metadata;
    using System.Xml.Linq;

    //using static Maze.Maui.App.Views.MainPage.WasmerInterop;


    /// <summary>
    /// This class represents the main page within the application. It provides
    /// functionality to design and solve mazes.
    /// 
    /// This is how the page appears on Windows Desktop:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Design</strong></th>
    ///         <th><strong>Solved</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/windows-design.png" width="250"/></td>
    ///         <td><img src="../../images/screenshots/windows-solved.png" width="250"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    ///  
    /// and this is how it appears on Android/iOS devices:
    /// 
    ///   <table>
    ///     <thead>
    ///       <tr>
    ///         <th><strong>Android</strong></th>
    ///         <th><strong>iOS</strong></th>
    ///       </tr>
    ///     </thead>
    ///     <tbody>
    ///       <tr>
    ///         <td><img src="../../images/screenshots/android-design.png" width="250"/></td>
    ///         <td><img src="../../images/screenshots/ios-design.png" width="250"/></td>
    ///       </tr>
    ///     </tbody> 
    ///  </table>
    /// </summary>
    public partial class MainPage : ContentPage
    {
        const String APP_TITLE = "MAZE";
        MainPageViewModel _viewModel;

        public MainPage()
        {
            InitializeComponent();
            IDeviceTypeService deviceTypeService = new DeviceTypeService();

            _viewModel = new MainPageViewModel(deviceTypeService);
            BindingContext = _viewModel;

            _viewModel.InsertRowsRequested += (s, e) => InsertRows();
            _viewModel.DeleteRowsRequested += (s, e) => DeleteRows();
            _viewModel.InsertColumnsRequested += (s, e) => InsertColumns();
            _viewModel.DeleteColumnsRequested += (s, e) => DeleteColumns();
            _viewModel.SelectRangeRequested += (s, e) => { SetSelectRangeMode(true); };
            _viewModel.DoneRequested += (s, e) => { SetSelectRangeMode(false); };
            _viewModel.SetWallRequested += (s, e) => { ChangeSelectionToWall(); };
            _viewModel.SetStartRequested += (s, e) => { ChangeSelectionToStart(); };
            _viewModel.SetFinishRequested += (s, e) => { ChangeSelectionToFinish(); };
            _viewModel.ClearRequested += (s, e) => { ClearSelection(); };
            _viewModel.SolveRequested += (s, e) => { Solve(); };
            _viewModel.ClearSolutionRequested += (s, e) => { ClearSolution(); };

            MazeGrid.Initialize(!deviceTypeService.IsTouchOnlyDevice());
            MazeGrid.CellTapped += OnMazeGridCellTapped;
            MazeGrid.CellDoubleTapped += OnMazeGridCellDoubleTapped;
            MazeGrid.KeyDown += OnMazeGridKeyDown;
            MazeGrid.SelectionChanged += OnMazeGridSelectionChanged;

            MazeGrid.ActivateCell(1, 1, false);

            UpdateControls();
        }

        private bool IsTouchOnlyDevice { get => _viewModel.IsTouchOnlyDevice; }

        private bool IsSolveSupported { get => !IsTouchOnlyDevice;  }

        private bool IsSolutionDisplayed { get; set;  } = false;

        private void OnMazeGridCellTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            MazeGrid.OnCellTapped(e.Cell, false);
        }

        private void OnMazeGridCellDoubleTapped(object sender, MazeGridCellTappedEventArgs e)
        {
            bool inExtendedSelectionMode = MazeGrid.IsExtendedSelectionMode;
            if (IsTouchOnlyDevice && inExtendedSelectionMode)
                SetSelectRangeMode(false);

            MazeGrid.OnCellDoubleTapped(e.Cell, false);

            if (IsTouchOnlyDevice && !inExtendedSelectionMode)
                SetSelectRangeMode(true);
        }

        private void OnMazeGridKeyDown(object sender, MazeGridKeyDownEventArgs e)
        {
            switch (e.Key)
            {
                case Controls.Keyboard.Key.W:
                    if (_viewModel.CanSetWall)
                        _viewModel.SetWallCommand.Execute(null);
                    break;
                case Controls.Keyboard.Key.S:
                    if (_viewModel.CanSetStart)
                        _viewModel.SetStartCommand.Execute(null);
                    break;
                case Controls.Keyboard.Key.F:
                    if (_viewModel.CanSetFinish)
                        _viewModel.SetFinishCommand.Execute(null);
                    break;
                case Controls.Keyboard.Key.Delete:
                    if (_viewModel.CanClear)
                        _viewModel.ClearCommand.Execute(null);
                    break;
                default:
                    MazeGrid.OnProcessKeyDown(e.KeyState, e.Key, false);
                    break;
            }
        }

        private void ChangeSelectionToWall()
        {
            ChangeSelectedCellsContent(Maze.CellType.Wall);
        }

        private void ChangeSelectionToStart()
        {
            ChangeSelectedCellsContent(Maze.CellType.Start);
        }

        private void ChangeSelectionToFinish()
        {
            ChangeSelectedCellsContent(Maze.CellType.Finish);
        }

        private void ClearSelection()
        {
            ChangeSelectedCellsContent(Maze.CellType.Empty);
        }

        private void ChangeSelectedCellsContent(Maze.CellType newCellType)
        {
            MazeGrid.SetSelectionContent(newCellType);
            ExitSelectionModeAndUpdateControls();
        }

        private void DeleteRows()
        {
            MazeGrid.DeleteSelectedRows(); ;
            ExitSelectionModeAndUpdateControls();
        }

        private void DeleteColumns()
        {
            MazeGrid.DeleteSelectedColumns();
            ExitSelectionModeAndUpdateControls();
        }

        private void InsertRows()
        {
            MazeGrid.InsertSelectedRows(); ;
            ExitSelectionModeAndUpdateControls();
        }

        private void InsertColumns()
        {
            MazeGrid.InsertSelectedColumns(); ;
            ExitSelectionModeAndUpdateControls();
        }

        private void Solve()
        {
            try
            {
                Maze maze = MazeGrid.ToMaze();
                Solution solution = maze.Solve();

                IsSolutionDisplayed = MazeGrid.DisplaySolution(solution);
                UpdateControls();
            }
            catch (Exception ex) 
            { 
                DisplayAlert(APP_TITLE, $"Unable to solve maze\n\nReason: {ex.Message}", "OK");
            }
            
        }

        private void ClearSolution()
        {
            IsSolutionDisplayed = !MazeGrid.ClearLastSolution();
            UpdateControls();
        }

        private void SetSelectRangeMode(bool enable)
        {
            EnableExtendedSelectionMode(enable);
            ShowSelectRangeButtons(!enable);
        }

        private void ExitSelectionModeAndUpdateControls()
        {
            EnableExtendedSelectionMode(false);
            UpdateControls();
        }

        private void EnableExtendedSelectionMode(bool enable)
        {
            if (MazeGrid.IsExtendedSelectionMode == enable)
                return;

            if (enable)
                MazeGrid.EnableExtendedSelection();
            else
                MazeGrid.CancelExtendedSelection();
        }

        private void ShowCellEditButtons(bool haveSelection)
        {
            CellStatus status = MazeGrid.GetCurrentSelectionStatus();

            _viewModel.CanSetWall = !status.IsAllWalls && !IsSolutionDisplayed;
            _viewModel.CanSetStart = status.IsSingleCell && !status.IsStart && !IsSolutionDisplayed;
            _viewModel.CanSetFinish = status.IsSingleCell && !status.IsFinish && !IsSolutionDisplayed;
            _viewModel.CanClear = !status.IsEmpty && !IsSolutionDisplayed;
        }

        private void ShowEditRowColumnButtons()
        {
            bool allRowsSelected = MazeGrid.AllRowsSelected;
            bool allColumnsSelected = MazeGrid.AllColumnsSelected;

            _viewModel.CanInsertRows = allColumnsSelected && !IsSolutionDisplayed;
            _viewModel.CanDeleteRows = allColumnsSelected && !allRowsSelected && !IsSolutionDisplayed;
            _viewModel.CanInsertColumns = allRowsSelected && !IsSolutionDisplayed;
            _viewModel.CanDeleteColumns = allRowsSelected && !allColumnsSelected && !IsSolutionDisplayed;
        }

        private void ShowSelectRangeButtons(bool show)
        {
            bool touchOnly = IsTouchOnlyDevice;
            bool showSelectRangeBtn = show && touchOnly && !IsSolutionDisplayed;
            bool showDoneBtn = !show && touchOnly && !IsSolutionDisplayed;

            _viewModel.CanSelectRange = showSelectRangeBtn;
            _viewModel.CanShowDone = showDoneBtn;
        }


        private void ShowSolveButtons()
        {
            _viewModel.CanSolve = IsSolveSupported && !IsSolutionDisplayed;
            _viewModel.CanClearSolution = IsSolveSupported && IsSolutionDisplayed;
        }

        private void OnMazeGridSelectionChanged(object sender, MazeGridSelectionChangedEventArgs e)
        {
            UpdateControls();
        }

        private void UpdateControls()
        {
            bool showSelectRangeButtons = IsTouchOnlyDevice || MazeGrid.IsExtendedSelectionMode;
            bool haveSelection = MazeGrid.ActiveCell != null;
            bool showTopRowLayout = showSelectRangeButtons || haveSelection;
            ShowMainGridRow(0, showTopRowLayout);
            if (showTopRowLayout)
            {
                ShowCellEditButtons(haveSelection);
                ShowEditRowColumnButtons();
                ShowSelectRangeButtons(!MazeGrid.IsExtendedSelectionMode);
                ShowSolveButtons();
            }
        }

        private void ShowMainGridRow(int row, bool show)
        {
            MainGrid.RowDefinitions[row].Height = show ? GridLength.Auto : new GridLength(0);
            if (row == 0)
                TopRowLayout.IsVisible = show;
        }

        public static class WasmerInterop
        {
            // Define the name of the Wasmer library
            private const string LibraryName =
#if WINDOWS
        "wasmer.dll"; // For Windows
#elif LINUX
        "libwasmer.so"; // For Linux
#elif OSX
        "libwasmer.dylib"; // For macOS
#else
        //"__Internal"; // For iOS with static linking
        "libwasmer.so";
#endif

            // Wasmer function: Create a new engine
            [DllImport(LibraryName, EntryPoint = "wasm_engine_new", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_engine_new();

            [DllImport(LibraryName, EntryPoint = "wasm_engine_delete", CallingConvention = CallingConvention.Cdecl)]
            public static extern void wasm_engine_delete(IntPtr engine);

            // Wasmer function: Create a new store
            [DllImport(LibraryName, EntryPoint = "wasm_store_new", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_store_new(IntPtr engine);

            // Wasmer function: Create a new store
            [DllImport(LibraryName, EntryPoint = "wasm_store_delete", CallingConvention = CallingConvention.Cdecl)]
            public static extern void wasm_store_delete(IntPtr store);

            [StructLayout(LayoutKind.Sequential)]
            public struct wasm_byte_vec_t
            {
                public nuint size;
                public IntPtr data;
            }

            [DllImport(LibraryName, EntryPoint = "wasm_byte_vec_new", CallingConvention = CallingConvention.Cdecl)]
            public static extern void wasm_byte_vec_new(ref wasm_byte_vec_t vec, nuint size, IntPtr data);

            [DllImport(LibraryName, EntryPoint = "wasm_byte_vec_delete", CallingConvention = CallingConvention.Cdecl)]
            public static extern void wasm_byte_vec_delete(ref wasm_byte_vec_t vec);

            [DllImport(LibraryName, EntryPoint = "wasm_module_validate", CallingConvention = CallingConvention.Cdecl)]
            public static extern bool wasm_module_validate(IntPtr store, ref wasm_byte_vec_t binary);

            [DllImport(LibraryName, EntryPoint = "wasm_module_new", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_module_new(IntPtr store, ref wasm_byte_vec_t binary);

            [DllImport(LibraryName, EntryPoint = "wasm_module_delete", CallingConvention = CallingConvention.Cdecl)]
            public static extern void wasm_module_delete(IntPtr module);

            [StructLayout(LayoutKind.Sequential)]
            public struct wasm_importtype_vec_t
            {
                public nuint size;
                public IntPtr data; // Array of `wasm_importtype_t`
            }

            [DllImport(LibraryName, EntryPoint = "wasm_module_imports", CallingConvention = CallingConvention.Cdecl)]
            public static extern void wasm_module_imports(IntPtr module, ref wasm_importtype_vec_t imports);

            public struct wasm_exporttype_vec_t
            {
                public nuint size;
                public IntPtr data; // Array of `wasm_exporttype_t`
            }

            [DllImport(LibraryName, EntryPoint = "wasm_module_exports", CallingConvention = CallingConvention.Cdecl)]
            public static extern void wasm_module_exports(IntPtr module, ref wasm_exporttype_vec_t exports);

            [DllImport(LibraryName, EntryPoint = "wasm_exporttype_name", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_exporttype_name(IntPtr module, IntPtr export);


            [DllImport(LibraryName, EntryPoint = "wasm_extern_vec_new", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_extern_vec_new(IntPtr instance);

            [StructLayout(LayoutKind.Sequential)]
            public struct wasm_extern_vec_t
            {
                public nuint size;
                public IntPtr data; // Array of `wasm_extern_t` pointers
            }


            [DllImport(LibraryName, EntryPoint = "wasm_instance_new", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_instance_new(IntPtr store, IntPtr module, ref wasm_extern_vec_t imports, IntPtr traps);

            [DllImport(LibraryName, EntryPoint = "wasm_instance_delete", CallingConvention = CallingConvention.Cdecl)]
            public static extern void wasm_instance_delete(IntPtr instance);

            // ***************************************
            // Instance Exports
            // ***************************************

            [DllImport(LibraryName, EntryPoint = "wasm_instance_exports", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_instance_exports(IntPtr instance, ref wasm_extern_vec_t exports);

            //      [DllImport(LibraryName, EntryPoint = "wasm_extern_name", CallingConvention = CallingConvention.Cdecl)]
            //     public static extern IntPtr wasm_extern_name(IntPtr wasmExtern);

            public enum ExternKind : byte
            {
                Function = 0,
                Global= 1,
                Table = 2,
                Memory = 3
            }

            [DllImport(LibraryName, EntryPoint = "wasm_extern_kind", CallingConvention = CallingConvention.Cdecl)]
            public static extern ExternKind wasm_extern_kind(IntPtr externPtr /*  wasm_extern_t * */); // returns wasm_externkind_t

            [DllImport(LibraryName, EntryPoint = "wasm_extern_type", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_extern_type(IntPtr externPtr /* wasm_extern_t */);// returns wasm_externtype_t (owned)


            // *******************************
            // Export Types
            // *******************************
            [DllImport(LibraryName, EntryPoint = "wasm_exporttype_name", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_exporttype_name(IntPtr exportTypePtr /* wasm_exporttype_t * */); // returns wasm_name_t * == wasm_byte_vec_t

            [DllImport(LibraryName, EntryPoint = "wasm_exporttype_type", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_exporttype_type(IntPtr exportTypePtr); // returns wasm_externtype_t *

            // *******************************
            // Extern Types
            // *******************************

            [DllImport(LibraryName, EntryPoint = "wasm_externtype_delete", CallingConvention = CallingConvention.Cdecl)]
            public static extern void wasm_externtype_delete(IntPtr externtTypePtr /* wasm_externtype_t */);

            

            [DllImport(LibraryName, EntryPoint = "wasm_externtype_kind", CallingConvention = CallingConvention.Cdecl)]
            public static extern ExternKind wasm_externtype_kind(IntPtr externtTypePtr /* wasm_externtype_t */);// returns wasm_externkind_t



            [DllImport(LibraryName, EntryPoint = "wasm_externtype_as_functype", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_externtype_as_functype(IntPtr externtTypPtr /* wasm_externtype_t */);// returns wasm_functype_t *

            [DllImport(LibraryName, EntryPoint = "wasm_extern_as_func", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_extern_as_func(IntPtr wasmExtern);


            [DllImport(LibraryName, EntryPoint = "wasm_trap_message", CallingConvention = CallingConvention.Cdecl)]
            public static extern IntPtr wasm_trap_message(IntPtr trap, ref wasm_byte_vec_t message);

            [DllImport(LibraryName, EntryPoint = "wasmer_last_error_length", CallingConvention = CallingConvention.Cdecl)]
            public static extern int wasmer_last_error_length();

            [DllImport(LibraryName, EntryPoint = "wasmer_last_error_message", CallingConvention = CallingConvention.Cdecl)]
            public static extern int wasmer_last_error_message(IntPtr buffer, int length);

            public static string GetExportName(IntPtr wasmExternPtr /* wasm_extern_t */)
            {
                string name = "N.A";
                IntPtr wasmExportTypePtr = IntPtr.Zero;
                IntPtr externTypePtr = WasmerInterop.wasm_extern_type(wasmExternPtr);
                if (externTypePtr != IntPtr.Zero)
                {
                    IntPtr namePtr = WasmerInterop.wasm_exporttype_name(externTypePtr);
                    if (namePtr == IntPtr.Zero)
                    {
                        wasm_byte_vec_t byteVec = Marshal.PtrToStructure<wasm_byte_vec_t>(namePtr);
                        name = Marshal.PtrToStringUTF8(byteVec.data, (int)byteVec.size);

                    }
                    WasmerInterop.wasm_externtype_delete(externTypePtr);
                }
                return name;
            }
        }

        public static string GetLastError()
        {
            int length = WasmerInterop.wasmer_last_error_length();
            if (length == 0)
            {
                return "No error message available.";
            }

            IntPtr buffer = Marshal.AllocHGlobal(length);
            try
            {
                WasmerInterop.wasmer_last_error_message(buffer, length);
                return Marshal.PtrToStringAnsi(buffer) ?? "Failed to retrieve error message.";
            }
            finally
            {
                Marshal.FreeHGlobal(buffer);
            }
        }

        private async void OnTestBtnClicked(object sender, EventArgs e)
        {
            IntPtr engine = IntPtr.Zero;
            IntPtr store = IntPtr.Zero;
            GCHandle handle = default;
            var wasmVec = new WasmerInterop.wasm_byte_vec_t();
            IntPtr module = IntPtr.Zero;
            IntPtr instance = IntPtr.Zero;
            var emptyImports = new WasmerInterop.wasm_extern_vec_t();

            int functionCount = 0, asFunctionCount = 0;
            byte[] wasmBytes;

            try
            {
                //string wasmPath = MazeWasmInterop.GetWasmPath();
                //byte[] wasmBytes = File.ReadAllBytes(wasmPath);
                //var executionPath = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);
                //string wasmPath = Path.Combine(FileSystem.AppDataDirectory, "maze_wasm.wasm");
                //if (!File.Exists(wasmPath))
               // {
               //     throw new InvalidOperationException($"Web assembly file not found at path ${wasmPath}");
               // }


                //await DisplayAlert("FOUND", wasmPath, "OK");
                using var stream = await FileSystem.OpenAppPackageFileAsync("maze_wasm.wasm");
                //using var reader = new StreamReader(stream);
                //var contents = reader.ReadToEnd();
                using var memoryStream = new MemoryStream();
                await stream.CopyToAsync(memoryStream);
                wasmBytes = memoryStream.ToArray();

                Debug.WriteLine("maze_wasm.wasm contents:");
                Debug.WriteLine($"Length = {wasmBytes.Length}");  
                await DisplayAlert("", "Read maze_wasm.wasm", "OK");
//#if WINDOWS
                engine = WasmerInterop.wasm_engine_new();
                if (engine == IntPtr.Zero)
                {
                    throw new Exception("Failed to create Wasm engine.");
                }
                store = WasmerInterop.wasm_store_new(engine);
                if (store== IntPtr.Zero)
                {
                    throw new Exception("Failed to create Wasm store.");
                }

                handle = GCHandle.Alloc(wasmBytes, GCHandleType.Pinned);

                WasmerInterop.wasm_byte_vec_new(ref wasmVec, (nuint)wasmBytes.Length, handle.AddrOfPinnedObject());

                bool isValid = WasmerInterop.wasm_module_validate(store, ref wasmVec);

                if (!isValid)
                {
                    throw new Exception("Web assembly module is invalid.");
                }

                module = WasmerInterop.wasm_module_new(store, ref wasmVec);
                if (module == IntPtr.Zero)
                {
                    throw new Exception($"Failed to create Wasm module: {GetLastError()}");
                }

                instance = WasmerInterop.wasm_instance_new(store, module, ref emptyImports, IntPtr.Zero); 
                if (instance == IntPtr.Zero)
                {
                    string errorMessage = GetLastError();
                    throw new Exception($"Failed to instantiate module => {errorMessage}");
                }

                var instanceExports = new WasmerInterop.wasm_extern_vec_t();
                WasmerInterop.wasm_instance_exports(instance, ref instanceExports /* wasm_extern_vec_t * */);

                var moduelExports = new WasmerInterop.wasm_exporttype_vec_t();
                WasmerInterop.wasm_module_exports(module, ref moduelExports);
                for (int i = 0; i < (int)moduelExports.size; i++)
                {
                    IntPtr exportTypePtr = Marshal.ReadIntPtr(moduelExports.data, i * IntPtr.Size);
                    IntPtr exportTypeNamePtr = WasmerInterop.wasm_exporttype_name(exportTypePtr);
                    WasmerInterop.wasm_byte_vec_t rawBytes = Marshal.PtrToStructure<WasmerInterop.wasm_byte_vec_t>(exportTypeNamePtr);
                    string name = Marshal.PtrToStringUTF8(rawBytes.data, (int)rawBytes.size);

                    IntPtr externTypePtr = WasmerInterop.wasm_exporttype_type(exportTypePtr);
                    WasmerInterop.ExternKind externKind = WasmerInterop.wasm_externtype_kind(externTypePtr);

                    if (externKind == WasmerInterop.ExternKind.Function)
                    {
                        functionCount++;
                        IntPtr wasmExternPtr = Marshal.ReadIntPtr(instanceExports.data + i * IntPtr.Size);
                        IntPtr wasExternFuncPtr = WasmerInterop.wasm_extern_as_func(wasmExternPtr);
                        if (wasExternFuncPtr != IntPtr.Zero)
                        {
                            asFunctionCount++;
                            Debug.WriteLine($"{name} - Extern Function Type Pointer = {wasExternFuncPtr}");

                            if (name == "new_maze_wasm")
                            {
                                Debug.WriteLine("** FOUND new_maze_wasm() **");
                            }
                        }
                    }
                }

                await DisplayAlert("", $"Successfully loaded WASM, initialised engine and compiled WASM - found {instanceExports.size} exports, {functionCount} functions, {asFunctionCount} as func count", "OK");
//#endif            
            }
            catch (Exception ex) {
                await DisplayAlert("Error", $"{ex.Message}", "OK");
            }
            finally
            {
                if (instance != IntPtr.Zero)
                    WasmerInterop.wasm_instance_delete(instance);

                if (module != IntPtr.Zero)
                    WasmerInterop.wasm_module_delete(module);

                if (store != IntPtr.Zero)
                    WasmerInterop.wasm_store_delete(store);

                if (engine != IntPtr.Zero)
                    WasmerInterop.wasm_engine_delete(engine);

                if (wasmVec.data != IntPtr.Zero)
                    WasmerInterop.wasm_byte_vec_delete(ref wasmVec);

                if (handle.IsAllocated)
                    handle.Free(); // Free pinned array
            }
        }
    }
}
