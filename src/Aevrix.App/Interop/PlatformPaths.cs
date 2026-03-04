using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;

namespace Hummingbird.App.Interop;

internal static class PlatformPaths
{
    public static bool IsWindows => RuntimeInformation.IsOSPlatform(OSPlatform.Windows);
    public static bool IsLinux => RuntimeInformation.IsOSPlatform(OSPlatform.Linux);
    public static bool IsMacOS => RuntimeInformation.IsOSPlatform(OSPlatform.OSX);

    public static string Arch =>
        RuntimeInformation.ProcessArchitecture switch
        {
            Architecture.X64 => "x64",
            Architecture.X86 => "x86",
            Architecture.Arm64 => "arm64",
            Architecture.Arm => "arm",
            _ => "unknown"
        };

    
    
    
    public static IEnumerable<string> EngineCandidates(string baseDir)
    {
        var names = IsWindows
            ? new[] { "hb_ffi.dll" }
            : IsMacOS ? new[] { "libhb_ffi.dylib" }
                      : new[] { "libhb_ffi.so" };

        
        foreach (var n in names)
            yield return Path.Combine(baseDir, n);

        
        var rid = Rid();
        foreach (var n in names)
            yield return Path.Combine(baseDir, "runtimes", rid, "native", n);

        
        foreach (var n in names)
            yield return Path.GetFullPath(Path.Combine(baseDir, "..", "..", "engine", "target", "release", n));
    }

    private static string Rid()
    {
        if (IsWindows) return $"win-{Arch}";
        if (IsMacOS) return $"osx-{Arch}";
        if (IsLinux) return $"linux-{Arch}";
        return $"unknown-{Arch}";
    }

    public static string AppDataDirectory(string appName)
    {
        string dir;
        if (IsWindows)
            dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), appName);
        else if (IsMacOS)
            dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Personal), "Library", "Application Support", appName);
        else
            dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), appName);

        Directory.CreateDirectory(dir);
        return dir;
    }
}
