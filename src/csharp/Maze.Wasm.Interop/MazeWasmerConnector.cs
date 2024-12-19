namespace Maze.Wasm.Interop
{
    using System.Reflection;
    using System.Runtime.InteropServices;
    using Microsoft.Extensions.Configuration;
    using System.Text;
    using Wasmtime;
    using System.Reflection.Metadata;
    using System.Diagnostics;
    using static System.Formats.Asn1.AsnWriter;
    using static Maze.Wasm.Interop.WasmerInterop;
    using System;
    using System.Numerics;
    using static System.Runtime.InteropServices.JavaScript.JSType;
    using System.Drawing;
    using System.Xml.Schema;
    using static Maze.Wasm.Interop.MazeWasmInterop;

    //using Wasmtime;

    /// <summary>
    ///  Wasmer library interop 
    /// </summary>
    internal static class WasmerInterop
    {
        // Define the name of the Wasmer library
        private const string LibraryName =
#if WINDOWS
        "wasmer.dll";
#elif LINUX
        "libwasmer.so";
#elif MACOS
        "libwasmer.dylib";
#elif ANDROID
        "libwasmer.so";
//#elif IOS
//        "libwasmer.so";
#else
        "wasmer.dll";
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

        public struct wasm_exporttype_vec_t
        {
            public nuint size;
            public IntPtr data; // Array of `wasm_exporttype_t`
        }

        [DllImport(LibraryName, EntryPoint = "wasm_module_exports", CallingConvention = CallingConvention.Cdecl)]
        public static extern void wasm_module_exports(IntPtr module, ref wasm_exporttype_vec_t exports);

        [StructLayout(LayoutKind.Sequential)]
        public struct wasm_extern_vec_t
        {
            public nuint size;
            public IntPtr data; // Array of `wasm_extern_t` pointers
        }

        public enum wasm_valkind_t : byte
        {
            I32 = 0,
            I64 = 1,
            F32 = 2,
            F64 = 3,
            EXTERNREF = 128,
            WASM_FUNCREF = 129,

            NONE = 255 // Custom (not part of the C API)
        }

        [StructLayout(LayoutKind.Explicit)]
        public struct wasm_val_union
        {
            [FieldOffset(0)] public int i32;
            [FieldOffset(0)] public long i64;
            [FieldOffset(0)] public float f32;
            [FieldOffset(0)] public double f64;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct wasm_val_t
        {
            public wasm_valkind_t kind;
            public wasm_val_union of;

            public void To_I32(int i32)
            {
                this.kind = wasm_valkind_t.I32;
                this.of.i32 = i32;
            }
            public void To_I64(long i64)
            {
                this.kind = wasm_valkind_t.I64;
                this.of.i64 = i64;
            }
            public void To_F32(float f32)
            {
                this.kind = wasm_valkind_t.F32;
                this.of.f32 = f32;
            }
            public void To_F64(double f64)
            {
                this.kind = wasm_valkind_t.F64;
                this.of.f64 = f64;
            }
            public void FromValue(wasm_valkind_t kind, object value)
            {
                try
                {
                    switch (kind)
                    {
                        case wasm_valkind_t.I32:
                            To_I32(Convert.ToInt32(value));
                            break;
                        case wasm_valkind_t.I64:
                            To_I64(Convert.ToInt64(value));
                            break;
                        case wasm_valkind_t.F32:
                            To_F32(Convert.ToSingle(value));
                            break;
                        case wasm_valkind_t.F64:
                            To_F64(Convert.ToDouble(value));
                            break;
                        case wasm_valkind_t.NONE:
                            this.kind = wasm_valkind_t.NONE;
                            break;
                        default:
                            throw new Exception($"kind is unsupported");
                    };
                }
                catch (Exception e) {
                    throw new Exception($"cannot convert value of kind {kind.ToString()} to wasm_val_t - {e.Message}");
                }
            }
            public object? ToValue()
            {
                object? value = null;
                try
                {
                    switch (kind)
                    {
                        case wasm_valkind_t.I32:
                            value = this.of.i32;
                            break;
                        case wasm_valkind_t.I64:
                            value = this.of.i64;
                            break;
                        case wasm_valkind_t.F32:
                            value = this.of.f32;
                            break;
                        case wasm_valkind_t.F64:
                            value = this.of.f64;
                            break;
                        case wasm_valkind_t.NONE:
                            break;
                        default:
                            throw new Exception($"kind is unsupported");
                    };
                }
                catch (Exception e)
                {
                    throw new Exception($"cannot convert value of kind {kind.ToString()} to object - {e.Message}");
                }
                return value;
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct wasm_val_vec_t
        {
            public nuint size;
            public IntPtr data; // Array of `wasm_val_vec_t` values
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

        public enum ExternKind : byte
        {
            Function = 0,
            Global = 1,
            Table = 2,
            Memory = 3
        }

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

        [DllImport(LibraryName, EntryPoint = "wasm_extern_as_memory", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr wasm_extern_as_memory(IntPtr wasmExtern); // Returns wasm_func_t *

        [DllImport(LibraryName, EntryPoint = "wasm_extern_as_func", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr wasm_extern_as_func(IntPtr wasmExtern); // Returns wasm_func_t *

        // *******************************
        // Memory
        // *******************************
        [DllImport(LibraryName, EntryPoint = "wasm_memory_data", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr wasm_memory_data(IntPtr memory); // Returns byte_t *

        [DllImport(LibraryName, EntryPoint = "wasm_memory_data_size", CallingConvention = CallingConvention.Cdecl)]
        public static extern uint wasm_memory_data_size(IntPtr memory); // Returns size_t *

        // *******************************
        // Functions
        // *******************************
        [DllImport(LibraryName, EntryPoint = "wasm_func_type", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr wasm_func_type(IntPtr func /* wasm_func_t* */); // Returns own wasm_functype_t *

        [StructLayout(LayoutKind.Sequential)]
        public struct wasm_valtype_vec_t
        {
            public UIntPtr size;
            public IntPtr data;
        }

        [DllImport(LibraryName, EntryPoint = "wasm_functype_params", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr wasm_functype_params(IntPtr funcType);

        [DllImport(LibraryName, EntryPoint = "wasm_functype_results", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr wasm_functype_results(IntPtr funcType);

        [DllImport(LibraryName, EntryPoint = "wasm_val_vec_new", CallingConvention = CallingConvention.Cdecl)]
        public static extern void wasm_val_vec_new(out wasm_val_vec_t outVec, UIntPtr size, IntPtr data);

        [DllImport(LibraryName, EntryPoint = "wasm_val_vec_new_uninitialized", CallingConvention = CallingConvention.Cdecl)]
        public static extern void wasm_val_vec_new_uninitialized(ref wasm_val_vec_t buffer, UIntPtr size);

        [DllImport(LibraryName, EntryPoint = "wasm_val_vec_delete", CallingConvention = CallingConvention.Cdecl)]
        public static extern void wasm_val_vec_delete(ref wasm_val_vec_t value );

        [DllImport(LibraryName, EntryPoint = "wasm_func_call", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr wasm_func_call(IntPtr functionPtr, ref wasm_val_vec_t args, ref wasm_val_vec_t results); // returns: own wasm_trap_t*

        // *******************************
        // Traps
        // *******************************
        [DllImport(LibraryName, EntryPoint = "wasm_trap_message", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr wasm_trap_message(IntPtr trap, ref wasm_byte_vec_t message); /* wasm_message_t* == wasm_byte_vec_t */

        [DllImport(LibraryName, EntryPoint = "wasm_trap_delete", CallingConvention = CallingConvention.Cdecl)]
        public static extern void wasm_trap_delete(IntPtr trap);

        [DllImport(LibraryName, EntryPoint = "wasm_trap_new", CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr wasm_trap_new(IntPtr store, ref wasm_byte_vec_t message /* wasm_message_t* == wasm_byte_vec_t */ );
        
        // *******************************
        // Last error
        // *******************************
        [DllImport(LibraryName, EntryPoint = "wasmer_last_error_length", CallingConvention = CallingConvention.Cdecl)]
        public static extern int wasmer_last_error_length();

        [DllImport(LibraryName, EntryPoint = "wasmer_last_error_message", CallingConvention = CallingConvention.Cdecl)]
        public static extern int wasmer_last_error_message(IntPtr buffer, int length);

        // *******************************
        // Helper functions
        // *******************************
        /// <summary>
        /// Returns export name associated with an unmanaged extern pointer
        /// </summary>
        /// <param name="wasmExternPtr">Extern pointer (`wasm_extern_t *`)</param>
        /// <returns>Name</returns>
        public static string GetExportName(IntPtr wasmExternPtr)
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
        /// <summary>
        /// EXecutes a WebAssembly function call
        /// </summary>
        /// <param name="name">Function name</param>
        /// <param name="functionPtr">Function pointer </param>
        /// <param name="args">Argument list</param>
        /// <param name="hasResult">Flag indicating whether the function is expected to return a result</param>
        /// <param name="expectedResultType">Expected result (applies only if `hasResult` is `true`)</param>
        /// <param name="resultBuffer">Buffer to receive the result (if any)</param>
        /// <returns>Nothing</returns>
        public static void CallFunction(
            string name, IntPtr functionPtr, wasm_val_t[] args,
            bool hasResult, wasm_valkind_t expectedResultType, ref wasm_val_t resultBuffer
        )
        {
            Exception? exception = null;
            GCHandle argsHandle = GCHandle.Alloc(args, GCHandleType.Pinned);
            IntPtr argsPtr = argsHandle.AddrOfPinnedObject();
            wasm_val_vec_t argsVec;
            wasm_val_vec_new(out argsVec, (UIntPtr)args.Length, argsPtr);
            WasmerInterop.wasm_val_vec_t resultVec = new WasmerInterop.wasm_val_vec_t();
            if (hasResult)
                WasmerInterop.wasm_val_vec_new_uninitialized(ref resultVec, 1);

            IntPtr trap = WasmerInterop.wasm_func_call(functionPtr, ref argsVec, ref resultVec);
            if (trap == IntPtr.Zero)
            {
                if (hasResult)
                {
                    WasmerInterop.wasm_val_t value = Marshal.PtrToStructure<WasmerInterop.wasm_val_t>(resultVec.data);
                    if (value.kind == expectedResultType)
                    {
                        resultBuffer = value;
                    }
                    else
                    {
                        exception = new Exception($"Wasmer call to WebAssembly function '{name}' returned unexpected result type {value.kind.ToString()} " +
                                                   "(expected: '{expectedResultType.ToString()})");
                    }
                }
            }
            else
            {
                exception = new Exception($"Wasmer call to WebAssembly function '{name}' failed - {WasmerInterop.GetTrapMessage(trap)}");
                WasmerInterop.wasm_trap_delete(trap);
            }
            // Tidy
            wasm_val_vec_delete(ref argsVec);
            wasm_val_vec_delete(ref resultVec);
            argsHandle.Free();

            if (exception != null)
                throw (exception);
        }
         /// <summary>
        /// Returns the latest error triggered within Wasmer (if any)
        /// </summary>
        /// <returns>Error string</returns>
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
        /// <summary>
        /// Returns the message associated with a trap
        /// </summary>
        /// <returns>Error string</returns>
        public static string GetTrapMessage(IntPtr trap)
        {
            if (trap == IntPtr.Zero)
                return "";

            WasmerInterop.wasm_byte_vec_t rawBytes = default;
            WasmerInterop.wasm_trap_message(trap, ref rawBytes);
            return Marshal.PtrToStringUTF8(rawBytes.data, (int)rawBytes.size);
        }
    }
    /// <summary>
    /// Provides a wrapper to [Wasmer](https://wasmer.io/) WebAssembly memory
    /// </summary>
    internal class MazeWasmerMemory : IMemory
    {
        IntPtr _wasmMemoryPtr = IntPtr.Zero;
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="wasmMemoryPtr">WebAssembly memory pointer</param>
        internal MazeWasmerMemory(IntPtr wasmMemoryPtr)
        {
            if(wasmMemoryPtr == IntPtr.Zero)
            {
                throw new Exception("Zero wasmMemoryPtr supplied to MazeWasmerMemory constructor");
            }
            _wasmMemoryPtr = wasmMemoryPtr;
        }
        /// <summary>
        /// Reads an unsigned integer from unmanaged memory
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to value</param>
        /// <returns>Value</returns>
        public UInt32 ReadUInt32(UInt32 ptrOffset)
        {
            IntPtr memoryBase = WasmerInterop.wasm_memory_data(_wasmMemoryPtr);
            IntPtr valuePtr = IntPtr.Add(memoryBase, (int)ptrOffset);
            return (UInt32)Marshal.ReadInt32(valuePtr);
        }
        /// <summary>
        /// Writes an array of bytes to a give target unmanaged memory offset,
        /// which is assumed to have sufficient space
        /// </summary>
        /// <param name="ptrTargetOffset">Target memory pointer offset to write to</param>
        /// <param name="bytes">Byte array</param>
        /// <returns>Value</returns>
        public void WriteBytes(UInt32 ptrTargetOffset, byte[] bytes)
        {
            IntPtr memoryBase = WasmerInterop.wasm_memory_data(_wasmMemoryPtr);
            IntPtr bytesPtr = IntPtr.Add(memoryBase, (int)ptrTargetOffset);
            Marshal.Copy(bytes, 0, bytesPtr, bytes.Length);
        }
        /// <summary>
        /// Reads a `MazeWasmResult` pointer into a `MazeWasmResult`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to result</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeWasmInterop.MazeWasmResult ReadMazeWasmResult(UInt32 ptrOffset)
        {
            IntPtr memoryBase = WasmerInterop.wasm_memory_data(_wasmMemoryPtr);
            IntPtr resultPtr = IntPtr.Add(memoryBase, (int)ptrOffset);
            return Marshal.PtrToStructure<MazeWasmInterop.MazeWasmResult>(resultPtr);
        }
        /// <summary>
        /// Reads a `MazeWasmPoint` pointer into a `MazeWasmPoint`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to point</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeWasmInterop.MazeWasmPoint ReadMazeWasmPoint(UInt32 ptrOffset)
        {
            IntPtr memoryBase = WasmerInterop.wasm_memory_data(_wasmMemoryPtr);
            IntPtr pointPtr = IntPtr.Add(memoryBase, (int)ptrOffset);
            return Marshal.PtrToStructure<MazeWasmInterop.MazeWasmPoint>(pointPtr);
        }
        /// <summary>
        /// Reads a `MazeWasmError` pointer into a `MazeWasmError`
        /// </summary>
        /// <param name="ptrOffset">Memory pointer offset to error</param>
        /// <returns>`MazeWasmResult` value</returns>
        public MazeWasmInterop.MazeWasmError ReadMazeWasmError(UInt32 ptrOffset)
        {
            IntPtr memoryBase = WasmerInterop.wasm_memory_data(_wasmMemoryPtr);
            IntPtr errorPtr = IntPtr.Add(memoryBase, (int)ptrOffset);
            return Marshal.PtrToStructure<MazeWasmInterop.MazeWasmError>(errorPtr);
        }
        /// <summary>
        /// Extracts the string value from a string pointer, else throws
        /// an exception if the operaiton failed.
        /// </summary>
        /// <param name="ptrOffset">Memory offset pointer to string</param>
        /// <returns>String value if successful</returns>
        public string StringPtrToString(UInt32 ptrOffset) 
        {
            IntPtr memoryBase = WasmerInterop.wasm_memory_data(_wasmMemoryPtr);
            uint memorySize = WasmerInterop.wasm_memory_data_size(_wasmMemoryPtr);
            if (ptrOffset > memorySize)
            {
                throw new Exception($"string pointer offset {ptrOffset} is out of bounds (memory size = {memorySize})");
            }
            IntPtr lengthStart = memoryBase + (int)ptrOffset;
            Int32 length = Marshal.ReadInt32(lengthStart);
            IntPtr stringPointer = IntPtr.Add(memoryBase, (int)ptrOffset + 4);
            byte[] buffer = new byte[length];
            Marshal.Copy(stringPointer, buffer, 0, length);
            return Encoding.UTF8.GetString(buffer);
        }
    };
    /// <summary>
    ///  This class wraps a Wasmer WebAssembly function
    /// </summary>
    class MazeWasmerFunction : IFunction
    {
        private string _name = "";
        private IntPtr _ptr = IntPtr.Zero; // type: wasm_func_t * 
        private List<wasm_valkind_t> _argTypes = new List<wasm_valkind_t>();
        private bool _hasResult = false;
        private List<wasm_valkind_t> _resultTypes = new List<wasm_valkind_t>();
        private wasm_valkind_t _expectedResultType = wasm_valkind_t.NONE;
        /// <summary>
        /// The function name within the WebAssembly
        /// </summary>
        /// <returns>WebAssembly function name</returns>
        public string Name { get => this._name; }
        /// <summary>
        /// The managed Wasmer function pointer
        /// </summary>
        /// <returns>WebAssembly function pointer</returns>
        public IntPtr Ptr { get => this._ptr; }
        /// <summary>
        /// The number of arguments expected
        /// </summary>
        /// <returns>Argument count</returns>
        public int ArgCount{ get => this._argTypes.Count; }
        /// <summary>
        /// Whether the function returns a result
        /// </summary>
        /// <returns>Boolean</returns>
        public bool HasResult { get => this._hasResult; }
        /// <summary>
        /// The expected function result type
        /// </summary>
        /// <returns>Result kind</returns>
        public wasm_valkind_t ExpectedResultType { get => this._expectedResultType; }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="name">Function name within WebAssembly</param>
        /// <param name="ptr">Function pointer within Wasmer</param>
        public MazeWasmerFunction(string name, IntPtr ptr)
        {
            this._name = name;
            this._ptr = ptr;

            // Capture parameters and results
            IntPtr funcType = WasmerInterop.wasm_func_type(ptr);
            IntPtr paramsPtr = WasmerInterop.wasm_functype_params(funcType);
            IntPtr resultsPtr = WasmerInterop.wasm_functype_results(funcType);

            WasmerInterop.wasm_valtype_vec_t paramsVec = Marshal.PtrToStructure<WasmerInterop.wasm_valtype_vec_t>(paramsPtr);
            WasmerInterop.wasm_valtype_vec_t resultsVec = Marshal.PtrToStructure<WasmerInterop.wasm_valtype_vec_t>(resultsPtr);

            for (int p = 0; p < (int)paramsVec.size; p++)
            {
                IntPtr paramTypePtr = Marshal.ReadIntPtr(paramsVec.data, p * IntPtr.Size);
                _argTypes.Add(GetValueKind(_name, true, p, paramTypePtr));
            }

            for (int r = 0; r < (int)resultsVec.size; r++)
            {
                IntPtr resultTypePtr = Marshal.ReadIntPtr(resultsVec.data, r * IntPtr.Size);
                _resultTypes.Add(GetValueKind(_name, false, r, resultTypePtr));
            }

            _hasResult = _resultTypes.Count > 0;
            if (_hasResult)
                _expectedResultType = _resultTypes[0];
        }
        /// <summary>
        /// Returns the kind of value associated with a given function argument or result value type
        /// </summary>
        /// <param name="functionName">Function name</param>
        /// <param name="isArgument">Flag indcating whether the value is an argument. If `false`, then it is a result.</param>
        /// <param name="index">Argument or result index (zero-based)</param>
        /// <param name="valtypePtr">Value type pointer (`byte *`)</param>
        /// <returns>Kind of value</returns>
        static wasm_valkind_t GetValueKind(string functionName, bool isArgument, int index, IntPtr valtypePtr)
        {
            string Context() => isArgument ? "argument" : "result";
            int kind = Marshal.ReadByte(valtypePtr);
            return kind switch
            {
                0 => wasm_valkind_t.I32,
                1 => wasm_valkind_t.I64,
                2 => wasm_valkind_t.F32,
                3 => wasm_valkind_t.F64,
                128 => wasm_valkind_t.EXTERNREF,
                129 => wasm_valkind_t.WASM_FUNCREF,
                _ => throw new Exception($"WebAssembly function '{functionName}' could not be loaded - the function has an unsupported {Context()} value type '{kind}' at index {index}")
            };
        }
        /// <summary>
        /// Invokes the given function with the given arguments
        /// </summary>
        /// <param name="args">Function arguments</param>
        /// <returns>Result (will be `null` if the function has no result)</returns>
        public object? Invoke(params object[] args)
        {
            object? result = null;
            int argCount = args.Length;
            if (argCount != ArgCount)
            {
                throw new Exception($"Incorrect number of arguments supplied for WebAssembly function '{this.Name}' - expected {this.ArgCount} but {argCount} supplied");
            }
            WasmerInterop.wasm_val_t resultBuffer = new WasmerInterop.wasm_val_t();
            WasmerInterop.CallFunction(Name, Ptr, ArgCount > 0 ? ToCallArgs(args): [], HasResult, ExpectedResultType, ref resultBuffer);
            if (HasResult)
            {
                result = resultBuffer.ToValue();
            }
            return result;
        }
        /// <summary>
        /// Converts the given set of object args into an array of `wasm_val_t` values 
        /// </summary>
        /// <param name="args">Arguments to convert</param>
        /// <returns>Converted array</returns>
        private WasmerInterop.wasm_val_t[] ToCallArgs(object[] args)
        {
            int argCount = args.Length;
            if (argCount == 0)
                return [];

            if (argCount != ArgCount)
            {
                throw new Exception($"Incorrect number of arguments supplied to function '{Name}'- expected {this.ArgCount} but {argCount} supplied");
            }

            wasm_val_t[] callArgs = new wasm_val_t[args.Length];
            try
            {
                for (int i = 0; i < argCount; i++)
                {
                    callArgs[i].FromValue(this._argTypes[i], args[i]);
                }
            }
            catch (Exception e)
            {
                throw new Exception($"Failed to prepare arguments for WebAssembly call to function '{Name}' - {e.Message}");
            }
            return callArgs;
        }
    }
    /// <summary>
    ///  This class provides a C# connector to the `maze_wasm` web assembly via [Wasmer](https://wasmer.io/), insulating the
    ///  calling application from the specifics of the underlying Web Assembly interop operations.
    ///  
    /// Developers can use <see cref="MazeWasmConnectorBase.NewMazeWasm()">NewMazeWasm()</see> to create
    /// a pointer to a maze object within Web Assembly and then other `MazeWasm` functions, such as 
    ///  <see cref="MazeWasmConnectorBase.MazeWasmInsertRows(UInt32,UInt32,UInt32)">MazeWasmInsertRows()</see> and 
    ///  <see cref="MazeWasmConnectorBase.MazeWasmSolve(UInt32)">MazeWasmSolve()</see>, to interact with the maze.
    ///  
    /// Once finished with, a maze should be destroyed using <see cref="MazeWasmConnectorBase.FreeMazeWasm(UInt32)">FreeMazeWasm()</see>
    /// to prevent memory leaks within Web Assembly.
    /// </summary>
    class MazeWasmerConnector : MazeWasmConnectorBase, IMazeWasmConnector
    {
        bool _disposed = false;

        // Wasmtime Store and Instance
        string instanceWasmPath;

        IntPtr _engine = IntPtr.Zero;
        IntPtr _store = IntPtr.Zero;
        WasmerInterop.wasm_byte_vec_t _wasmVec = new WasmerInterop.wasm_byte_vec_t();
        GCHandle _handle = default;
        IntPtr _module = IntPtr.Zero;
        IntPtr _instance = IntPtr.Zero;
        WasmerInterop.wasm_extern_vec_t _emptyImports = new WasmerInterop.wasm_extern_vec_t();
        WasmerInterop.wasm_extern_vec_t _instanceExports = new WasmerInterop.wasm_extern_vec_t();
        WasmerInterop.wasm_exporttype_vec_t _moduleExports = new WasmerInterop.wasm_exporttype_vec_t();

        // WebAssembly functions (initialized and validated on creation)
        public delegate ref IFunction? RefFunctionGetter();
        Dictionary<string, RefFunctionGetter>? _functionMap;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="wasmPath">WebAssembly path</param>
        public MazeWasmerConnector(string wasmPath)
        {
            instanceWasmPath = wasmPath;
            Initialize();
        }
        /// <summary>
        /// Handles object finalization (deletion)
        /// </summary>
        /// <returns>Nothing</returns>
        ~MazeWasmerConnector()
        {
            Dispose(false);
        }
        /// <summary>
        /// Handles object disposal, releasing managed and unmanaged resources and marking
        /// the object as having been finalized
        /// </summary>
        /// <returns>Nothing</returns>
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }
        /// <summary>
        /// Handles object disposal
        /// </summary>
        /// <param name="disposing">Flag indicating whether the object should be fully disposed (ie. including managed
        /// as well as unmanaged  resources)</param>
        /// <returns>Nothing</returns>
        protected virtual void Dispose(bool disposing)
        {
            if (!_disposed)
            {
                // Tidy resources
                if (_instance != IntPtr.Zero)
                    WasmerInterop.wasm_instance_delete(_instance);

                if (_module != IntPtr.Zero)
                    WasmerInterop.wasm_module_delete(_module);

                if (_store != IntPtr.Zero)
                    WasmerInterop.wasm_store_delete(_store);

                if (_engine != IntPtr.Zero)
                    WasmerInterop.wasm_engine_delete(_engine);

                if (_wasmVec.data != IntPtr.Zero)
                    WasmerInterop.wasm_byte_vec_delete(ref _wasmVec);

                if (_handle.IsAllocated)
                    _handle.Free();

                _disposed = true;
            }
        }
        /// <summary>
        /// Initializes the object
        /// </summary>
        /// <returns>Nothing</returns>
        private void Initialize()
        {
            InitializeModule();
            InitializePointers();
        }
        /// <summary>
        /// Initializes the Wasmer engine and loads the target WebAssembly into a module
        /// </summary>
        /// <returns>Nothing</returns>
        private void InitializeModule()
        {
            _engine = WasmerInterop.wasm_engine_new();
            if (_engine == IntPtr.Zero)
            {
                throw new Exception("Failed to create Wasmer engine.");
            }
            _store = WasmerInterop.wasm_store_new(_engine);
            if (_store == IntPtr.Zero)
            {
                throw new Exception("Failed to create Wasm store.");
            }

            byte[] wasmBytes = File.ReadAllBytes(instanceWasmPath);
            _wasmVec = new WasmerInterop.wasm_byte_vec_t();

            _handle = GCHandle.Alloc(wasmBytes, GCHandleType.Pinned);

            WasmerInterop.wasm_byte_vec_new(ref _wasmVec, (nuint)wasmBytes.Length, _handle.AddrOfPinnedObject());

            bool isValid = WasmerInterop.wasm_module_validate(_store, ref _wasmVec);

            if (!isValid)
            {
                throw new Exception("Web assembly module is invalid.");
            }

            _module = WasmerInterop.wasm_module_new(_store, ref _wasmVec);
            if (_module == IntPtr.Zero)
            {
                throw new Exception($"Failed to create Wasm module: {WasmerInterop.GetLastError()}");
            }

            _instance = WasmerInterop.wasm_instance_new(_store, _module, ref _emptyImports, IntPtr.Zero);
            if (_instance == IntPtr.Zero)
            {
                string errorMessage = WasmerInterop.GetLastError();
                throw new Exception($"Failed to instantiate module => {errorMessage}");
            }
        }
        /// <summary>
        /// Attempts to initializes the WebAssembly pointers (functions, memory) associated with the instance. 
        /// Will throw an exception if any required pointer is not found.
        /// </summary>
        /// <returns>Nothing</returns>
        private void InitializePointers()
        {
            WasmerInterop.wasm_instance_exports(_instance, ref _instanceExports /* wasm_extern_vec_t * */);
            WasmerInterop.wasm_module_exports(_module, ref _moduleExports);

            InitializeFunctionMap();

            for (int i = 0; i < (int)_moduleExports.size; i++)
            {
                IntPtr exportTypePtr = Marshal.ReadIntPtr(_moduleExports.data, i * IntPtr.Size);
                IntPtr exportTypeNamePtr = WasmerInterop.wasm_exporttype_name(exportTypePtr);
                WasmerInterop.wasm_byte_vec_t rawBytes = Marshal.PtrToStructure<WasmerInterop.wasm_byte_vec_t>(exportTypeNamePtr);
                string name = Marshal.PtrToStringUTF8(rawBytes.data, (int)rawBytes.size);

                IntPtr externTypePtr = WasmerInterop.wasm_exporttype_type(exportTypePtr);
                WasmerInterop.ExternKind externKind = WasmerInterop.wasm_externtype_kind(externTypePtr);
                IntPtr wasmExternPtr = Marshal.ReadIntPtr(_instanceExports.data + i * IntPtr.Size);

                if (externKind == WasmerInterop.ExternKind.Function)
                {
                    IntPtr wasmFuncPtr = WasmerInterop.wasm_extern_as_func(wasmExternPtr);
                    if (wasmFuncPtr != IntPtr.Zero)
                    {
                        AssignFunctionPtr(name, wasmFuncPtr);
                    }
                }
                else if (externKind == WasmerInterop.ExternKind.Memory && name == "memory")
                {
                    memory = new MazeWasmerMemory(WasmerInterop.wasm_extern_as_memory(wasmExternPtr));
                }
            }

            VerifyFunctions();
            VerifyMemoryPtrs();
        }
        /// <summary>
        /// Initializes the function map associated with the instance, which is used to
        /// map function pointer fields of the instance by name, with empty pointer values
        /// /// </summary>
        /// <returns>Nothing</returns>
        private void InitializeFunctionMap()
        {
            _functionMap = new Dictionary<string, RefFunctionGetter>
            {
                { "new_maze_wasm", () => ref this.newMazeWasm },
                { "free_maze_wasm", () => ref this.freeMazeWasm },
                { "maze_wasm_is_empty", () => ref this.mazeWasmIsEmpty },
                { "maze_wasm_resize", () => ref this.mazeWasmResize },
                { "maze_wasm_reset", () => ref this.mazeWasmReset },
                { "maze_wasm_get_row_count", () => ref this.mazeWasmGetRowCount },
                { "maze_wasm_get_col_count", () => ref this.mazeWasmGetColCount },
                { "maze_wasm_get_cell_type", () => ref this.mazeWasmGetCellType },
                { "maze_wasm_set_start_cell", () => ref this.mazeWasmSetStartCell },
                { "maze_wasm_get_start_cell", () => ref this.mazeWasmGetStartCell },
                { "maze_wasm_set_finish_cell", () => ref this.mazeWasmSetFinishCell },
                { "maze_wasm_get_finish_cell", () => ref this.mazeWasmGetFinishCell },
                { "maze_wasm_set_wall_cells", () => ref this.mazeWasmSetWallCells },
                { "maze_wasm_clear_cells", () => ref this.mazeWasmClearCells },
                { "maze_wasm_insert_rows", () => ref this.mazeWasmInsertRows },
                { "maze_wasm_delete_rows", () => ref this.mazeWasmDeleteRows },
                { "maze_wasm_insert_cols", () => ref this.mazeWasmInsertCols },
                { "maze_wasm_delete_cols", () => ref this.mazeWasmDeleteCols },
                { "maze_wasm_from_json", () => ref this.mazeWasmFromJson },
                { "maze_wasm_to_json", () => ref this.mazeWasmToJson },
                { "maze_wasm_solve", () => ref this.mazeWasmSolve },
                { "maze_wasm_solution_get_path_points", () => ref this.mazeWasmSolutionGetPathPoints },
                { "free_maze_wasm_result", () => ref this.freeMazeWasmResult },
                { "free_maze_wasm_solution", () => ref this.freeMazeWasmSolution },
                { "free_maze_wasm_error", () => ref this.freeMazeWasmError },
                { "allocate_sized_memory", () => ref this.allocateSizedMemory },
                { "free_sized_memory", () => ref this.freeSizedMemory },
                { "get_sized_memory_used", () => ref this.getSizedMemoryUsed },
                { "get_num_objects_allocated", () => ref this.getNumObjectsAllocated }
            };
        }
        /// <summary>
        /// Assigns a function pointer to the instance's corresponding field (if recognized)
        /// </summary>
        /// <param name="name">Function name</param>
        /// <param name="wasmFuncPtr">Function pointer</param>
        /// <returns>Nothing</returns>
        private void AssignFunctionPtr(string name, IntPtr wasmFuncPtr)
        {
            if (_functionMap == null) return;

            if (_functionMap.ContainsKey(name))
            {
                ref IFunction? function = ref _functionMap[name]();
                function = new MazeWasmerFunction(name, wasmFuncPtr);
            }
        }
        /// <summary>
        /// Verifies that all required WebAssembly functions have been located and their pointers captured.
        /// Will throw an exception if any are missing.
        /// </summary>
        /// <returns>Nothing</returns>
        private void VerifyFunctions()
        {
            if (_functionMap == null) return;
            foreach (var kvp in _functionMap)
            {
                string functionName = kvp.Key;
                ref IFunction? function = ref kvp.Value();
                if (function == null)
                {
                    throw new Exception($"Failed to load the WebAssembly function: {functionName} in {instanceWasmPath}.");
                }
            }
        }
        /// <summary>
        /// Verifies that all required WebAssembly memory has been located and their pointers captured.
        /// Will throw an exception if any are missing.
        /// </summary>
        /// <returns>Nothing</returns>
        private void VerifyMemoryPtrs()
        {
            if (memory == null)
                throw new Exception($"Failed to load the WebAssembly memory from {instanceWasmPath}.");
        }
    }
}
