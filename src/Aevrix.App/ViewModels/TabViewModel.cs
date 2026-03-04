using System;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Threading;
using Hummingbird.App.Models;
using Hummingbird.App.Services;

namespace Hummingbird.App.ViewModels;

public class TabViewModel : INotifyPropertyChanged
{
    private readonly ISettingsService _settings;
    private readonly SemaphoreSlim _renderGate = new(1, 1);

    public TabModel Model { get; }
    public string Title { get => Model.Title; set { Model.Title = value; OnPropertyChanged(); } }
    public string Html { get => Model.Html; set { Model.Html = value; OnPropertyChanged(); } }

    private WriteableBitmap? _bitmap;
    public WriteableBitmap? Bitmap { get => _bitmap; private set { _bitmap = value; OnPropertyChanged(); } }

    private (int w, int h) _lastViewport = (800, 600);

    static TabViewModel()
    {
        
        HbNative.hb_init(800, 600);
    }

    public TabViewModel(TabModel model, ISettingsService settings)
    {
        _settings = settings;
        Model = model;
    }

    public async Task RenderAsync(int viewportWidth, int viewportHeight)
    {
        _lastViewport = (viewportWidth, viewportHeight);
        await _renderGate.WaitAsync().ConfigureAwait(false);
        try
        {
            var html = Html ?? string.Empty;

            var surf = new HbSurface();
            int rc = HbNative.hb_render_html(html, (uint)viewportWidth, (uint)viewportHeight, ref surf);
            if (rc != 0 || surf.pixels == IntPtr.Zero || surf.len == UIntPtr.Zero)
            {
                HbNative.hb_surface_release(ref surf);
                return;
            }

            
            await Dispatcher.UIThread.InvokeAsync(() =>
            {
                if (Bitmap == null || Bitmap.PixelSize.Width != (int)surf.width || Bitmap.PixelSize.Height != (int)surf.height)
                {
                    Bitmap = new WriteableBitmap(new PixelSize((int)surf.width, (int)surf.height),
                                                 new Avalonia.Vector(96, 96),
                                                 PixelFormat.Rgba8888,
                                                 AlphaFormat.Unpremul);
                }

                unsafe
                {
                    using var fb = Bitmap!.Lock();
                    Buffer.MemoryCopy((void*)surf.pixels, (void*)fb.Address, (long)surf.len, (long)surf.len);
                }
            });

            HbNative.hb_surface_release(ref surf);
        }
        finally
        {
            _renderGate.Release();
        }
    }

    public Task RenderAsyncCurrentViewport() => RenderAsync(_lastViewport.w, _lastViewport.h);

    public void RequestRender()
    {
        _ = RenderAsyncCurrentViewport();
    }

    public event PropertyChangedEventHandler? PropertyChanged;
    protected void OnPropertyChanged([CallerMemberName] string? n = null)
        => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(n));
}


[StructLayout(LayoutKind.Sequential)]
internal struct HbSurface
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

internal static class HbNative
{
    private const string LIB =
#if WINDOWS
        "hb_ffi.dll";
#elif OSX
        "libhb_ffi.dylib";
#else
        "libhb_ffi.so";
#endif

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl)]
    public static extern int hb_init(uint viewport_width, uint viewport_height);

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl)]
    public static extern int hb_render_html(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string html_utf8,
        uint w, uint h,
        ref HbSurface outSurf);

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl)]
    public static extern void hb_surface_release(ref HbSurface surf);
}
