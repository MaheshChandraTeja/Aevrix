using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Hummingbird.App.Services.EngineHost;





internal static class EngineNative
{
    private const string LIB =
#if WINDOWS
        "hb_ffi.dll";
#elif OSX
        "libhb_ffi.dylib";
#else
        "libhb_ffi.so";
#endif

    [StructLayout(LayoutKind.Sequential)]
    internal struct hb_surface
    {
        public IntPtr pixels;
        public uint width;
        public uint height;
        public uint stride;
#if NET8_0_OR_GREATER
        public nuint len;
#else
        public UIntPtr len;
#endif
    }

    

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl, EntryPoint = "hb_version")]
    private static extern IntPtr _hb_version();

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl, EntryPoint = "hb_init")]
    private static extern int _hb_init(uint viewport_width, uint viewport_height);

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl, EntryPoint = "hb_render_html")]
    private static extern int _hb_render_html(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string html_utf8,
        uint viewport_width,
        uint viewport_height,
        ref hb_surface outSurf);

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl, EntryPoint = "hb_surface_release")]
    private static extern void _hb_surface_release(ref hb_surface surf);

    

    
    public static string Version
    {
        get
        {
            var ptr = _hb_version();
            return Marshal.PtrToStringUTF8(ptr) ?? "hb(dev)";
        }
    }

    
    public static void Init(uint viewportWidth, uint viewportHeight)
    {
        var rc = _hb_init(viewportWidth, viewportHeight);
        if (rc != 0)
            throw new InvalidOperationException($"hb_init failed with {rc}");
    }

    
    
    
    public static SafeSurface RenderHtml(string htmlUtf8, uint viewportWidth, uint viewportHeight)
    {
        var s = default(hb_surface);
        var rc = _hb_render_html(htmlUtf8 ?? string.Empty, viewportWidth, viewportHeight, ref s);
        if (rc != 0 || s.pixels == IntPtr.Zero || ToUInt64(s.len) == 0UL)
        {
            
            try { _hb_surface_release(ref s); } catch {  }
            throw new InvalidOperationException($"hb_render_html failed (rc={rc})");
        }
        return new SafeSurface(s);
    }

    
    internal static void Release(ref hb_surface s) => _hb_surface_release(ref s);

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static ulong ToUInt64(nuint v) => (ulong)v;

#if !NET8_0_OR_GREATER
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    private static ulong ToUInt64(UIntPtr v) => (ulong)v;
#endif

    

    
    
    
    public sealed class SafeSurface : IDisposable
    {
        private hb_surface _s;
        private bool _disposed;

        internal SafeSurface(hb_surface inner) => _s = inner;

        public IntPtr Pixels => _s.pixels;
        public int Width => unchecked((int)_s.width);
        public int Height => unchecked((int)_s.height);
        public int Stride => unchecked((int)_s.stride);
        public ulong Length => ToUInt64(_s.len);

        
        public byte[] ToArray()
        {
            var len = checked((int)Length);
            var dst = new byte[len];
            if (len > 0 && Pixels != IntPtr.Zero)
                Marshal.Copy(Pixels, dst, 0, len);
            return dst;
        }

        public void Dispose()
        {
            if (_disposed) return;
            try { EngineNative.Release(ref _s); }
            finally { _s = default; _disposed = true; }
        }
    }
}
