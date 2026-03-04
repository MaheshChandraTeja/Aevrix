using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Media.Imaging;
using Avalonia.PixelSize;
using Avalonia.Platform;
using Avalonia.Threading;

namespace Aevrix.App.Views;

public partial class ShellView : Window
{
    public ObservableCollection<BrowserTab> Tabs { get; } = new();
    public BrowserTab? SelectedTab { get; set; }

    public string? SelectedTitle => SelectedTab?.Title;

    public ShellView()
    {
        InitializeComponent();
#if DEBUG
        this.AttachDevTools();
#endif
        DataContext = this;

        
        HbNative.hb_init(1024, 700);

        
        var initialHtml = """
            <style> body { background:#0b0b0f; color:#e6e6e6; font-size:18px } .card{background:#181a1f} </style>
            <div style="background:#181a1f; padding:8px">
            <p style="font-size:20px;color:#a0a0ff">Aevrix</p>
            <span>Deterministic first paint — Rust engine • Avalonia host</span>
            </div>
            """;
        var t = new BrowserTab("New Tab", initialHtml);
        Tabs.Add(t);
        SelectedTab = t;
        RenderTab(t);
        this.SizeChanged += (_, __) => { if (SelectedTab != null) RenderTab(SelectedTab); };
    }

    private void OnNewTabClick(object? sender, Avalonia.Interactivity.RoutedEventArgs e)
    {
        var t = new BrowserTab("New Tab", "<div style='font-size:18px'>Blank</div>");
        Tabs.Add(t);
        SelectedTab = t;
        RenderTab(t);
    }

    private void OnCloseTabClick(object? sender, Avalonia.Interactivity.RoutedEventArgs e)
    {
        if (SelectedTab == null) return;
        var idx = Tabs.IndexOf(SelectedTab);
        Tabs.Remove(SelectedTab);
        if (Tabs.Count == 0) return;
        SelectedTab = Tabs[Math.Clamp(idx - 1, 0, Tabs.Count - 1)];
    }

    private void OnRefreshClick(object? sender, Avalonia.Interactivity.RoutedEventArgs e)
    {
        if (SelectedTab != null) RenderTab(SelectedTab);
    }

    private void OnAddressKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key == Key.Enter && SelectedTab != null)
        {
            var text = AddressBox.Text ?? "";
            if (LooksLikeUrl(text))
            {
                
                SelectedTab.Html = $$"""
                    <div style="background:#181a1f">
                      <p style="color:#ffcc66;font-size:18px">Networking disabled (offline snapshot mode)</p>
                      <span>Requested URL: {{Escape(text)}}</span>
                    </div>
                    """;
                SelectedTab.Title = "Offline";
            }
            else
            {
                SelectedTab.Html = text;
                SelectedTab.Title = "Doc";
            }
            RenderTab(SelectedTab);
        }
    }

    private static bool LooksLikeUrl(string s) =>
        s.StartsWith("http://", StringComparison.OrdinalIgnoreCase) ||
        s.StartsWith("https://", StringComparison.OrdinalIgnoreCase);

    private static string Escape(string s) => s.Replace("<", "&lt;").Replace(">", "&gt;");

    private void RenderTab(BrowserTab tab)
    {
        
        var host = this.FindControl<Image>("AevrixCanvas");
        int w = (int)Math.Max(1, host?.Bounds.Width ?? Bounds.Width);
        int h = (int)Math.Max(1, host?.Bounds.Height ?? (Bounds.Height - 50));

        var surf = new HbSurface();
        int rc = HbNative.hb_render_html(tab.Html, (uint)w, (uint)h, ref surf);
        if (rc != 0 || surf.pixels == IntPtr.Zero || surf.len == 0)
        {
            
            tab.Html = "<div style='color:#ff7777'>Render error</div>";
            HbNative.hb_surface_release(ref surf);
            return;
        }

        
        if (tab.Bitmap == null || tab.Bitmap.PixelSize.Width != surf.width || tab.Bitmap.PixelSize.Height != surf.height)
        {
            tab.Bitmap = new WriteableBitmap(new Avalonia.PixelSize(surf.width, surf.height),
                                             new Avalonia.Vector(96, 96),
                                             Avalonia.Platform.PixelFormat.Rgba8888,
                                             Avalonia.Platform.AlphaFormat.Unpremul);
        }

        
        using (var fb = tab.Bitmap.Lock())
        {
            unsafe
            {
                Buffer.MemoryCopy((void*)surf.pixels, (void*)fb.Address, surf.len, surf.len);
            }
        }

        HbNative.hb_surface_release(ref surf);
        tab.RaiseChanged(nameof(BrowserTab.Bitmap));
        tab.RaiseChanged(nameof(BrowserTab.Title));
    }
}



public class BrowserTab : INotifyPropertyChanged
{
    private string _title;
    private string _html;

    public BrowserTab(string title, string html)
    {
        _title = title;
        _html = html;
    }

    public string Title { get => _title; set { _title = value; RaiseChanged(); } }
    public string Html { get => _html; set { _html = value; RaiseChanged(); } }

    public WriteableBitmap? Bitmap { get; set; }

    public event PropertyChangedEventHandler? PropertyChanged;
    public void RaiseChanged([CallerMemberName] string? n = null) =>
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(n));
}



[StructLayout(LayoutKind.Sequential)]
public struct HbSurface
{
    public IntPtr pixels;
    public uint width;
    public uint height;
    public uint stride;
    public nuint len;
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
    public static extern IntPtr hb_version();

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl)]
    public static extern int hb_init(uint viewport_width, uint viewport_height);

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl)]
    public static extern uint hb_load_html([MarshalAs(UnmanagedType.LPUTF8Str)] string html_utf8);

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl)]
    public static extern int hb_render_html([MarshalAs(UnmanagedType.LPUTF8Str)] string html_utf8, uint w, uint h, ref HbSurface outSurf);

    [DllImport(LIB, CallingConvention = CallingConvention.Cdecl)]
    public static extern void hb_surface_release(ref HbSurface surf);
}
