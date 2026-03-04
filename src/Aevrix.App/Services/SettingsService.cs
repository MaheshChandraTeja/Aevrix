using System;
using System.IO;
using System.Text.Json;

namespace Hummingbird.App.Services;

public interface ISettingsService
{
    bool TelemetryEnabled { get; set; }          
    double UiScale { get; set; }                 
    void Save();
}

public sealed class SettingsService : ISettingsService
{
    private const string FileName = "settings.json";
    private readonly string _path;

    private record Bag(bool TelemetryEnabled, double UiScale);

    public bool TelemetryEnabled { get; set; } = false;
    public double UiScale { get; set; } = 1.0;

    public SettingsService()
    {
        _path = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "Hummingbird",
            FileName);

        TryLoad();
    }

    public void Save()
    {
        try
        {
            var dir = Path.GetDirectoryName(_path)!;
            Directory.CreateDirectory(dir);
            var bag = new Bag(TelemetryEnabled, UiScale);
            var json = JsonSerializer.Serialize(bag, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(_path, json);
        }
        catch
        {
            
        }
    }

    private void TryLoad()
    {
        try
        {
            if (!File.Exists(_path)) return;
            var json = File.ReadAllText(_path);
            var bag = JsonSerializer.Deserialize<Bag>(json);
            if (bag is null) return;
            TelemetryEnabled = bag.TelemetryEnabled;
            UiScale = bag.UiScale;
        }
        catch
        {
            
            TelemetryEnabled = false;
            UiScale = 1.0;
        }
    }
}
