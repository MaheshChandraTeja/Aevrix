using System;

namespace Hummingbird.App.Models;

public class TabModel
{
    public Guid Id { get; init; } = Guid.NewGuid();
    public string Title { get; set; } = "New Tab";
    public string Html { get; set; } = "<div style='font-size:18px'>Blank</div>";

    public static TabModel CreateInitial() => new()
    {
        Title = "Hummingbird",
        Html =
            """
            <style> body { background:#0b0b0f; color:#e6e6e6; font-size:18px } </style>
            <div style="background:#181a1f; padding:8px">
              <p style="font-size:20px;color:#a0a0ff">Hummingbird</p>
              <span>Deterministic first paint — Rust engine · Avalonia host</span>
            </div>
            """
    };
}
