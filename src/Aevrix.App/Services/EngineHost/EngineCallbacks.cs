using System;

namespace Hummingbird.App.Services.EngineHost;





public static class EngineCallbacks
{
    

    
    public static event EventHandler<RenderCompletedEventArgs>? RenderCompleted;

    
    public static event EventHandler<EngineWarningEventArgs>? Warning;

    internal static void OnRenderCompleted(int width, int height, TimeSpan elapsed)
        => RenderCompleted?.Invoke(null, new RenderCompletedEventArgs(width, height, elapsed));

    internal static void OnWarning(string code, string message)
        => Warning?.Invoke(null, new EngineWarningEventArgs(code, message));

    public sealed class RenderCompletedEventArgs : EventArgs
    {
        public int Width { get; }
        public int Height { get; }
        public TimeSpan Elapsed { get; }
        public RenderCompletedEventArgs(int w, int h, TimeSpan elapsed)
        { Width = w; Height = h; Elapsed = elapsed; }
    }

    public sealed class EngineWarningEventArgs : EventArgs
    {
        public string Code { get; }
        public string Message { get; }
        public EngineWarningEventArgs(string code, string message)
        { Code = code; Message = message; }
    }
}
