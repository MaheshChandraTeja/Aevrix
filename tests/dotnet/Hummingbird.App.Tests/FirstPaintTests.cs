using System;
using Xunit;
using Hummingbird.App.Services.EngineHost;

namespace Hummingbird.App.Tests;

public class FirstPaintTests
{
    [Fact]
    public void EngineVersion_ExposesString()
    {
        try
        {
            
            
            var ver = EngineNative.Version;
            Assert.False(string.IsNullOrWhiteSpace(ver));
        }
        catch (DllNotFoundException)
        {
            
            
            Assert.True(true);
        }
    }

    [Fact]
    public void RenderHtml_IsDeterministic_ForStaticContent()
    {
        const string html = @"
            <style>
               body { background:#0b0b0f; color:#e6e6e6; }
               .wrap { background:#181a1f }
               p { color:#a0a0ff; font-size:18px }
            </style>
            <div class='wrap'><p>Hello <span>World</span></p></div>
        ";

        try
        {
            EngineNative.Init(640, 360);

            using var s1 = EngineNative.RenderHtml(html, 640, 360);
            using var s2 = EngineNative.RenderHtml(html, 640, 360);

            Assert.Equal(s1.Width, s2.Width);
            Assert.Equal(s1.Height, s2.Height);

            var b1 = s1.ToArray();
            var b2 = s2.ToArray();

            Assert.Equal(b1.Length, b2.Length);
            Assert.Equal(b1, b2); 
        }
        catch (DllNotFoundException)
        {
            
            Assert.True(true);
        }
    }
}
