using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using Avalonia.Input;
using Avalonia.Threading;
using Hummingbird.App.ViewModels;

namespace Hummingbird.App.Views;

public partial class TabView : UserControl
{
    private Image? _canvas;

    public TabView()
    {
        InitializeComponent();
        DataContextChanged += OnDataContextChanged;
        AttachedToVisualTree += OnAttachedToVisualTree;
        DetachedFromVisualTree += OnDetachedFromVisualTree;
    }

    private void InitializeComponent()
        => AvaloniaXamlLoader.Load(this);

    private void OnAttachedToVisualTree(object? sender, VisualTreeAttachmentEventArgs e)
    {
        _canvas = this.FindControl<Image>("AevrixCanvas") ?? this.FindControl<Image>("Canvas");
        if (_canvas != null)
        {
            _canvas.AttachedToVisualTree += Canvas_AttachedToVisualTree;
            _canvas.SizeChanged += Canvas_SizeChanged;
        }

        
        if (DataContext is TabViewModel vm && _canvas != null)
        {
            var (w, h) = Viewport();
            _ = vm.RenderAsync(w, h);
        }
    }

    private void OnDetachedFromVisualTree(object? sender, VisualTreeAttachmentEventArgs e)
    {
        if (_canvas != null)
        {
            _canvas.AttachedToVisualTree -= Canvas_AttachedToVisualTree;
            _canvas.SizeChanged -= Canvas_SizeChanged;
        }
    }

    private void OnDataContextChanged(object? sender, EventArgs e)
    {
        if (DataContext is TabViewModel vm && _canvas != null)
        {
            var (w, h) = Viewport();
            _ = vm.RenderAsync(w, h);
        }
    }

    private void Canvas_AttachedToVisualTree(object? sender, VisualTreeAttachmentEventArgs e)
    {
        if (DataContext is TabViewModel vm)
        {
            var (w, h) = Viewport();
            _ = vm.RenderAsync(w, h);
        }
    }

    private void Canvas_SizeChanged(object? sender, SizeChangedEventArgs e)
    {
        if (e.NewSize.Width <= 0 || e.NewSize.Height <= 0) return;
        if (DataContext is TabViewModel vm)
        {
            var (w, h) = Viewport();
            
            Dispatcher.UIThread.Post(() => _ = vm.RenderAsync(w, h), DispatcherPriority.Background);
        }
    }

    private (int w, int h) Viewport()
    {
        if (_canvas != null)
        {
            var b = _canvas.Bounds;
            return ((int)Math.Max(1, b.Width), (int)Math.Max(1, b.Height));
        }

        var self = Bounds;
        return ((int)Math.Max(1, self.Width), (int)Math.Max(1, self.Height));
    }
}
