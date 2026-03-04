using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;

namespace Hummingbird.App.Interop;





internal static class NativeLibraryResolver
{
    private static bool _installed;

    
    public static void Install()
    {
        if (_installed) return;
        _installed = true;

        NativeLibrary.SetDllImportResolver(Assembly.GetExecutingAssembly(), Resolve);
    }

    private static IntPtr Resolve(string libraryName, Assembly assembly, DllImportSearchPath? _)
    {
        if (!libraryName.Contains("hb_ffi", StringComparison.OrdinalIgnoreCase))
            return IntPtr.Zero;

        var baseDir = AppContext.BaseDirectory;
        var candidates = PlatformPaths.EngineCandidates(baseDir);

        foreach (var path in candidates)
        {
            try
            {
                if (File.Exists(path))
                {
                    var handle = NativeLibrary.Load(path);
                    if (handle != IntPtr.Zero) return handle;
                }
            }
            catch
            {
                
            }
        }

        
        return IntPtr.Zero;
    }
}
