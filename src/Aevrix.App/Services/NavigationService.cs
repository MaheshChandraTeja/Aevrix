using System;

namespace Hummingbird.App.Services;

public interface INavigationService
{
    
    (string title, string html) Resolve(string address);
}

public sealed class NavigationService : INavigationService
{
    public (string title, string html) Resolve(string address)
    {
        var text = address?.Trim() ?? string.Empty;
        if (LooksLikeUrl(text))
        {
            
            var safe = Escape(text);
            var html =
                $$"""
                <div style="background:#181a1f">
                  <p style="color:#ffcc66;font-size:18px">Networking disabled (offline snapshot mode)</p>
                  <span>Requested URL: {{safe}}</span>
                </div>
                """;
            return ("Offline", html);
        }

        
        return ("Document", text);
    }

    private static bool LooksLikeUrl(string s) =>
        s.StartsWith("http://", StringComparison.OrdinalIgnoreCase) ||
        s.StartsWith("https://", StringComparison.OrdinalIgnoreCase);

    private static string Escape(string s) => s.Replace("<", "&lt;").Replace(">", "&gt;");
}
